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

use super::http;
use super::sample::percentile;

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

/// Run the profile. Returns folded stats; per-request work happens on
/// `profile.workers` plain threads.
pub fn run(profile: &LoadProfile, vars: &VarStore) -> LoadStats {
    // Pre-interpolate once: load templates must be constant during the run.
    let request = profile.request.clone();
    if let Err(e) = vars.interpolate(&request.url) {
        return LoadStats {
            abort: Some(e),
            ..Default::default()
        };
    }

    let interval = Duration::from_secs_f64(1.0 / profile.target_qps.max(1) as f64);
    let total_ticks = (profile.target_qps as u64) * profile.duration_secs;
    let warmup_cutoff = Duration::from_secs(profile.warmup_secs);

    let (tx, rx) = mpsc::channel::<Duration>(); // tick offset from start
    let rx = Arc::new(Mutex::new(rx));
    // (offset_from_start, latency_from_scheduled_ms, ok)
    let observations: Arc<Mutex<Vec<(Duration, f64, bool)>>> =
        Arc::new(Mutex::new(Vec::with_capacity(total_ticks as usize)));

    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..profile.workers.max(1) {
        let rx = Arc::clone(&rx);
        let observations = Arc::clone(&observations);
        let request = request.clone();
        let vars = vars.clone();
        handles.push(std::thread::spawn(move || {
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
                let outcome = http::execute(&request, &vars);
                let done = Instant::now();
                let latency_ms = done.duration_since(scheduled).as_secs_f64() * 1000.0;
                let ok = matches!(&outcome, Ok(o) if o.violation.is_none());
                observations
                    .lock()
                    .expect("observations lock")
                    .push((offset, latency_ms, ok));
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
    let measured: Vec<&(Duration, f64, bool)> = observations
        .iter()
        .filter(|(offset, _, _)| *offset >= warmup_cutoff)
        .collect();
    let measured_window_secs =
        (profile.duration_secs.saturating_sub(profile.warmup_secs)).max(1) as f64;

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
