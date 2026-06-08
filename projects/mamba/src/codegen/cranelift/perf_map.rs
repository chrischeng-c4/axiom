//! JIT symbol export for native profilers (#2094).
//!
//! When `MAMBA_PERF_MAP=1` is set, the JIT backend appends one line per
//! finalized function to `/tmp/perf-<pid>.map` in the standard Linux
//! `perf` JIT-map format:
//!
//! ```text
//! <addr-hex> <size-hex> <symbol>
//! ```
//!
//! macOS profilers `samply` and (with a small extension) `cargo flamegraph`
//! consume the same file, so JIT'd Mamba functions are resolved by name
//! instead of bare hex addresses. Cranelift already writes the map when
//! `PERF_BUILDID_DIR` is set, but that is a global perf workflow knob —
//! `MAMBA_PERF_MAP` is the explicit, Mamba-scoped opt-in.
//!
//! The file is **append-only**. A long-running test process may attribute
//! stale addresses if pages get reused, but that is identical to the
//! Linux perf convention and acceptable for development profiling.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Returns the perf-map path for the current process, or `None` if the
/// `MAMBA_PERF_MAP` env var is not set to a truthy value.
///
/// Truthy: `1`, `true`, `yes`, `on` (case-insensitive). Anything else,
/// or unset, disables export.
pub fn perf_map_path() -> Option<PathBuf> {
    if !is_enabled() {
        return None;
    }
    Some(PathBuf::from(format!(
        "/tmp/perf-{}.map",
        std::process::id()
    )))
}

/// Public truthiness check on `MAMBA_PERF_MAP`. Exposed so callers can
/// skip the cost of building a symbol name when the export is off.
pub fn is_enabled() -> bool {
    match std::env::var("MAMBA_PERF_MAP") {
        Ok(v) => {
            let lower = v.to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Append a single perf-map record. Best-effort: any I/O failure is
/// silently dropped — perf-map export is a diagnostic aid and must never
/// fail the JIT pipeline.
pub fn record(addr: *const u8, size: usize, symbol: &str) {
    let Some(path) = perf_map_path() else { return };
    // Strip newlines from the symbol so we cannot corrupt the file even if
    // a future code path passes an unexpected name.
    let safe_symbol: String = symbol
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{:x} {:x} {}", addr as usize, size, safe_symbol);
    }
}

/// Test-only lock to serialize `MAMBA_PERF_MAP` env mutation across
/// modules (this one and `jit::tests::perf_map_written_when_env_set`).
/// Without it, a parallel test flipping the env could race the JIT
/// pipeline's `is_enabled()` check and the perf-map record would never
/// be written. `pub` so the JIT test module can grab the same lock.
#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: scoped env var that restores on drop, even on panic.
    struct EnvGuard {
        key: &'static str,
        prev: Option<String>,
    }
    impl EnvGuard {
        fn set(key: &'static str, val: &str) -> Self {
            let prev = std::env::var(key).ok();
            // SAFETY: tests in this module are serialized below with a Mutex
            // to make env mutation safe under Rust 2024's `unsafe`-flagged
            // `set_var` rules.
            unsafe { std::env::set_var(key, val) };
            Self { key, prev }
        }
        fn unset(key: &'static str) -> Self {
            let prev = std::env::var(key).ok();
            unsafe { std::env::remove_var(key) };
            Self { key, prev }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(v) => unsafe { std::env::set_var(self.key, v) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }

    // Serialize env-touching tests so they don't race each other.
    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        super::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn is_enabled_truthy_variants() {
        let _g = env_lock();
        for v in ["1", "true", "TRUE", "yes", "on", "On"] {
            let _e = EnvGuard::set("MAMBA_PERF_MAP", v);
            assert!(is_enabled(), "expected {v} to enable perf map");
        }
    }

    #[test]
    fn is_enabled_falsy_variants() {
        let _g = env_lock();
        for v in ["0", "false", "no", "off", "", "bogus"] {
            let _e = EnvGuard::set("MAMBA_PERF_MAP", v);
            assert!(!is_enabled(), "expected {v} to disable perf map");
        }
    }

    #[test]
    fn is_enabled_unset_is_false() {
        let _g = env_lock();
        let _e = EnvGuard::unset("MAMBA_PERF_MAP");
        assert!(!is_enabled());
    }

    #[test]
    fn record_writes_expected_line_format() {
        let _g = env_lock();
        let _e = EnvGuard::set("MAMBA_PERF_MAP", "1");
        // Start from a clean slate for this test process so we can isolate
        // our record. Use a per-test marker symbol to make the assertion
        // robust against concurrent JIT activity in other tests.
        let path = perf_map_path().expect("env set above");
        // Don't truncate — JIT tests in this binary may have already
        // written lines; assertion below looks for our marker only.
        let marker = format!(
            "_mb_perfmap_unit_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let fake_addr: *const u8 = 0xdead_beef_usize as *const u8;
        let fake_size: usize = 0x42;
        record(fake_addr, fake_size, &marker);
        let body = std::fs::read_to_string(&path).expect("read perf map");
        let expected = format!("{:x} {:x} {}", fake_addr as usize, fake_size, marker);
        assert!(
            body.lines().any(|l| l == expected),
            "expected line {expected:?} in {path:?}; got:\n{body}"
        );
    }

    #[test]
    fn record_is_noop_when_disabled() {
        let _g = env_lock();
        let _e = EnvGuard::unset("MAMBA_PERF_MAP");
        // No panic, no file required.
        record(0x1 as *const u8, 1, "should_not_appear");
        // We can't assert non-presence in a shared /tmp file robustly,
        // but the contract is: perf_map_path() returns None and record()
        // bails before any I/O.
        assert!(perf_map_path().is_none());
    }
}
