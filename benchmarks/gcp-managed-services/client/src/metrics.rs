use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct ProcessSample {
    cpu_ns: u64,
    rss_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ProcessDelta {
    pub cpu_ms: f64,
    pub rss_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct LatencySeries {
    pub samples: usize,
    pub operations_per_sample: usize,
    pub throughput_per_second: f64,
    pub p50_us: u128,
    pub p95_us: u128,
    pub p99_us: u128,
    pub durations_us: Vec<u128>,
}

impl ProcessSample {
    pub fn capture() -> Result<Self> {
        let schedstat =
            fs::read_to_string("/proc/self/schedstat").context("read /proc/self/schedstat")?;
        let cpu_ns = schedstat
            .split_whitespace()
            .next()
            .context("missing schedstat runtime")?
            .parse()
            .context("parse schedstat runtime")?;
        let status = fs::read_to_string("/proc/self/status").context("read /proc/self/status")?;
        let rss_kib = status
            .lines()
            .find_map(|line| line.strip_prefix("VmRSS:"))
            .and_then(|value| value.split_whitespace().next())
            .context("missing VmRSS")?
            .parse::<u64>()
            .context("parse VmRSS")?;
        Ok(Self {
            cpu_ns,
            rss_bytes: rss_kib.saturating_mul(1024),
        })
    }

    pub fn delta(self, after: Self) -> ProcessDelta {
        ProcessDelta {
            cpu_ms: after.cpu_ns.saturating_sub(self.cpu_ns) as f64 / 1_000_000.0,
            rss_bytes: after.rss_bytes,
        }
    }
}

impl LatencySeries {
    pub fn from_durations(operations_per_sample: usize, durations: &[Duration]) -> Self {
        let durations_us = durations
            .iter()
            .map(|duration| duration.as_micros().max(1))
            .collect::<Vec<_>>();
        let total_seconds = durations.iter().map(Duration::as_secs_f64).sum::<f64>();
        Self {
            samples: durations.len(),
            operations_per_sample,
            throughput_per_second: operations_per_sample.saturating_mul(durations.len()) as f64
                / total_seconds.max(f64::EPSILON),
            p50_us: percentile(&durations_us, 0.50),
            p95_us: percentile(&durations_us, 0.95),
            p99_us: percentile(&durations_us, 0.99),
            durations_us,
        }
    }
}

fn percentile(values: &[u128], fraction: f64) -> u128 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let index = ((sorted.len().saturating_sub(1)) as f64 * fraction).round() as usize;
    sorted.get(index).copied().unwrap_or_default()
}
