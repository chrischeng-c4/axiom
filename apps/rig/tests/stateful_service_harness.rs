// HANDWRITE-BEGIN gap="missing-generator:unit-test:6e483bd5" tracker="#1645" reason="Exercise the runner against a real bounded local HTTP stateful fixture and prove failure evidence plus teardown behavior. generator gap: missing-generator:stateful-harness-test (#1645)."
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, bail, ensure, Result};
use rig::engine::stateful::{
    run_stateful, PhaseOutcome, StatefulActions, StatefulLimits, StatefulPhase, StatefulScenario,
};
use serde_json::json;

#[test]
fn real_http_service_survives_fault_and_preserves_state() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture");
    listener.set_nonblocking(true).expect("nonblocking fixture");
    let addr = listener.local_addr().expect("fixture address");
    let faulted = Arc::new(AtomicBool::new(false));
    let count = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let server = {
        let faulted = Arc::clone(&faulted);
        let count = Arc::clone(&count);
        let stop = Arc::clone(&stop);
        thread::spawn(move || serve_fixture(listener, faulted, count, stop))
    };
    let base = format!("http://{addr}");

    let report = run_stateful(
        StatefulScenario::new(
            "rig-real-http-stateful",
            StatefulActions {
                warmup: {
                    let base = base.clone();
                    Box::new(move |evidence| {
                        post(&format!("{base}/append"))?;
                        post(&format!("{base}/append"))?;
                        evidence.record("warmup_writes", json!({"count": 2}));
                        Ok(())
                    })
                },
                observe: {
                    let base = base.clone();
                    Box::new(move |evidence| {
                        let observed = get_count(&base)?;
                        evidence.record("steady_state", json!({"count": observed}));
                        ensure!(observed == 2, "expected two durable writes");
                        Ok(())
                    })
                },
                fault: {
                    let base = base.clone();
                    let faulted = Arc::clone(&faulted);
                    Box::new(move |evidence| {
                        faulted.store(true, Ordering::Release);
                        let unavailable = matches!(
                            ureq::get(&format!("{base}/count")).call(),
                            Err(ureq::Error::Status(503, _))
                        );
                        evidence.record("fault_observed", json!({"http_503": unavailable}));
                        ensure!(unavailable, "fault must make the service unavailable");
                        Ok(())
                    })
                },
                recover: {
                    let faulted = Arc::clone(&faulted);
                    Box::new(move |evidence| {
                        faulted.store(false, Ordering::Release);
                        evidence.record("service_recovered", json!({"faulted": false}));
                        Ok(())
                    })
                },
                verify: {
                    let base = base.clone();
                    Box::new(move |evidence| {
                        let observed = get_count(&base)?;
                        evidence.record("continuity", json!({"count": observed}));
                        ensure!(observed == 2, "fault recovery lost service state");
                        Ok(())
                    })
                },
                teardown: {
                    let stop = Arc::clone(&stop);
                    Box::new(move |evidence| {
                        stop.store(true, Ordering::Release);
                        let _ = TcpStream::connect(addr);
                        evidence.record("cleanup", json!({"listener_stopped": true}));
                        Ok(())
                    })
                },
            },
        )
        .with_limits(short_limits()),
    );

    server.join().expect("fixture server exits");
    assert!(report.passed, "{report:#?}");
    assert_eq!(report.failed_phase, None);
    assert_eq!(report.phases.len(), 6);
    assert!(report
        .phases
        .iter()
        .all(|phase| phase.outcome == PhaseOutcome::Passed));
    assert_eq!(
        report
            .evidence
            .iter()
            .map(|record| record.kind.as_str())
            .collect::<Vec<_>>(),
        [
            "warmup_writes",
            "steady_state",
            "fault_observed",
            "service_recovered",
            "continuity",
            "cleanup",
        ]
    );
}

