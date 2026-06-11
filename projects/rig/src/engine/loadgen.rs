//! Open-loop load generator.
//!
//! The request schedule is FIXED at `target_qps` (one tick every
//! 1/target_qps), regardless of response latency: `workers` threads drain
//! the tick queue, and when they fall behind, the backlog's queueing delay
//! lands in the measured latency instead of silently throttling the
//! offered load (coordinated-omission honesty). Latency is therefore
//! measured from the SCHEDULED tick time to completion. `achieved_qps`
//! is always reported; a shortfall below the honesty ratio becomes a
//! `load_honesty` finding at the caller.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::scenario::interp::VarStore;
use crate::scenario::load::LoadProfile;

use super::sample::percentile;
use super::transport::{HttpTransport, Transport};

/// Folded result of one load run.
#[derive(Debug, Clone, Default)]
pub struct LoadStats {
    pub p50_ms: f64,
    pub p99_ms: f64,
    pub error_rate: f64,
    pub achieved_qps: f64,
    pub total: u64,
    pub failed: u64,
    /// Transport/template error that aborted the run, if any.
    pub abort: Option<String>,
}

impl LoadStats {
    pub fn get(&self, key: &str) -> Option<f64> {
        match key {
            "p50_ms" => Some(self.p50_ms),
            "p99_ms" => Some(self.p99_ms),
            "error_rate" => Some(self.error_rate),
            "achieved_qps" => Some(self.achieved_qps),
            _ => None,
        }
    }
}

/// The open-loop schedule, transport-free: offered rate, concurrency, window.
#[derive(Debug, Clone, Copy)]
pub struct Schedule {
    pub target_qps: u32,
    pub workers: u32,
    pub duration_secs: u64,
    pub warmup_secs: u64,
}

/// Run the HTTP profile. Thin wrapper over [`run_transport`] preserving the
/// original API. Per-request work happens on `profile.workers` plain threads.
pub fn run(profile: &LoadProfile, vars: &VarStore) -> LoadStats {
    // Pre-interpolate once: load templates must be constant during the run, so
    // a bad template is an abort, not a per-request failure.
    if let Err(e) = vars.interpolate(&profile.request.url) {
        return LoadStats {
            abort: Some(e),
            ..Default::default()
        };
    }
    let transport: Arc<dyn Transport> = Arc::new(HttpTransport {
        request: profile.request.clone(),
        vars: vars.clone(),
    });
    let schedule = Schedule {
        target_qps: profile.target_qps,
        workers: profile.workers,
        duration_secs: profile.duration_secs,
        warmup_secs: profile.warmup_secs,
    };
    run_transport(&schedule, &transport)
}

