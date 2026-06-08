---
id: cap-source-config
summary: Source replay payload for projects/cap/src/config.rs
fill_sections: [source, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: full
    rationale: "The cap source group implements command wrapping, daemon leases, throttling, and structured run outcomes."
  - id: agent-hook-installation
    role: primary
    gap: claude-and-codex-hook-installation
    claim: claude-and-codex-hook-installation
    coverage: full
    rationale: "The cap source group contains the installer logic for Claude Code and Codex CLI hook registration."
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: full
    rationale: "The same source group contains hook rewrite and hook installation adapters."
  - id: command-lease-throttling
    role: primary
    gap: memory-and-cpu-pressure-sampling
    claim: memory-and-cpu-pressure-sampling
    coverage: full
    rationale: "The sampler and throttle modules in this source group implement memory and CPU pressure sampling."
  - id: daemon-lifecycle-and-status
    role: primary
    gap: daemon-process-lifecycle
    claim: daemon-process-lifecycle
    coverage: full
    rationale: "The daemon, status, wait, and ping surfaces are implemented in this source group."
  - id: daemon-lifecycle-and-status
    role: primary
    gap: cli-status-and-wait-surfaces
    claim: cli-status-and-wait-surfaces
    coverage: full
    rationale: "The CLI module in this source group implements status and wait command surfaces."
  - id: config-logging-and-reap-policy
    role: primary
    gap: configuration-defaults-and-compatibility
    claim: configuration-defaults-and-compatibility
    coverage: full
    rationale: "Configuration, JSONL logging, and reap policy modules live in this source group."
  - id: config-logging-and-reap-policy
    role: primary
    gap: run-log-persistence
    claim: run-log-persistence
    coverage: full
    rationale: "The event log module in this source group implements JSONL run-log persistence."
  - id: config-logging-and-reap-policy
    role: primary
    gap: reap-allowlist-policy
    claim: reap-allowlist-policy
    coverage: full
    rationale: "The reap module in this source group implements bounded allowlist-based process reaping."
---

# Source TD: projects/cap/src/config.rs

## Overview
<!-- type: overview lang: markdown -->

### Symbols

| Symbol | Coverage |
|---|---|
| `Config` | public Rust symbol in `projects/cap/src/config.rs` |
| `Defaults` | public Rust symbol in `projects/cap/src/config.rs` |
| `Log` | public Rust symbol in `projects/cap/src/config.rs` |
| `Protect` | public Rust symbol in `projects/cap/src/config.rs` |
| `load` | public Rust symbol in `projects/cap/src/config.rs` |
| `save` | public Rust symbol in `projects/cap/src/config.rs` |

## Source
<!-- type: source lang: rust -->

`````rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub protect: Protect,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub log: Log,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Protect {
    /// Absolute floor for available memory (GB). Safety net for small
    /// machines — `max`'d against the derived percentage floors below.
    pub min_free_gb: f64,
    /// Memory pause floor as a percentage of total RAM in use
    /// (0..=100). Translated at daemon start to
    /// `pause_floor_gb = max(min_free_gb, total * (1 - pause_used_percent/100))`.
    /// Crossing this triggers SIGSTOP-newest and Acquire backpressure.
    /// Backward-compatible alias `max_used_percent` is accepted in
    /// existing config files.
    #[serde(alias = "max_used_percent")]
    pub pause_used_percent: u32,
    /// Memory kill floor as a percentage of total RAM in use
    /// (0..=100). Translated at daemon start to
    /// `kill_floor_gb = max(min_free_gb, total * (1 - kill_used_percent/100))`.
    /// Crossing this triggers victim selection + SIGTERM/SIGKILL.
    /// Must be strictly greater than `pause_used_percent`.
    pub kill_used_percent: u32,
    /// CPU pause floor as a percentage of nproc. Triggers SIGSTOP-newest
    /// (and Acquire backpressure) when
    /// `loadavg_1m > nproc * pause_load_percent/100`. CPU has no kill
    /// threshold (SIGSTOP fully releases CPU).
    ///
    /// Default 0 = **disabled**. loadavg(1m) lags the real instantaneous
    /// load by tens of seconds, so a sub-second control loop over it
    /// over-pauses, and a healthy parallel build legitimately drives
    /// load to ~1.0/core — exactly the work cap exists to let run. Memory
    /// is the OOM signal; CPU pause is opt-in. Values may exceed 100
    /// (e.g. 150 = "pause once load passes 1.5x nproc") since load
    /// routinely runs above core count on a busy box.
    pub pause_load_percent: u32,
    /// SIGTERM-to-SIGKILL grace window. cap sends SIGTERM to the lease
    /// leader first; if still alive after this many seconds, SIGKILL the
    /// whole process group. Set 0 to skip SIGTERM and SIGKILL immediately.
    pub kill_grace_secs: u64,
    /// Last-resort eviction: if still below the kill floor for this many
    /// consecutive ticks after a victim kill, SIGKILL every paused lease.
    pub kill_all_paused_after_ticks: u32,
    /// How often (ms) to sample system memory and load.
    pub sample_interval_ms: u64,
    /// How many consecutive sub-threshold samples must pass before
    /// pausing/killing. Prevents reacting to a single noisy reading.
    pub trigger_samples: u32,
    /// When under kill-floor pressure, may cap SIGTERM known
    /// auto-restarting non-lease processes (LSPs etc., see
    /// `reap::REAP_ALLOWLIST`)? Default: true. Set false to keep cap
    /// strictly hands-off non-lease processes.
    pub reap_enabled: bool,
    /// Minimum process uptime (seconds) before a reap candidate is
    /// eligible. Prevents cascade-kill loops with IDE-restarted LSPs:
    /// editor relaunches the LSP, cap waits this long before
    /// considering it again. Default: 60.
    pub reap_min_uptime_secs: u64,
    /// How often (seconds) the reap path is allowed to fire. The
    /// process-table scan is the only expensive call cap makes; this
    /// debounce keeps a sustained low-memory state from re-scanning
    /// every tick. Default: 10.
    pub reap_cooldown_secs: u64,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    /// Priority bump applied to children (higher = lower priority).
    pub nice: i32,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Log {
    /// Write a structured record of every command that ran through cap
    /// to `~/.cap/logs/events-YYYY-MM-DD.jsonl` (one JSON object per
    /// line). Set false to disable run logging entirely.
    pub enabled: bool,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Default for Protect {
    fn default() -> Self {
        // 80% used = pause (back-pressure new submissions, SIGSTOP-newest).
        // 90% used = kill (victim eviction). Percentages auto-scale to
        // machine size; 2 GB absolute floor keeps small (≤10 GB) boxes
        // safe regardless of the percentage math.
        Self {
            min_free_gb: 2.0,
            pause_used_percent: 80,
            kill_used_percent: 90,
            // CPU pause off by default — see field doc. Memory is the
            // protection that matters; loadavg is too laggy for a
            // 500 ms loop and would throttle the agent's own builds.
            pause_load_percent: 0,
            kill_grace_secs: 3,
            kill_all_paused_after_ticks: 5,
            sample_interval_ms: 500,
            trigger_samples: 2,
            reap_enabled: true,
            reap_min_uptime_secs: 60,
            reap_cooldown_secs: 10,
        }
    }
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Default for Defaults {
    fn default() -> Self {
        Self { nice: 5 }
    }
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Default for Log {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Config {
    pub fn load() -> Result<Self> {
        let path = paths::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let cfg: Self =
            toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        paths::ensure_home()?;
        let path = paths::config_path()?;
        let text = toml::to_string_pretty(self)?;
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_safe_and_ordered() {
        let p = Protect::default();
        assert!(
            p.kill_used_percent > p.pause_used_percent,
            "kill floor must trigger AFTER pause floor (kill_used_percent > pause_used_percent)"
        );
        assert!(p.pause_used_percent <= 100 && p.kill_used_percent <= 100);
        assert!(p.min_free_gb > 0.0);
    }

    #[test]
    fn legacy_max_used_percent_alias_accepted() {
        // Existing user configs only have `max_used_percent` — the
        // serde alias must accept it as `pause_used_percent`.
        let toml = r#"
            min_free_gb = 1.5
            max_used_percent = 70
            kill_used_percent = 85
            pause_load_percent = 75
            kill_grace_secs = 5
            kill_all_paused_after_ticks = 3
            sample_interval_ms = 200
            trigger_samples = 4
        "#;
        let cfg: Config = toml::from_str(&format!("[protect]\n{toml}")).unwrap();
        assert_eq!(cfg.protect.pause_used_percent, 70);
        assert_eq!(cfg.protect.kill_used_percent, 85);
        assert_eq!(cfg.protect.min_free_gb, 1.5);
    }

    #[test]
    fn new_pause_used_percent_key_accepted() {
        let toml = r#"
            min_free_gb = 1.5
            pause_used_percent = 70
            kill_used_percent = 85
            pause_load_percent = 75
            kill_grace_secs = 5
            kill_all_paused_after_ticks = 3
            sample_interval_ms = 200
            trigger_samples = 4
        "#;
        let cfg: Config = toml::from_str(&format!("[protect]\n{toml}")).unwrap();
        assert_eq!(cfg.protect.pause_used_percent, 70);
    }

    #[test]
    fn empty_protect_section_uses_defaults() {
        let cfg: Config = toml::from_str("[protect]\n").unwrap();
        let d = Protect::default();
        assert_eq!(cfg.protect.pause_used_percent, d.pause_used_percent);
        assert_eq!(cfg.protect.kill_used_percent, d.kill_used_percent);
        assert_eq!(cfg.protect.kill_grace_secs, d.kill_grace_secs);
    }

    #[test]
    fn legacy_only_falls_back_to_default_kill_floor() {
        // Realistic upgrade scenario: existing config has only
        // `max_used_percent`; `kill_used_percent` must take its default.
        let cfg: Config = toml::from_str(
            r#"
            [protect]
            min_free_gb = 2.0
            max_used_percent = 80
            sample_interval_ms = 500
            trigger_samples = 2
        "#,
        )
        .unwrap();
        assert_eq!(cfg.protect.pause_used_percent, 80);
        assert_eq!(cfg.protect.kill_used_percent, 90); // default
    }

    #[test]
    fn round_trip_serialize_deserialize() {
        let original = Protect::default();
        let text = toml::to_string(&Config {
            protect: original.clone(),
            defaults: Defaults::default(),
            log: Log::default(),
        })
        .unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(
            parsed.protect.pause_used_percent,
            original.pause_used_percent
        );
        assert_eq!(parsed.protect.kill_used_percent, original.kill_used_percent);
        assert_eq!(parsed.protect.kill_grace_secs, original.kill_grace_secs);
        assert_eq!(
            parsed.protect.pause_load_percent,
            original.pause_load_percent
        );
        assert_eq!(
            parsed.protect.kill_all_paused_after_ticks,
            original.kill_all_paused_after_ticks
        );
    }
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/cap/src/config.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/cap/tech-design/semantic/cap-src.md#rust_source.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-cap-src-config-rs-source-replay-superseded>"
```