#[test]
fn failed_observation_retains_evidence_and_always_tears_down() {
    let cleaned = Arc::new(AtomicBool::new(false));
    let report = run_stateful(
        StatefulScenario::new(
            "rig-failed-observation",
            StatefulActions {
                warmup: Box::new(|evidence| {
                    evidence.record("seeded", json!({"items": 3}));
                    Ok(())
                }),
                observe: Box::new(|evidence| {
                    evidence.record("bad_observation", json!({"visible": 2, "expected": 3}));
                    bail!("one item was not visible")
                }),
                fault: ok_action(),
                recover: ok_action(),
                verify: ok_action(),
                teardown: {
                    let cleaned = Arc::clone(&cleaned);
                    Box::new(move |evidence| {
                        cleaned.store(true, Ordering::Release);
                        evidence.record("cleanup", json!({"complete": true}));
                        Ok(())
                    })
                },
            },
        )
        .with_limits(short_limits()),
    );

    assert!(!report.passed);
    assert_eq!(report.failed_phase, Some(StatefulPhase::Observe));
    assert_eq!(
        report.phase(StatefulPhase::Observe).unwrap().outcome,
        PhaseOutcome::Failed
    );
    assert_eq!(
        report.phase(StatefulPhase::Fault).unwrap().outcome,
        PhaseOutcome::Skipped
    );
    assert_eq!(
        report.phase(StatefulPhase::Teardown).unwrap().outcome,
        PhaseOutcome::Passed
    );
    assert!(cleaned.load(Ordering::Acquire));
    assert_eq!(
        report
            .evidence
            .iter()
            .map(|record| record.kind.as_str())
            .collect::<Vec<_>>(),
        ["seeded", "bad_observation", "cleanup"]
    );
}

#[test]
fn timed_out_phase_is_bounded_and_teardown_still_runs() {
    let cleaned = Arc::new(AtomicBool::new(false));
    let report = run_stateful(
        StatefulScenario::new(
            "rig-timeout",
            StatefulActions {
                warmup: ok_action(),
                observe: Box::new(|evidence| {
                    evidence.record("before_wait", json!({"captured": true}));
                    thread::sleep(Duration::from_millis(100));
                    evidence.record("after_wait", json!({"must_not_leak": true}));
                    Ok(())
                }),
                fault: ok_action(),
                recover: ok_action(),
                verify: ok_action(),
                teardown: {
                    let cleaned = Arc::clone(&cleaned);
                    Box::new(move |_| {
                        cleaned.store(true, Ordering::Release);
                        Ok(())
                    })
                },
            },
        )
        .with_limits(StatefulLimits {
            phase_timeout: Duration::from_millis(10),
            scenario_timeout: Duration::from_millis(50),
            teardown_timeout: Duration::from_millis(50),
        }),
    );

    assert_eq!(report.failed_phase, Some(StatefulPhase::Observe));
    assert_eq!(
        report.phase(StatefulPhase::Observe).unwrap().outcome,
        PhaseOutcome::TimedOut
    );
    assert!(cleaned.load(Ordering::Acquire));
    assert_eq!(report.evidence.len(), 1);
    assert_eq!(report.evidence[0].kind, "before_wait");
}

fn short_limits() -> StatefulLimits {
    StatefulLimits {
        phase_timeout: Duration::from_secs(1),
        scenario_timeout: Duration::from_secs(3),
        teardown_timeout: Duration::from_secs(1),
    }
}

fn ok_action() -> rig::engine::stateful::PhaseAction {
    Box::new(|_| Ok(()))
}

fn post(url: &str) -> Result<()> {
    ureq::post(url)
        .call()
        .map_err(|error| anyhow!(error.to_string()))?;
    Ok(())
}

fn get_count(base: &str) -> Result<usize> {
    let body = ureq::get(&format!("{base}/count"))
        .call()
        .map_err(|error| anyhow!(error.to_string()))?
        .into_string()?;
    Ok(body.parse()?)
}

fn serve_fixture(
    listener: TcpListener,
    faulted: Arc<AtomicBool>,
    count: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut request = [0_u8; 2_048];
                let size = stream.read(&mut request).unwrap_or(0);
                let first_line = String::from_utf8_lossy(&request[..size])
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_owned();
                let (status, body) = if faulted.load(Ordering::Acquire) {
                    ("503 Service Unavailable", "faulted".to_owned())
                } else if first_line.starts_with("POST /append ") {
                    let value = count.fetch_add(1, Ordering::AcqRel) + 1;
                    ("200 OK", value.to_string())
                } else if first_line.starts_with("GET /count ") {
                    ("200 OK", count.load(Ordering::Acquire).to_string())
                } else {
                    ("404 Not Found", "not found".to_owned())
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => panic!("fixture accept failed: {error}"),
        }
    }
}
// HANDWRITE-END