/// Drive `transport` under the open-loop `schedule`. The scheduler is identical
/// for every transport, so two transports (HTTP vs Postgres) measured this way
/// are comparable by construction. Returns `abort` only when every worker
/// failed to connect (e.g. the backend is down).
pub fn run_transport(schedule: &Schedule, transport: &Arc<dyn Transport>) -> LoadStats {
    let interval = Duration::from_secs_f64(1.0 / schedule.target_qps.max(1) as f64);
    let total_ticks = (schedule.target_qps as u64) * schedule.duration_secs;
    let warmup_cutoff = Duration::from_secs(schedule.warmup_secs);

    let (tx, rx) = mpsc::channel::<Duration>(); // tick offset from start
    let rx = Arc::new(Mutex::new(rx));
    // (offset_from_start, latency_from_scheduled_ms, ok)
    let observations: Arc<Mutex<Vec<(Duration, f64, bool)>>> =
        Arc::new(Mutex::new(Vec::with_capacity(total_ticks as usize)));
    let connect_err: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..schedule.workers.max(1) {
        let rx = Arc::clone(&rx);
        let observations = Arc::clone(&observations);
        let transport = Arc::clone(transport);
        let connect_err = Arc::clone(&connect_err);
        handles.push(std::thread::spawn(move || {
            // One per-worker op handle (e.g. a pg connection + prepared stmt).
            // A connect failure just retires this worker; the unbounded tick
            // channel is drained by the survivors. If ALL fail, the empty
            // observation set becomes an abort below.
            let mut worker = match transport.connect() {
                Ok(w) => w,
                Err(e) => {
                    *connect_err.lock().expect("connect_err lock") = Some(e);
                    return;
                }
            };
            loop {
                let tick = {
                    let guard = rx.lock().expect("tick channel lock");
                    guard.recv()
                };
                let Ok(offset) = tick else { break };
                // Honor the schedule: don't fire early.
                let scheduled = start + offset;
                let now = Instant::now();
                if scheduled > now {
                    std::thread::sleep(scheduled - now);
                }
                let res = worker.execute();
                let done = Instant::now();
                let latency_ms = done.duration_since(scheduled).as_secs_f64() * 1000.0;
                observations.lock().expect("observations lock").push((
                    offset,
                    latency_ms,
                    res.is_ok(),
                ));
            }
        }));
    }

    // Producer: emit the fixed schedule. Sleep in interval steps so the
    // channel backlog (not the producer) absorbs slow workers.
    for i in 0..total_ticks {
        let offset = interval * i as u32;
        let scheduled = start + offset;
        let now = Instant::now();
        if scheduled > now {
            std::thread::sleep(scheduled - now);
        }
        if tx.send(offset).is_err() {
            break;
        }
    }
    drop(tx);
    for h in handles {
        let _ = h.join();
    }

    let observations = observations.lock().expect("observations lock");
    // Nothing measured because every worker failed to connect => abort with the
    // reason (backend down / bad dsn), not a silent zero.
    if observations.is_empty() {
        if let Some(e) = connect_err.lock().expect("connect_err lock").clone() {
            return LoadStats {
                abort: Some(e),
                ..Default::default()
            };
        }
    }

    let measured: Vec<&(Duration, f64, bool)> = observations
        .iter()
        .filter(|(offset, _, _)| *offset >= warmup_cutoff)
        .collect();
    let measured_window_secs =
        (schedule.duration_secs.saturating_sub(schedule.warmup_secs)).max(1) as f64;

    let mut ok_latencies: Vec<f64> = measured
        .iter()
        .filter(|(_, _, ok)| *ok)
        .map(|(_, ms, _)| *ms)
        .collect();
    ok_latencies.sort_by(|a, b| a.partial_cmp(b).expect("finite latencies"));
    let total = measured.len() as u64;
    let failed = total - ok_latencies.len() as u64;

    LoadStats {
        p50_ms: percentile(&ok_latencies, 50.0),
        p99_ms: percentile(&ok_latencies, 99.0),
        error_rate: if total == 0 {
            1.0
        } else {
            failed as f64 / total as f64
        },
        achieved_qps: ok_latencies.len() as f64 / measured_window_secs,
        total,
        failed,
        abort: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::step::{HttpExpect, HttpRequest};
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn stub() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut s) = stream else { break };
                std::thread::spawn(move || {
                    let mut buf = [0u8; 2048];
                    let _ = s.read(&mut buf);
                    let body = r#"{"ok":true}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = s.write_all(resp.as_bytes());
                });
            }
        });
        addr
    }

    #[test]
    fn open_loop_hits_target_against_fast_stub() {
        let addr = stub();
        let profile = LoadProfile {
            target_qps: 50,
            workers: 4,
            duration_secs: 2,
            warmup_secs: 0,
            request: HttpRequest {
                method: "GET".into(),
                url: format!("http://{addr}/x"),
                body: None,
                expect: HttpExpect::default(),
            },
        };
        let stats = run(&profile, &VarStore::new());
        assert!(stats.abort.is_none());
        assert_eq!(stats.total, 100);
        assert_eq!(stats.failed, 0);
        // Fast local stub: achieved must be close to offered.
        assert!(
            stats.achieved_qps > 40.0,
            "achieved_qps = {}",
            stats.achieved_qps
        );
        assert!(stats.p99_ms > 0.0);
    }

    #[test]
    fn unreachable_target_reports_full_error_rate() {
        let dead = {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().to_string()
        };
        let profile = LoadProfile {
            target_qps: 20,
            workers: 2,
            duration_secs: 1,
            warmup_secs: 0,
            request: HttpRequest {
                method: "GET".into(),
                url: format!("http://{dead}/x"),
                body: None,
                expect: HttpExpect {
                    timeout_ms: 200,
                    ..Default::default()
                },
            },
        };
        let stats = run(&profile, &VarStore::new());
        assert_eq!(stats.error_rate, 1.0);
        assert_eq!(stats.achieved_qps, 0.0);
    }
}
