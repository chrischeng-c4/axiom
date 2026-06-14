# cap — resource-protection wrapper

`cap` keeps heavy local commands (`cargo test`, `uv run`, `pnpm build`,
…) from eating the whole machine. It is built for one job in
particular: **throttling the Bash commands a coding agent
(Claude Code, Codex CLI) fires off**, so an agent that happily launches
8 `cargo test`s at once can't OOM your box.

It is **not** an environment manager. No sandboxing, no container, no
chroot, no PATH munging. It watches the OS's idea of free memory and
pauses / resumes / kills the commands you run through it.

## Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Agent Hook Installation | - | implemented | verified | smoke | ready | `cargo test -p cap hook_install` |
| Command Lease Throttling | - | implemented | verified | smoke | ready | `cargo test -p cap throttle` |
| Daemon Lifecycle and Status | - | implemented | verified | smoke | ready | `cargo test -p cap daemon` |
| Config, Logging, and Reap Policy | - | implemented | verified | smoke | ready | `cargo test -p cap config eventlog reap` |

## AW Verification Snapshot

| Field | Value |
|---|---|
| Last verified | 2026-06-05 |
| Production readiness | ready |
| Tech design root | `projects/cap/tech-design` |
| TD lock | `projects/cap/tech-design/td.lock` |
| External-contract inventory | `projects/cap/tests/aw-ec.toml` |
| Source ownership | full codegen, 100.0% (15/15) |
| Semantic coverage | 100.0% |
| Traceability coverage | 100.0% |
| External-contract gate | passed, 4/4 |
| Test gate | `cargo test -p cap` passed |
| Health gate | `aw health cap --verify-traceability --verify-cb --verify-cold --verify-tests --verify-ec` |

## Agent Hook Installation

| Field | Value |
|---|---|
| ID | agent-hook-installation |
| Root WI | - |
| Status | verified |
| Promise | `cap init` installs fail-open PreToolUse hook snippets for Claude Code and Codex CLI, preserving unrelated user configuration while routing Bash commands through cap. |
| Required Verification | smoke |
| Gate Inventory | `cargo test -p cap hook_install`; `cargo test -p cap hook` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Claude and Codex hook installation | epic | - | implemented | verified | smoke | `cargo test -p cap hook_install` |
| Hook payload rewrite adapters | epic | - | implemented | verified | smoke | `cargo test -p cap hook` |

## Command Lease Throttling

| Field | Value |
|---|---|
| ID | command-lease-throttling |
| Root WI | - |
| Status | verified |
| Promise | `cap run` wraps local commands in daemon leases, applies memory-pressure backpressure, and emits structured outcomes when a command must wait, pause, resume, or be killed. |
| Required Verification | smoke |
| Gate Inventory | `cargo test -p cap throttle`; `cargo test -p cap sampler` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lease admission and process supervision | epic | - | implemented | verified | smoke | `cargo test -p cap throttle` |
| Memory and CPU pressure sampling | epic | - | implemented | verified | smoke | `cargo test -p cap sampler` |

## Daemon Lifecycle and Status

| Field | Value |
|---|---|
| ID | daemon-lifecycle-and-status |
| Root WI | - |
| Status | verified |
| Promise | The cap daemon can start, stop, report status, answer liveness probes, and keep command leases isolated by process group without becoming a hard dependency for agent commands. |
| Required Verification | smoke |
| Gate Inventory | `cargo test -p cap daemon`; `cargo test -p cap cli` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Daemon process lifecycle | epic | - | implemented | verified | smoke | `cargo test -p cap daemon` |
| CLI status and wait surfaces | epic | - | implemented | verified | smoke | `cargo test -p cap cli` |

## Config, Logging, and Reap Policy

| Field | Value |
|---|---|
| ID | config-logging-and-reap-policy |
| Root WI | - |
| Status | verified |
| Promise | Cap exposes durable local configuration, JSONL run logging, and a bounded reap policy for auto-restarting tool processes under kill-floor pressure. |
| Required Verification | smoke |
| Gate Inventory | `cargo test -p cap config`; `cargo test -p cap eventlog`; `cargo test -p cap reap` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Configuration defaults and compatibility | epic | - | implemented | verified | smoke | `cargo test -p cap config` |
| Run-log persistence | epic | - | implemented | verified | smoke | `cargo test -p cap eventlog` |
| Reap allowlist policy | epic | - | implemented | verified | smoke | `cargo test -p cap reap` |

## Why

You wouldn't run 8 `cargo test`s and a `uv run pytest` in parallel by
hand — but agents, cron loops, and IDE integrations happily do exactly
that, and the box dies. `cap` is the throttle in front of them.

## Quick start (the agent use case)

Build + install, then run `cap init`:

