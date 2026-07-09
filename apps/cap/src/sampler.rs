// SPEC-MANAGED: apps/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! Thin wrapper around `sysinfo` so the rest of the crate doesn't
//! depend on it directly (makes the throttler easy to test with
//! synthetic numbers).
//!
//! Per-OS quirks:
//!
//! - **Linux**: `sysinfo::System::available_memory()` reads
//!   `/proc/meminfo`'s `MemAvailable`, which is the kernel's
//!   authoritative "how much we can allocate without going to swap".
//!   Use it directly.
//! - **macOS**: `available_memory()` is effectively `free_memory()`
//!   and reports ~0 on any active machine, because macOS aggressively
//!   parks RAM as inactive / compressed / file cache.
//!   `used_memory()` excludes those reclaimable pages, so
//!   `total - used` is a much better proxy for "memory I can hand
//!   out without paging".
//!
//! This split keeps the throttler's input meaningful on the platform
//! we actually develop on.

use std::collections::HashMap;

use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
pub struct MemorySampler {
    sys: System,
}

/// Per-process RSS lookup, scoped to a caller-provided PID list so we
/// don't pay the cost of scanning the entire process table each tick.
/// The throttler only needs RSS at kill time, but Slice 3 refreshes
/// every tick — N is typically ≤ 8 (one entry per active lease).
/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
pub struct RssSampler {
    sys: System,
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl RssSampler {
    pub fn new() -> Self {
        // Start with a bare System; we only ever poke processes.
        Self { sys: System::new() }
    }

    /// Refresh the named PIDs and return RSS in bytes for each one we
    /// could read. Dead / unknown PIDs are simply absent from the map.
    pub fn rss_bytes(&mut self, pids: &[i32]) -> HashMap<i32, u64> {
        if pids.is_empty() {
            return HashMap::new();
        }
        let pid_list: Vec<Pid> = pids
            .iter()
            .filter(|p| **p > 0)
            .map(|p| Pid::from(*p as usize))
            .collect();
        if pid_list.is_empty() {
            return HashMap::new();
        }
        // `remove_dead_processes = true` keeps the internal map from
        // growing across calls when lease PIDs come and go.
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&pid_list),
            true,
            ProcessRefreshKind::new().with_memory(),
        );
        pids.iter()
            .copied()
            .filter(|p| *p > 0)
            .filter_map(|p| {
                self.sys
                    .process(Pid::from(p as usize))
                    .map(|proc| (p, proc.memory()))
            })
            .collect()
    }
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl Default for RssSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-process CPU% lookup (usage since the last refresh), scoped to
/// caller-provided PIDs. This is the idle-timeout proxy signal: a
/// process making real forward progress accrues CPU; one blocked on
/// stdin, a socket, or a deadlock reads ~0% every tick. `sysinfo`
/// computes the since-last-refresh delta internally, so no manual
/// cumulative-time bookkeeping is needed here — unlike RSS, which is
/// an absolute snapshot, CPU% already comes as a rate.
/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
pub struct CpuSampler {
    sys: System,
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl CpuSampler {
    pub fn new() -> Self {
        Self { sys: System::new() }
    }

    /// Refresh the named PIDs and return CPU% since the last refresh
    /// for each one we could read. Dead / unknown PIDs are absent.
    pub fn cpu_usage(&mut self, pids: &[i32]) -> HashMap<i32, f32> {
        if pids.is_empty() {
            return HashMap::new();
        }
        let pid_list: Vec<Pid> = pids
            .iter()
            .filter(|p| **p > 0)
            .map(|p| Pid::from(*p as usize))
            .collect();
        if pid_list.is_empty() {
            return HashMap::new();
        }
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&pid_list),
            true,
            ProcessRefreshKind::new().with_cpu(),
        );
        pids.iter()
            .copied()
            .filter(|p| *p > 0)
            .filter_map(|p| {
                self.sys
                    .process(Pid::from(p as usize))
                    .map(|proc| (p, proc.cpu_usage()))
            })
            .collect()
    }
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl Default for CpuSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// 1-minute load average normalized by core count. > 1.0 means the
/// machine is oversubscribed; cap's CPU pause floor is a fraction of
/// that (default 0.80 = "stop submitting once load > 80% of nproc").
/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
pub struct LoadSampler {
    nproc: f64,
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl LoadSampler {
    pub fn new() -> Self {
        // available_parallelism rounds physical+SMT cores the same way
        // every other tool the user thinks in does (htop, top -1). Fall
        // back to 1.0 to keep the math defined on weird platforms.
        let nproc = std::thread::available_parallelism()
            .map(|n| n.get() as f64)
            .unwrap_or(1.0);
        Self { nproc }
    }

    pub fn load_per_core(&self) -> f64 {
        // sysinfo's load_average is a static-method-style read of the
        // OS-wide rolling averages; doesn't need a System instance.
        let load = sysinfo::System::load_average();
        if self.nproc > 0.0 {
            load.one / self.nproc
        } else {
            load.one
        }
    }
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl Default for LoadSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl MemorySampler {
    pub fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
        );
        Self { sys }
    }

    pub fn free_gb(&mut self) -> f64 {
        self.sys.refresh_memory();
        bytes_to_gb(self.available_bytes())
    }

    /// Total installed RAM (GB). Stable for the life of the process,
    /// so callers cache the value at startup.
    pub fn total_gb(&mut self) -> f64 {
        self.sys.refresh_memory();
        bytes_to_gb(self.sys.total_memory())
    }

    #[cfg(target_os = "macos")]
    fn available_bytes(&self) -> u64 {
        self.sys
            .total_memory()
            .saturating_sub(self.sys.used_memory())
    }

    #[cfg(not(target_os = "macos"))]
    fn available_bytes(&self) -> u64 {
        self.sys.available_memory()
    }
}

/// @spec apps/cap/tech-design/semantic/cap-src.md#schema
impl Default for MemorySampler {
    fn default() -> Self {
        Self::new()
    }
}

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_sampler_reports_current_pid_and_skips_unknown() {
        let mut sampler = CpuSampler::new();
        let me = std::process::id() as i32;
        // First call establishes the sysinfo baseline; give it a
        // moment before the second so a delta actually exists.
        sampler.cpu_usage(&[me]);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let usage = sampler.cpu_usage(&[me, i32::MAX]);
        assert!(usage.contains_key(&me), "own PID must be readable");
        assert!(
            !usage.contains_key(&i32::MAX),
            "unknown/dead PID must be absent, not zero"
        );
    }

    #[test]
    fn cpu_sampler_empty_pids_returns_empty_map() {
        let mut sampler = CpuSampler::new();
        assert!(sampler.cpu_usage(&[]).is_empty());
    }
}
// CODEGEN-END
