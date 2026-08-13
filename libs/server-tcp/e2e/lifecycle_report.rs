use server_lifecycle::{
    ConnectionBudget, ConnectionMetrics, LifecycleController, LifecyclePhase, ShutdownDeadline,
};
use server_tcp::{TcpConnectionResult, TcpConnectionTerminal, TcpServerReport};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

#[derive(Default)]
struct Metrics {
    accepted: AtomicUsize,
    closed: AtomicUsize,
}
impl ConnectionMetrics for Metrics {
    fn connection_accepted(&self) {
        self.accepted.fetch_add(1, Ordering::SeqCst);
    }
    fn connection_closed(&self) {
        self.closed.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn report_is_typed_and_bounded() {
    let report = TcpServerReport::default();
    assert_eq!(report.accepted, 0);
    let result = TcpConnectionResult {
        terminal: TcpConnectionTerminal::Completed,
        streams_admitted: 1,
        streams_active_at_drain: 0,
        streams_completed: 1,
        streams_refused: 0,
        streams_timed_out: 0,
        streams_ambiguous: 0,
    };
    assert_eq!(result.streams_completed, 1);
}

#[tokio::test]
async fn listener_report_covers_success_failure_and_rejection() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let lifecycle = LifecycleController::serving();
    let control = lifecycle.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let metrics = Arc::new(Metrics::default());
    let gate = Arc::new(tokio::sync::Notify::new());
    let budget = ConnectionBudget::new(1);
    let config = server_tcp::TcpServerConfig::new(server_lifecycle::BindConfig::localhost(0))
        .with_connection_budget(budget)
        .with_connection_metrics(metrics.clone());
    let task = tokio::spawn(server_tcp::serve_with_report(
        listener,
        config,
        {
            let calls = Arc::clone(&calls);
            let gate = gate.clone();
            move |_stream, _cx| {
                let n = calls.fetch_add(1, Ordering::SeqCst);
                let gate = gate.clone();
                async move {
                    if n == 0 {
                        gate.notified().await;
                        TcpConnectionResult::default()
                    } else {
                        TcpConnectionResult {
                            terminal: TcpConnectionTerminal::Failed,
                            ..Default::default()
                        }
                    }
                }
            }
        },
        lifecycle,
    ));
    let mut first = tokio::net::TcpStream::connect(addr).await.expect("first");
    first.write_all(b"x").await.expect("write");
    let _second = tokio::net::TcpStream::connect(addr).await.expect("second");
    tokio::time::sleep(Duration::from_millis(20)).await;
    let deadline = ShutdownDeadline::from_now(Duration::from_millis(50), Duration::ZERO).unwrap();
    let _ = control.shutdown(deadline, "test", "stop").await;
    let report = task.await.expect("server");
    assert!(report.accepted >= 1);
    assert_eq!(
        report.accepted,
        report.completed + report.failed + report.timed_out + report.unfinished
    );
    assert!(report.rejected >= 1);
    assert_eq!(
        metrics.accepted.load(Ordering::SeqCst) as u64,
        report.accepted
    );
    assert_eq!(
        metrics.closed.load(Ordering::SeqCst) as u64,
        report.accepted
    );
}

#[tokio::test]
async fn already_draining_without_published_deadline_is_bounded() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("bind");
    let lifecycle = LifecycleController::serving();
    lifecycle
        .transition(LifecyclePhase::Draining, "test", "manual")
        .unwrap();
    let report = server_tcp::serve_with_report(
        listener,
        server_tcp::TcpServerConfig::new(server_lifecycle::BindConfig::localhost(0)),
        |_stream, _cx| async { TcpConnectionResult::default() },
        lifecycle,
    )
    .await;
    assert_eq!(report.accepted, 0);
    assert!(report.deadline_missing);
}