```bash
# 1. build & put `cap` on your PATH (e.g. ~/.local/bin)
projects/cap/build.sh debug

# 2. wire the PreToolUse hook into your agents (user-global)
cap init        # installs into BOTH Claude Code and Codex CLI
```

`cap init` with no arguments registers the hook into both
`~/.claude/settings.json` and `~/.codex/config.toml`. From then on every
Bash command the agent runs is transparently rewritten to:

```
/abs/path/to/cap run '<original Bash command>'
```

The hook uses cap's **absolute path** (not a bare `cap`), so it works
regardless of the agent shell's `PATH`. It does not decide whether `find`,
`grep`, pipes, or any other command should be optimized. That decision belongs
inside cap.

Cap's planner owns automatic command replacement. It preserves the familiar
command shape while selecting faster implementations only for safe subsets
that satisfy a measured resource gate. The preferred gate is dual-win: cap must
beat the original command on both CPU time and peak RSS. When a tiny command is
blocked by platform process-floor CPU cost, cap can use an RSS fallback gate
only when the RSS improvement is large enough to justify the CPU regression.
Other same-name commands may have candidate hot paths, but they are not claimed
as replacements until their benchmark wins the dual-win gate or a material
RSS-fallback gate.

Hook boundary:

| Layer | Responsibility |
|---|---|
| Agent Bash hook | Receives the Bash tool's command string and rewrites it to `cap run '<original>'`. It should stay thin: empty-command and recursion prevention only. |
| `cap run "<command string>"` | Owns command-string wrapping. It parses shell-free simple commands into argv, sends them through the command planner, and falls back to `bash -c` for pipes, redirects, globs, shell variables, `cd && ...`, and shell builtins. |
| `cap run -- <argv...>` | Manual explicit argv mode. It skips shell-string parsing and plans the exact argv the user supplied. |
| cap command planner | Owns same-name command replacement decisions and benchmark-gated fallback behavior. |

For example, the hook emits `cap run 'find . -type f -name "*.txt"'`; cap
parses that string internally and can run the same native `find` replacement as
`cap find . -type f -name "*.txt"`. For `find . -type f | xargs wc -l`, cap
detects shell syntax and wraps the original string as `bash -c` internally so
the shell keeps pipe behavior.

Active same-name replacements:

| Command | Replaced subset | Gate | Notes |
|---|---|---|---|
| `ls` | simple `ls -1` / `ls -a` / `ls -A` over one path | dual-win | High-entry-count directories. |
| `cat` | regular file arguments without flags | dual-win | Large regular-file reads; missing-file errors are parity-tested. |
| `uniq` | one input file | dual-win | Especially large adjacent-duplicate or stdout-discard cases. |
| `find` | `<root> -type f -name "*.txt"` | dual-win | Simple name/type walk only. |
| `du` | `du -sk <root>` | dual-win | Includes stdout-discard fast path; missing-root errors are parity-tested. |
| `sort` | one regular file of at least 1 MiB | dual-win | Large single-file sorting. |
| `sed` | `sed -n <start>,<end>p <file>` | dual-win | Bounded line-range reads. |
| `grep` | recursive literal `grep -R <pattern> <root>` subset | dual-win | Literal recursive search; no-match and missing-root behavior are parity-tested. |

Promotion requires both resource evidence and behavior parity. The installed
binary shape is tested against system commands for successful stdout, nonzero
exit codes, missing-path stderr diagnostics, and quiet nonzero cases such as
recursive `grep` no-match.

Pipe behavior is deliberate:

| Input shape | Current hook rewrite | Replacement behavior |
|---|---|---|
| `find ... | xargs ...` | `cap run '<original>'` | Cap wraps the whole pipeline as one command and internally uses `bash -c`. |
| `grep -R ... | head ...` | `cap run '<original>'` | The shell still owns pipe semantics and early close behavior. |
| `awk ... | xargs ...` | `cap run '<original>'` | `awk` and `xargs` stay as original PATH commands. |

The hook does not currently rewrite each pipeline segment into `cap find |
cap xargs`, no matter how many commands appear in the pipe. That avoids subtle
shell-behavior drift around quoting, file descriptors, SIGPIPE, xargs batching,
empty input, and per-segment exit status. Pipe-shaped commands are measured as
a compatibility fallback today; future promotion needs a pipe-aware planner or
fused native path with its own CPU/RSS benchmark and behavior parity tests.

Already tested but not replaced by default:

| Command / shape | Status | Reason |
|---|---|---|
| `true`, `false` | scout-only | CPU regresses heavily and RSS does not beat Apple-signed process floor. |
| `pwd`, `basename`, `dirname` | scout-only | Tiny-command CPU regression is not justified by small or absent RSS gains. |
| `head`, `tail` | scout-only | Candidate paths miss the RSS gate; `tail` also has workload-sensitive CPU behavior. |
| `mkdir`, `touch` | scout-only | Existing-path cases lose both CPU and RSS after wrapper overhead. |
| `awk`, `xargs` | scout-only | Current cap path falls through to the original command behind cap-full, so CPU/RSS both lose. |
| shell pipes such as `grep -R ... | head`, `awk ... | xargs`, `find ... | xargs` | compatibility fallback | The hook emits `cap run '<original>'`; cap internally keeps shell semantics through `bash -c`. Measured wrapper cost loses CPU/RSS, so there is no pipe replacement yet. |

Use `cap explain -- <command> ...` to see whether a command will use a native
implementation or the original command.

Command replacement is resource-benchmarked, not assumed. The benchmark compares
both public surfaces, `cap <cmd>` and hook-emitted `cap run "<command string>"`,
against the original system command with CPU time (`user + system`) and peak
RSS as the decision metrics:

```bash
cargo bench -p cap --bench command_resources
```

The latest baseline and interpretation live in `projects/cap/BENCHMARKS.md`.

Narrowing it down:

```bash
cap init claude       # just Claude Code
cap init codex        # just Codex CLI
cap init --project    # write ./.claude, ./.codex instead of user-global
cap init --print      # print the snippets, touch nothing
```

