---
id: projects-rig-src-engine-timeout-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/engine/timeout.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/engine/timeout.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Child-process timeout policy — ported from mamba's
//! `tests/harness/cpython/harness_common.rs` (the proven shared body of its
//! spawn loops), with `RIG_TIMEOUT_SECS` as the env override.

use std::process::{Child, Output};
use std::time::{Duration, Instant};

/// Outcome of [`wait_with_timeout`]: either the child finished on its own
/// and we collected its `Output`, or the budget elapsed and we killed it
/// (still collecting whatever output had been buffered).
pub enum WaitOutcome {
    Finished(Output),
    TimedOut(Output),
}

/// A single-source-of-truth timeout budget.
#[derive(Clone, Copy)]
pub struct TimeoutPolicy {
    timeout: Duration,
    poll_interval: Duration,
}

impl TimeoutPolicy {
    /// Read `var_name` as positive u64 seconds, falling back to
    /// `default_secs` when unset, unparseable, or `0`.
    pub fn from_env(var_name: &str, default_secs: u64) -> Self {
        let secs = std::env::var(var_name)
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(default_secs);
        Self::fixed(Duration::from_secs(secs))
    }

    /// A fixed budget with no env lookup. Poll interval defaults to 20ms.
    pub fn fixed(timeout: Duration) -> Self {
        Self {
            timeout,
            poll_interval: Duration::from_millis(20),
        }
    }

    pub fn with_poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Drive an already-spawned `child` to completion under `policy`, polling
/// with `try_wait` and killing it if the budget elapses.
pub fn wait_with_timeout(mut child: Child, policy: TimeoutPolicy) -> std::io::Result<WaitOutcome> {
    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(_status) => {
                return Ok(WaitOutcome::Finished(child.wait_with_output()?));
            }
            None if start.elapsed() > policy.timeout => {
                let _ = child.kill();
                return Ok(WaitOutcome::TimedOut(child.wait_with_output()?));
            }
            None => std::thread::sleep(policy.poll_interval),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    #[test]
    fn fast_child_finishes() {
        let child = Command::new("true").stdout(Stdio::null()).spawn().unwrap();
        match wait_with_timeout(child, TimeoutPolicy::fixed(Duration::from_secs(5))).unwrap() {
            WaitOutcome::Finished(out) => assert!(out.status.success()),
            WaitOutcome::TimedOut(_) => panic!("true(1) timed out"),
        }
    }

    #[test]
    fn slow_child_times_out_and_dies() {
        let child = Command::new("sleep")
            .arg("30")
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let started = Instant::now();
        match wait_with_timeout(child, TimeoutPolicy::fixed(Duration::from_millis(100))).unwrap() {
            WaitOutcome::TimedOut(_) => {
                assert!(started.elapsed() < Duration::from_secs(5));
            }
            WaitOutcome::Finished(_) => panic!("sleep 30 finished early"),
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/engine/timeout.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/engine/timeout.rs` captured during rig
      standardization onto the codegen ladder.
```