`cap init` is idempotent (re-running won't duplicate the hook) and
preserves any unrelated hooks already in the file.

### Fail-open by design

The hook wraps *every* Bash call, so cap must never become a single
point of failure. If the daemon is unreachable and can't be started,
`cap run` prints a one-line warning and **runs the command unthrottled**
rather than failing it. A broken cap degrades to a no-op; it never wedges
the agent.

## Model

```
   cap <cmd>              cap <cmd>             cap <cmd>
       │                      │                     │
       └──── Acquire ─────────┴──── Spawned ────────┘
                                  │
                                  ▼
                            cap daemon
                            (UDS RPC + sampler loop)
                                  │
               every sample_interval_ms:
                 free = OS available memory
                 free ≥ pause_floor              → resume oldest paused
                 kill_floor ≤ free < pause_floor → pause (SIGSTOP) newest
                 free < kill_floor               → kill largest victim
                                                   (SIGTERM → grace → SIGKILL)
```

* Each `cap <cmd>` registers a **lease** with the daemon, spawns the
  child in its own process group, and reports the PID. The daemon then
  owns SIGSTOP / SIGCONT / SIGTERM / SIGKILL on that group.
* Two memory floors, derived at startup from total RAM:
  * **pause floor** — below it, SIGSTOP the newest running command (stop
    it allocating more) and back-pressure new `cap run`s.
  * **kill floor** — below it, pick the largest victim by RSS and
    SIGTERM it (then SIGKILL after a grace window). After repeated ticks
    still under the kill floor, SIGKILL every paused lease as a last
    resort.
* If a `cap <cmd>` client crashes, the daemon notices the UDS disconnect
  and releases its lease automatically — no leaked budget, no orphaned
  paused process.
* When cap kills a command it returns a structured **kill envelope**
  (classification + suggested next step + sibling RSS) and prints a
  multi-line diagnostic on stderr, so the agent can tell a cap eviction
  apart from a real test failure and decide whether to wait-and-retry or
  change strategy.

## Usage

```bash
# explicit form
cap run --label "mamba build" -- cargo build -p mamba

# default form — anything after `cap` is treated as the command
cap cargo test
cap uv run pytest

# block until the box has headroom again (exit 0 = ok, 124 = timed out,
# matching GNU `timeout`); useful in `cap wait && cargo test` idioms
cap wait
cap wait --timeout 120

# daemon lifecycle (a daemon auto-starts on first `cap run`)
cap daemon start
cap daemon status
cap daemon stop

# inspect
cap status     # leases + memory/CPU headroom
cap ps         # alias of status
cap ping

# config
cap config init     # write default ~/.cap/config.toml
cap config show
```

> Note: in the default form, cap's own subcommand names shadow programs
> of the same name — `cap status` always means cap's status, never the
> system `status` binary. Use `cap run -- <cmd>` to be unambiguous. The
> agent hook always emits `cap run '<original Bash command>'`; cap then parses
> shell-free strings internally or falls back to `bash -c` when shell semantics
> are required, so hook-wrapped commands are not affected by cap subcommand
> shadowing.

## Config

`~/.cap/config.toml` (or `$CAP_HOME/config.toml`). All keys are optional
and fall back to the defaults shown:

```toml
[protect]
min_free_gb                 = 2.0   # absolute free-memory floor (small-box safety net)
pause_used_percent          = 80    # SIGSTOP-newest once RAM usage crosses this %
kill_used_percent           = 90    # evict victims once RAM usage crosses this % (> pause)
pause_load_percent          = 0     # CPU pause floor (% of nproc); 0 = off (see below)
kill_grace_secs             = 3     # SIGTERM → wait → SIGKILL; 0 = SIGKILL immediately
kill_all_paused_after_ticks = 5     # last-resort: kill all paused after N stuck ticks
sample_interval_ms          = 500   # how often to sample memory + load
trigger_samples             = 2     # consecutive sub-threshold samples before acting
reap_enabled                = true  # may SIGTERM auto-restarting LSPs under kill pressure
reap_min_uptime_secs        = 60    # min process age before it's a reap candidate
reap_cooldown_secs          = 10    # min seconds between process-table scans

[defaults]
nice = 5    # priority bump applied to children (higher = lower priority)

[log]
enabled = true    # write a per-command run log (see below); false to disable
```

The percentages auto-scale to the machine: floors are computed as
`total_RAM * (1 - used_percent/100)`, then `max`'d against `min_free_gb`
so a small box keeps its absolute reserve. `kill_used_percent` must be
strictly greater than `pause_used_percent` or the daemon refuses to
start.

**CPU pause is off by default** (`pause_load_percent = 0`). The 1-minute
load average lags real load by tens of seconds, so a sub-second control
loop over it over-pauses — and a healthy parallel build legitimately
drives load to ~1.0/core, which is exactly the work cap exists to let
run. Memory is the OOM signal that matters. If you do want CPU back-off,
set a value; it may exceed 100 (e.g. `150` = "pause once load passes
1.5× nproc").

## Run log

Every command that actually ran through cap is appended as one JSON line
to `~/.cap/logs/events-YYYY-MM-DD.jsonl` (daily file, picked at write
time so a long-lived daemon rolls over at midnight). This is the audit
of what cap did to the agent's commands — how long each waited, how long
it ran, how much memory it used, and whether cap had to kill it.

```jsonc
{
  "ts": "2026-05-29T17:52:10.539+08:00",   // finished at
  "started_at": "2026-05-29T17:52:10.355+08:00", // submitted to cap
  "lease": 2,
  "command": "cargo test -p cap",
  "program": "cargo",
  "cwd": "/Users/me/proj",
  "client_pid": 75678,
  "child_pid": 75679,
  "queue_ms": 2,            // wait between submit and start (Acquire backpressure)
  "duration_ms": 182,       // wall-clock run time
  "paused_ms": 0,           // total time cap SIGSTOPped it
  "peak_rss_gb": 1.83,      // peak leader-process RSS (see note)
  "free_gb_at_start": 4.90, // system free memory when it started
  "exit_code": 0,           // null if terminated by a signal (incl. cap SIGKILL)
  "outcome": "completed",   // or "killed"
  "kill_classification": null // "competition" | "oversize" | "external" when killed
}
```

Notes:

* Memory/free fields are sampled on the daemon's tick (`sample_interval_ms`,
  default 500 ms). Commands that finish faster than one tick show
  `peak_rss_gb: 0` / `free_gb_at_start: null` — they weren't observed.
  Heavy commands (the ones worth logging) get many samples.
* `peak_rss_gb` is the **leader** process's RSS, not the whole process
  group — for `cargo`/`pytest` the child compilers/workers aren't summed
  in, so treat it as a lower bound.
* Since the hook wraps *every* Bash call, the log includes trivial
  commands (`ls`, `cat`) too. Filter with `jq` as needed, e.g. the slow
  ones: `jq 'select(.duration_ms > 1000)' events-*.jsonl`.

Set `[log] enabled = false` to turn it off.

## Reap allowlist

Under kill-floor pressure cap may also SIGTERM a few **auto-restarting,
non-lease** processes to reclaim RAM — only the hardcoded names in
`reap::REAP_ALLOWLIST` (`rust-analyzer`, `gopls`, `clangd`, …), which the
editor transparently relaunches. The list is not user-extensible; set
`reap_enabled = false` to turn the behavior off entirely.

## Upgrading

The daemon is a long-lived process holding an exclusive lock, so an old
daemon keeps running after you replace the binary. After upgrading cap,
restart it so clients and daemon speak the same protocol:

```bash
cap daemon stop      # next `cap run` auto-spawns the new one
```

## Status / limitations

* Memory protection is the mature path (two floors, grace-period kill,
  last-resort eviction, structured kill envelopes). CPU pause is
  opt-in and coarse (see above).
* No live RSS→config promotion or per-command memory profiles yet.
* Memory floors are derived once at daemon start; changing `[protect]`
  requires a `cap daemon stop`.
</content>
</invoke>
