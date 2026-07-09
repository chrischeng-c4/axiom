# cap — resource-protection wrapper

## Brief

`cap` keeps heavy local commands (`cargo test`, `uv run`, `pnpm build`,
…) from eating the whole machine. It is built for one job in
particular: **throttling the Bash commands a coding agent
(Claude Code, Codex CLI) fires off**, so an agent that happily launches
8 `cargo test`s at once can't OOM your box.

It is **not** an environment manager. No sandboxing, no container, no
chroot, no PATH munging. It watches the OS's idea of free memory and
pauses / resumes / kills the commands you run through it.

## AW Verification Snapshot

Last verified: 2026-06-05
Production readiness: ready
Tech design root: `apps/cap/tech-design`
TD lock: `apps/cap/tech-design/td.lock`
External-contract inventory: `apps/cap/aw.toml`
Source ownership: full codegen, 100.0% (15/15)
Semantic coverage: 100.0%
Traceability coverage: 100.0%
External-contract gate: passed, 4/4
Test gate: `cargo test -p cap` passed
Health gate: `aw health cap --verify-traceability --verify-cb --verify-cold --verify-tests --verify-ec`


## Capabilities

Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Agent Hook Installation | - | implemented | verified | smoke | ready | `cargo test -p cap hook_install` |
| Standard Agent CLI Operations | #477 | implemented | verified | smoke | ready | `cargo test -p cap cli_std_convention`; `cargo test -p cap installed_frontend_exposes_standard_agent_commands` |
| Command Lease Throttling | - | implemented | verified | smoke | ready | `cargo test -p cap throttle` |
| Daemon Lifecycle and Status | - | implemented | verified | smoke | ready | `cargo test -p cap daemon` |
| Config, Logging, and Reap Policy | - | implemented | verified | smoke | ready | `cargo test -p cap config eventlog reap` |

### Agent Hook Installation

ID: agent-hook-installation
Type: AgentFirst
Surfaces: CLI: `cap init` + `cap hook` + `cap run '<command string>'` - Agent hook installation and hook-adapter routing that rewrites agent Bash commands through cap run.; AgentHook: `Claude Code PreToolUse` + `Codex CLI PreToolUse` - Fail-open agent hook snippets that preserve unrelated user config and route Bash commands through cap.
EC Dimensions: behavior: `cap` - Claude/Codex hook installation, command-string rewrite adapters, recursion prevention, and fail-open routing behavior
Root WI: -
Status: verified
Required Verification: smoke
Promise:
`cap init` installs fail-open PreToolUse hook snippets for Claude Code and Codex CLI, preserving unrelated user configuration while routing Bash commands through cap.
Gate Inventory:
- `cargo test -p cap hook_install`; `cargo test -p cap hook`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Claude and Codex hook installation | epic | - | implemented | verified | smoke | `cargo test -p cap hook_install` |
| Hook payload rewrite adapters | epic | - | implemented | verified | smoke | `cargo test -p cap hook` |

### Standard Agent CLI Operations

ID: standard-agent-cli-operations
Type: RuntimeTool
Surfaces: CLI: `cap llm` + `cap upgrade` + `cap issue search/view/create` + `cap report-issue` - Repo-wide agent-facing self-documentation, self-update, and diagnostics-rich issue filing through `cli-std`; `report-issue` is a deprecated compatibility alias for older issue text.
EC Dimensions: behavior: `cap` - standard CLI command registration, offline LLM docs, release upgrade routing, and project-scoped issue diagnostics
Root WI: #477
Status: verified
Required Verification: smoke
Promise:
Cap exposes the repo-wide standard agent commands through the shared `cli-std`
implementation: `llm` for offline guidance, `upgrade` for cap release updates,
and `issue` for tracker search/view/create with `app:cap` diagnostics.
Gate Inventory:
- `cargo test -p cap cli_std_convention`; `cargo test -p cap installed_frontend_exposes_standard_agent_commands`; `cargo build -p cap --features release`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Shared standard CLI commands | change | #477 | implemented | verified | smoke | `cargo test -p cap cli_std_convention`; `cargo test -p cap installed_frontend_exposes_standard_agent_commands` |

### Command Lease Throttling

ID: command-lease-throttling
Type: RuntimeTool
Surfaces: CLI: `cap run '<command string>'` + `cap run -- <argv...>` + `cap <passthrough...>` + `cap run --timeout <secs>` + `cap run --idle-timeout <secs>` + `cap wait` - Command wrapping, explicit argv mode, passthrough wrapping, per-invocation wall-clock/idle timeout overrides, and headroom wait entrypoints for agent-launched local commands.
EC Dimensions: behavior: `cap` - command wrapping, shell-string fallback, lease admission, pause/resume/kill outcomes, absolute and idle wall-clock timeouts, and structured run envelopes; efficiency: `cap` - same-name command replacement decisions and resource gates over CPU time and peak RSS; stability: `cap` - memory and CPU pressure backpressure, and independent wall-clock/idle-progress timeouts, that prevent agent-launched local commands from exhausting the host or hanging silently
Root WI: -
Status: verified
Required Verification: smoke
Promise:
`cap run` wraps local commands in daemon leases, applies memory-pressure backpressure, and emits structured outcomes when a command must wait, pause, resume, or be killed. Independently of memory/CPU pressure, `--timeout <secs>` kills a lease once its wall-clock run time (excluding time spent `Paused`) exceeds the budget, and `--idle-timeout <secs>` kills a lease that makes no CPU progress for that many seconds (also excluding `Paused` time). Both flags default to the daemon's `default_timeout_secs`/`default_idle_timeout_secs` config (0 = disabled) when omitted, and an explicit `0` disables the trigger for that invocation regardless of the config default. Both triggers reuse the existing SIGTERM-grace/SIGKILL escalation and `KillEnvelope` reporting — no new kill mechanism — surfaced as the `AbsoluteTimeout`/`IdleTimeout` classifications with `RaiseTimeoutOrSplit`/`InvestigateHang` actions.
Gate Inventory:
- `cargo test -p cap throttle`; `cargo test -p cap sampler`; `cargo test -p cap protocol`; `cargo test -p cap cli`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lease admission and process supervision | epic | - | implemented | verified | smoke | `cargo test -p cap throttle` |
| Memory and CPU pressure sampling | epic | - | implemented | verified | smoke | `cargo test -p cap sampler` |
| Absolute and idle wall-clock timeouts | epic | #1323 | implemented | verified | smoke | `cargo test -p cap throttle` |

### Daemon Lifecycle and Status

ID: daemon-lifecycle-and-status
Type: RuntimeTool
Surfaces: CLI: `cap daemon` + `cap status` + `cap ps` + `cap ping` + `cap wait` - Daemon lifecycle, lease/pressure status, liveness probe, and wait-for-headroom entrypoints.
EC Dimensions: behavior: `cap` - daemon lifecycle, lease status, liveness, process group isolation, and wait command behavior; stability: `cap` - fail-open command path, bounded wait behavior, process-group lease isolation, and daemon liveness recovery
Root WI: -
Status: verified
Required Verification: smoke
Promise:
The cap daemon can start, stop, report status, answer liveness probes, and keep command leases isolated by process group without becoming a hard dependency for agent commands.
Gate Inventory:
- `cargo test -p cap daemon`; `cargo test -p cap cli`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Daemon process lifecycle | epic | - | implemented | verified | smoke | `cargo test -p cap daemon` |
| CLI status and wait surfaces | epic | - | implemented | verified | smoke | `cargo test -p cap cli` |

### Config, Logging, and Reap Policy

ID: config-logging-and-reap-policy
Type: RuntimeTool
Surfaces: CLI: `cap config` + `cap status` - Local configuration and status surfaces for inspecting runtime pressure, leases, and cap policy state.; Config: `~/.cap/config.toml` + `JSONL run log` - Durable local configuration, run-log persistence, and bounded reap allowlist policy artifacts.
EC Dimensions: behavior: `cap` - configuration defaults/compatibility, JSONL run-log persistence, and reap allowlist policy behavior; stability: `cap` - bounded auto-reap policy and persistent logs/config that keep restart and pressure decisions auditable
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Cap exposes durable local configuration, JSONL run logging, and a bounded reap policy for auto-restarting tool processes under kill-floor pressure.
Gate Inventory:
- `cargo test -p cap config`; `cargo test -p cap eventlog`; `cargo test -p cap reap`

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
CAP_INSTALL="$HOME/.local/bin" apps/cap/build.sh debug

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

Standard agent commands:

| Command | Purpose |
|---|---|
| `cap llm [--topic <topic>] [--format md\|json]` | Offline self-documentation for agents. |
| `cap upgrade [--version <tag>] [--check]` | Self-update from `cap@*` GitHub releases through `cli-std`. |
| `cap issue search [query]` / `view <n>` / `create [--title <t>] [message...]` | Search, read, and file `app:cap` issues with build diagnostics. |
| `cap report-issue --dry-run ...` | Deprecated compatibility alias for older automation; prefer `cap issue create`. |

Cap's planner owns automatic command replacement. It preserves the familiar
command shape while selecting native implementations for safe shell-free
subsets. Tiny safe workloads are still allowed to use cap's native path: the
fixed process overhead is acceptable for interactive agent use, and keeping the
same path warm matters more than winning a micro-benchmark. Unknown, risky, or
shell-sensitive workloads still stay on the original command path. Resource
benchmarks remain a regression and capacity signal, but they no longer decide
whether a small safe same-name command is admitted to the native path.

Hook boundary:

| Layer | Responsibility |
|---|---|
| Agent Bash hook | Receives the Bash tool's command string and rewrites it to `cap run '<original>'`. It should stay thin: empty-command and recursion prevention only. |
| `cap run "<command string>"` | Owns command-string wrapping. A resident light-shell session captures the current cwd/env, attempts conservative in-process native stages for proven shell-free commands and selected fused pipelines, and dynamically falls back to `bash -c <original>` for redirects, unsupported pipes, globs, shell variables, `cd && ...`, shell builtins, and unproven command shapes. |
| `cap run -- <argv...>` | Manual explicit argv mode. It skips shell-string parsing and plans the exact argv the user supplied. |
| cap command planner | Owns same-name native dispatch decisions, behavior parity, and fallback behavior. |

For example, the hook emits `cap run 'find . -type f -name "*.txt"'`; cap
parses that string internally and can run the same native `find` replacement as
`cap find . -type f -name "*.txt"`. For an unsupported pipe such as
`find . -type f | sort | sed -n '1,10p'`, cap detects shell syntax and wraps the
original string as `bash -c` internally so the shell keeps pipe behavior. The
resident layer is an optimizer boundary, not a sandbox or replacement shell:
unsupported syntax must delegate to Bash rather than fail as command-not-found.

Session queueing is opt-in through `CAP_SESSION_ID`. When it is set, cap can
queue profiled no-observe side effects and return job metadata immediately;
the first slice only queues simple `touch <path...>` commands. Observe commands
such as `ls`, `cat`, `grep`, and `find` act as same-session barriers: they wait
for earlier queued jobs before returning their own result, and if a prior job
failed they report that job id and stderr instead of hiding the cause behind
the observe command. Without `CAP_SESSION_ID`, or for unknown/risky command
strings, `cap run` keeps the existing synchronous behavior.

Active same-name replacements are conservative native fast paths:

| Command | Replaced subset | Notes |
|---|---|---|
| `true`, `false` | any args | In-process exit primitives. |
| `pwd` | no flags | Prints cap's current working directory. |
| `echo` | plain args or `echo -n` with non-option args | Joins args with spaces; option-sensitive forms such as `-e` stay original. |
| `printf` | exact `%s` or `%s\n` format with string args; one no-conversion literal format arg with `\\`, `\n`, `\t`, or `\r` escapes | Narrow string/literal emitter; conversion formats and unsupported escapes stay original. |
| `seq` | one, two, or three integer args without flags; nonzero step | Narrow integer range generator. |
| `whoami` | no flags | Prints the effective user name. |
| `id` | no flags, or exact `-u`, `-un`, `-g`, `-gn`, `-G`, or `-Gn` | Effective uid/gid, supplementary group-list, and default identity summary lookups; unsupported flags stay original. |
| `uname` | no flags or exact `-s`, `-n`, `-r`, `-v`, `-m`, `-p`, `-a` | Narrow utsname and processor field rendering. |
| `hostname` | no args | In-process kernel hostname output; flags and hostname mutation stay original. |
| `test`, `[` | unary file/string predicates, string equality/inequality, integer comparisons, optional leading `!`; `[` requires trailing `]` in argv mode | Narrow predicate evaluator; compound expressions and shell-string `[ ... ]` stay original. |
| `basename` | one path plus optional suffix | Simple path primitive. |
| `dirname` | one path | Simple path primitive. |
| `ls` | simple `ls -1` / `ls -a` / `ls -A` over one path | Directory listing; unsupported long/options stay original. |
| `cat` | regular file arguments without flags | Streams regular files in process. |
| `head` | default lines, positive `-n <n>` or `-c <n>` over one file or stdin with no file operand | Simple line/byte windows; zero counts stay original on macOS. |
| `tail` | default lines, `-n <n>`, or `-c <n>` over one file or stdin with no file operand | Simple line/byte windows, including zero-count success. |
| `mkdir` | plain paths or `mkdir -p <path...>` | Simple directory creation. |
| `touch` | plain paths without flags | Create/update path timestamps. |
| `uniq` | one input file or stdin with no file operand | Adjacent duplicate filtering. |
| `find` | simple root plus optional `-type f|d` and `-name <glob>` | Simple name/type walks. |
| `du` | `du -sk <root>` | Includes stdout-discard fast path; missing-root errors are parity-tested. |
| `sort` | one regular file or stdin with no file operand | Buffered in-process line sorting. |
| `cut` | one regular file or stdin with no file operand, using single `-f <field>` and optional single-byte `-d <char>` delimiter | Single-field extractor; byte/char/range/list/suppress forms stay original. |
| `tr` | ASCII byte translate sets such as `a-z A-Z`, exact `[:lower:]`/`[:upper:]`/`[:digit:]` sets, or `tr -d <set>` | Streaming stdin transformer; other classes, escapes, complement, squeeze, and padded sets stay original. |
| `sed` | `sed -n <start>,<end>p <file>` | Ranged line printing. |
| `grep` | literal `grep <pattern>` over stdin, literal `grep <pattern> <file>` over one regular file, and recursive literal `grep -R <pattern> <root>` subset | Stdin/single-file grep emits matching lines without path prefixes; recursive grep preserves path-prefixed output. No-match and missing-root behavior are parity-tested. |
| `wc` | default line/word/byte counts or `wc -l/-c/-w`, over stdin or regular files | Unsupported options and non-file operands stay original. |
| `awk` | exact `/NEEDLE/ { c++ } END { print c }` plus whitespace variants of `{ print $<field> }` and `/NEEDLE/ { print $<field> }`, where `<field>` is a positive decimal field number, over one file or stdin with no file operand | Narrow counted-match and fixed-field scanners; general awk language stays original. |
| `xargs` | exact `xargs`, `xargs echo`, `xargs -n <positive> [echo]` / `xargs -n<positive> [echo]`, and `xargs wc -l` | Batches stdin tokens, emits fixed-size token batches, or batches stdin path lists in-process; other xargs options stay original. |
| `which` | one or more command names, optionally with `-a` | PATH executable lookup matching Bash `/usr/bin/which` stdout and quiet misses; `-a` scans all PATH matches. |
| `command` | exact `command -v <name...>` | Bash-style shell word and PATH lookup; other `command` forms stay original. |
| `env` | no args | Prints the inherited environment for same-name/direct command-string execution; assignment, option, and command wrapper forms stay original. |
| `printenv` | no args or one non-option name | Prints the inherited environment or a single value; flags and multi-name platform-specific forms stay original. |

Active fused pipeline replacements:

| Command string | Replacement behavior |
|---|---|
| `echo <args...> | wc -l/-c/-w` | Counts the generated echo output without a shell pipe. |
| `echo [-n] <args...> | head -n <positive>` | Emits the one supported echo line directly to the head result. |
| `echo [-n] <args...> | tail -n <nonnegative>` | Emits the supported echo line directly when the tail window is non-empty. |
| `echo [-n] <args...> | tr <set1> <set2>` | Transforms the supported echo output in-process. |
| `echo [-n] <args...> | awk '{ print $<field> }' [| ...]` | Feeds generated echo output into narrow awk fixed-field extraction, then applies supported count/head/tail/sort/xargs downstreams in-process. |
| `echo [-n] <args...> | xargs echo` | Batches generated echo tokens into the `xargs echo` line without launching a shell pipe. |
| `echo [-n] <args...> | xargs -n <positive> [echo]` | Emits generated echo tokens in fixed-size batches without launching a shell pipe. |
| `echo [-n] <paths...> | xargs wc -l` | Batches generated path tokens directly into the native `xargs wc -l` line-count path. |
| `xargs [echo] | wc -l` | Streams stdin token detection into the fused default/echo xargs output count; empty input emits zero lines. |
| `xargs -n <positive> [echo] | wc -l` | Streams stdin token counting into the fused fixed-size batch output count. |
| `xargs [echo] | grep <literal> [| ...]` | Streams stdin token detection into the default/echo xargs output, filters it by literal grep, and fuses supported count/head/tail/sort/xargs downstreams. |
| `xargs -n <positive> [echo] | grep <literal> [| ...]` | Treats each fixed-size stdin token batch as its own emitted line before supported literal-grep downstreams. |
| `grep <literal> | ...` | Reads current stdin as a literal grep producer and fuses supported count/head/tail/sort/xargs downstreams without an extra shell round trip. |
| `printf <literal-format> | ...` | For one no-conversion literal format arg using only `\\`, `\n`, `\t`, or `\r` escapes, emits the literal bytes once and fuses supported count/head/tail/sort/xargs plus literal-grep downstreams. |
| `printf '%s\n' <args...> | wc -l/-c/-w` | Counts generated printf output directly from argv. |
| `printf '%s\n' <args...> | head -n <positive>` | Emits the requested prefix of generated printf lines. |
| `printf '%s\n' <args...> | tail -n <nonnegative>` | Emits the requested suffix of generated printf lines. |
| `printf '%s\n' <args...> | awk '{ print $<field> }' [| ...]` | Feeds generated printf lines into narrow awk fixed-field extraction, then applies supported count/head/tail/sort/xargs downstreams in-process. |
| `printf '%s\n' <args...> | grep <literal>` | Filters generated printf lines by literal substring and preserves grep no-match status. |
| `printf '%s\n' <args...> | grep <literal> | wc -l/-c/-w` | Counts literal-filtered generated printf output in-process. |
| `printf '%s\n' <args...> | grep <literal> | head -n <positive>` | Emits the requested prefix of literal-filtered generated printf lines. |
| `printf '%s\n' <args...> | grep <literal> | tail -n <nonnegative>` | Emits the requested suffix of literal-filtered generated printf lines. |
| `printf '%s\n' <args...> | grep <literal> | sort` | Sorts literal-filtered generated printf lines in-process. |
| `printf '%s\n' <args...> | grep <literal> | sort | uniq` | Sorts and de-duplicates literal-filtered generated printf lines in-process. |
| `printf '%s\n' <args...> | grep <literal> | sort | uniq | wc -l` | Counts unique sorted literal-filtered generated printf lines in-process. |
| `printf '%s\n' <args...> | grep <literal> | sort | uniq | ...` | Treats the unique sorted literal-filtered printf output as a producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `printf '%s\n' <args...> | grep <literal> | sort | wc -l` | Counts sorted literal-filtered generated printf lines in-process. |
| `printf '%s\n' <args...> | grep <literal> | sort | head -n <positive>` | Emits the requested sorted literal-filtered generated-line prefix. |
| `printf '%s\n' <args...> | grep <literal> | sort | tail -n <nonnegative>` | Emits the requested sorted literal-filtered generated-line suffix. |
| `printf '%s\n' <args...> | grep <literal> | sort | xargs echo` | Sorts literal-filtered generated printf tokens before batching the `xargs echo` line. |
| `printf '%s\n' <args...> | grep <literal> | xargs echo` | Batches literal-filtered generated printf tokens into the `xargs echo` line. |
| `printf '%s\n' <args...> | tr <set1> <set2>` | Transforms generated printf lines in-process. |
| `printf '%s\n' <args...> | sort` | Sorts generated printf lines in-process. |
| `printf '%s\n' <args...> | sort | uniq` | Sorts and de-duplicates generated printf lines in-process. |
| `printf '%s\n' <args...> | sort | uniq | wc -l` | Counts unique sorted generated printf lines in-process. |
| `printf '%s\n' <args...> | sort | uniq | ...` | Treats the unique sorted printf output as a producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `printf '%s\n' <args...> | sort | wc -l` | Counts sorted generated printf lines in-process. |
| `printf '%s\n' <args...> | sort | head -n <positive>` | Emits the requested sorted generated-line prefix. |
| `printf '%s\n' <args...> | sort | tail -n <nonnegative>` | Emits the requested sorted generated-line suffix. |
| `printf '%s\n' <args...> | sort | xargs echo` | Sorts generated printf tokens before batching the `xargs echo` line. |
| `printf '%s\n' <args...> | sort | xargs -n <positive> [echo]` | Sorts generated printf lines before emitting fixed-size token batches. |
| `printf '%s\n' <paths...> | sort | xargs wc -l` | Sorts generated path tokens before the native `xargs wc -l` path. |
| `printf '%s\n' <args...> | xargs echo` | Batches generated printf tokens into the `xargs echo` line without launching a shell pipe. |
| `printf '%s\n' <args...> | xargs -n <positive> [echo]` | Emits generated printf tokens in fixed-size batches without launching a shell pipe. |
| `printf '%s\n' <paths...> | xargs wc -l` | Batches generated path tokens directly into the native `xargs wc -l` line-count path. |
| `seq <integer args...> | wc -l/-c/-w` | Counts generated integer range output directly from the range. |
| `seq <integer args...> | head -n <positive>` | Emits only the requested prefix of the integer range. |
| `seq <integer args...> | tail -n <nonnegative>` | Computes and emits only the requested suffix of the integer range. |
| `seq <integer args...> | sort` | Sorts generated integer range lines in-process. |
| `seq <integer args...> | sort | uniq` | Sorts and de-duplicates generated integer range lines in-process. |
| `seq <integer args...> | sort | uniq | wc -l` | Counts unique sorted generated integer range lines in-process. |
| `seq <integer args...> | sort | uniq | ...` | Treats the unique sorted range output as a producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `seq <integer args...> | sort | wc -l` | Counts sorted generated integer range lines in-process. |
| `seq <integer args...> | sort | head -n <positive>` | Emits the requested sorted range prefix. |
| `seq <integer args...> | sort | tail -n <nonnegative>` | Emits the requested sorted range suffix. |
| `seq <integer args...> | sort | xargs echo` | Sorts generated integer range tokens before batching the `xargs echo` line. |
| `seq <integer args...> | sort | xargs -n <positive> [echo]` | Sorts generated integer range lines before emitting fixed-size token batches. |
| `seq <integer args...> | grep <literal>` | Filters generated integer range lines by literal substring. |
| `seq <integer args...> | grep <literal> | wc -l` | Counts filtered generated range lines in-process. |
| `seq <integer args...> | grep <literal> | head -n <positive>` | Emits the requested filtered range prefix. |
| `seq <integer args...> | grep <literal> | tail -n <nonnegative>` | Emits the requested filtered range suffix. |
| `seq <integer args...> | grep <literal> | sort` | Sorts filtered generated range lines in-process. |
| `seq <integer args...> | grep <literal> | sort | uniq` | Sorts and de-duplicates filtered generated range lines in-process. |
| `seq <integer args...> | grep <literal> | sort | uniq | wc -l` | Counts unique sorted filtered generated range lines in-process. |
| `seq <integer args...> | grep <literal> | sort | uniq | ...` | Treats the unique sorted filtered range output as a producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `seq <integer args...> | grep <literal> | sort | wc -l` | Counts sorted filtered generated range lines in-process. |
| `seq <integer args...> | grep <literal> | sort | head -n <positive>` | Emits the requested sorted filtered range prefix. |
| `seq <integer args...> | grep <literal> | sort | tail -n <nonnegative>` | Emits the requested sorted filtered range suffix. |
| `seq <integer args...> | grep <literal> | sort | xargs echo` | Sorts filtered generated range tokens before batching the `xargs echo` line. |
| `seq <integer args...> | grep <literal> | xargs echo` | Batches filtered generated range tokens into the `xargs echo` line. |
| `seq <integer args...> | xargs echo` | Batches generated integer range tokens into the `xargs echo` line without launching a shell pipe. |
| `seq <integer args...> | xargs -n <positive> [echo]` | Emits generated integer range tokens in fixed-size batches without launching a shell pipe. |
| `yes [word] | head -n <positive>` | Generates only the bounded number of repeated lines; bare `yes` emits `y`. |
| `<single-line-cmd> | wc -l` | Counts the supported one-line command output without launching a shell pipe. |
| `<single-line-cmd> | head -n <positive>` | Emits the supported one-line command output directly when the head window is non-empty. |
| `<single-line-cmd> | tail -n <nonnegative>` | Emits the supported one-line command output directly when the tail window is non-empty. |
| `<single-line-cmd> | sort` | Keeps the supported one-line command output in-process through sort. |
| `<single-line-cmd> | xargs echo` | Batches the supported one-line command output into the `xargs echo` line. |
| `<single-line-cmd> | xargs wc -l` | Treats the supported one-line command output as path tokens for native `xargs wc -l`. |
| `<single-line-cmd> | grep <literal>` | Filters the supported one-line command output and preserves grep no-match status. |
| `<single-line-cmd> | grep <literal> | ...` | Supports the same downstream `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l` shapes as bounded head/tail producers. |
| `sed -n <range>p <file> | ...` | Treats ranged sed output as a bounded producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `sed -n <range>p <file> | grep <literal> | ...` | Filters ranged sed output and supports the same grep downstream count/head/tail/sort/xargs modes as bounded head/tail producers. |
| `cat <file> | sed -n <range>p` | Treats the stdin-style ranged sed form as the same in-process ranged file read. |
| `cat <file> | sed -n <range>p | ...` | Streams the file into ranged sed and fuses supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | sed -n <range>p | grep <literal> | ...` | Filters cat/sed output and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `awk '{ print $<field> }' | ...` | Reads the current stdin as a narrow awk fixed-field producer and fuses supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | awk '{ print $<field> }'` | Treats the stdin-style narrow awk fixed-field extraction as the same in-process file scan. |
| `cat <file> | awk '{ print $<field> }' | ...` | Streams the file into narrow awk fixed-field extraction and fuses supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | awk '{ print $<field> }' | grep <literal> | ...` | Streams the file into narrow awk fixed-field extraction, filters fields by literal grep, and keeps supported count/head/tail/sort/xargs downstreams in-process. |
| `cat <file> | awk '/NEEDLE/ { print $<field> }'` | Treats the stdin-style narrow awk field extraction as the same in-process file scan. |
| `cat <file> | awk '/NEEDLE/ { print $<field> }' | ...` | Streams the file into narrow awk and fuses supported count/head/tail/sort/sort|uniq/xargs downstreams. |
| `cut -d <char> -f <field> [<file>] | ...` | Treats narrow cut output from one file or stdin as a producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`; direct `wc -l` streams the record count without materializing cut output. |
| `cut -d <char> -f <field> [<file>] | grep <literal> | ...` | Filters narrow cut output from one file or stdin and supports the same grep downstream count/head/tail/sort/xargs modes as bounded head/tail producers. |
| `cat <file> | cut -d <char> -f <field> | ...` | Streams the file into narrow cut and fuses the same finite downstream count/head/tail/sort/xargs modes. |
| `cat <file> | cut -d <char> -f <field> | grep <literal> | ...` | Filters cat/cut output and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `cat <file> | tr <set1> <set2> | ...` | Streams the file through narrow tr and fuses the same finite downstream count/head/tail/sort/xargs modes. |
| `cat <file> | tr <set1> <set2> | grep <literal> | ...` | Filters cat/tr output and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `ls [-1|-a|-A] <dir> | wc -l` | Counts selected directory entries without launching a shell pipe; omitted flag and `-1` keep visible entries, `-a` includes dot entries plus `.` and `..`, and `-A` includes dot entries except `.` and `..`. |
| `ls [-1|-a|-A] <dir> | head -n <positive>` | Emits the requested selected-entry prefix from sorted `ls` output. |
| `ls [-1|-a|-A] <dir> | tail -n <nonnegative>` | Emits the requested selected-entry suffix from sorted `ls` output. |
| `ls [-1|-a|-A] <dir> | sort` | Emits sorted selected directory entries without launching a second process. |
| `ls [-1|-a|-A] <dir> | sort | uniq` | Sorts and de-duplicates selected directory entries in one process. |
| `ls [-1|-a|-A] <dir> | sort | uniq | wc -l` | Counts unique sorted selected directory entries in one process. |
| `ls [-1|-a|-A] <dir> | sort | uniq | ...` | Fuses supported downstream count/head/tail/sort/`xargs echo` modes after sorted unique selected entries. |
| `ls [-1|-a|-A] <dir> | sort | uniq | grep <literal> | ...` | Filters sorted unique selected entries and keeps supported grep downstream count/head/tail/sort/`xargs echo` modes in-process. |
| `ls [-1|-a|-A] <dir> | sort | wc -l` | Counts sorted selected directory entries without launching sort or wc. |
| `ls [-1|-a|-A] <dir> | sort | head -n <positive>` | Emits the requested sorted selected-entry prefix. |
| `ls [-1|-a|-A] <dir> | sort | tail -n <nonnegative>` | Emits the requested sorted selected-entry suffix. |
| `ls [-1|-a|-A] <dir> | sort | xargs echo` | Sorts selected directory entries and batches them into one `xargs echo` token line. |
| `ls [-1|-a|-A] <dir> | grep <literal>` | Filters sorted selected directory entries by literal substring. |
| `ls [-1|-a|-A] <dir> | grep <literal> | wc -l` | Counts sorted selected directory entries matching a literal substring. |
| `ls [-1|-a|-A] <dir> | grep <literal> | ...` | Treats literal-filtered selected entries as a finite producer for supported count/head/tail/sort/sort-uniq/`xargs echo` downstreams. |
| `ls [-1|-a|-A] <dir> | grep <literal> | xargs echo` | Filters sorted selected directory entries and batches matching tokens into the `xargs echo` line. |
| `ls [-1|-a|-A] <dir> | grep <literal> | sort | xargs echo` | Filters and sorts selected directory entries before batching matching tokens into the `xargs echo` line. |
| `ls [-1|-a|-A] <dir> | grep <literal> | xargs wc -l` | Keeps shell fallback because `ls` emits entry names relative to the caller's working directory. |
| `ls [-1|-a|-A] <dir> | xargs echo` | Batches sorted selected directory entries into the `xargs echo` token line without launching a shell pipe. |
| `sort <file> | uniq` | Sorts and de-duplicates adjacent sorted lines in one process. |
| `sort <file> | uniq | wc -l` | Sorts, de-duplicates, and counts unique sorted lines in one process. |
| `sort <file> | uniq | ...` | Treats unique sorted lines as a finite producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `sort <file> | uniq | grep <literal> | ...` | Filters unique sorted lines and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `sort <file> | grep <literal> | ...` | Filters sorted file lines and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `sort <file> | head -n <positive>` | Sorts and emits only the requested prefix. |
| `sort <file> | tail -n <nonnegative>` | Sorts and emits only the requested suffix. |
| `sort <file> | wc -l` | Counts sorted output lines without launching a second process. |
| `sort <file> | xargs echo` | Sorts file lines and batches sorted tokens into the `xargs echo` line. |
| `sort <path-list-file> | xargs wc -l` | Sorts path-list lines and counts each referenced file through the native `xargs wc -l` path. |
| `head -n <positive> [<file>] | wc -l` | Streams the bounded input prefix and counts newline-terminated output lines without launching a shell pipe. |
| `head -n <positive> [<file>] | head -n <positive>` | Streams the bounded input prefix and emits the downstream prefix directly. |
| `head -n <positive> [<file>] | tail -n <nonnegative>` | Streams the bounded input prefix while keeping only the downstream tail window. |
| `head -n <positive> [<file>] | sort` | Sorts the bounded input prefix in-process. |
| `head -n <positive> [<file>] | sort | uniq` | Sorts and de-duplicates the bounded input prefix in-process. |
| `head -n <positive> [<file>] | sort | uniq | wc -l` | Counts unique sorted lines from the bounded input prefix in-process. |
| `head -n <positive> [<file>] | sort | wc -l` | Counts sorted lines from the bounded input prefix in-process. |
| `head -n <positive> [<file>] | sort | head -n <positive>` | Emits the requested sorted prefix of the bounded input prefix. |
| `head -n <positive> [<file>] | sort | tail -n <nonnegative>` | Emits the requested sorted suffix of the bounded input prefix. |
| `head -n <positive> [<file>] | xargs echo` | Streams bounded input-prefix tokens into one native `xargs echo` line. |
| `head -n <positive> [<path-list-file>] | xargs wc -l` | Streams bounded path-list tokens into the native `xargs wc -l` path. |
| `head -n <positive> [<file>] | sort | xargs echo` | Sorts bounded input-prefix tokens before batching the `xargs echo` line. |
| `head -n <positive> [<path-list-file>] | sort | xargs wc -l` | Sorts bounded path-list tokens before the native `xargs wc -l` path. |
| `head -n <positive> [<file>] | grep <literal>` | Streams the bounded input prefix through literal grep and preserves no-match status. |
| `head -n <positive> [<file>] | grep <literal> | wc -l` | Counts literal-filtered lines from the bounded input prefix in-process. |
| `head -n <positive> [<file>] | grep <literal> | head -n <positive>` | Emits the requested literal-filtered prefix from the bounded input prefix. |
| `head -n <positive> [<file>] | grep <literal> | tail -n <nonnegative>` | Emits the requested literal-filtered suffix from the bounded input prefix. |
| `head -n <positive> [<file>] | grep <literal> | sort` | Sorts literal-filtered lines from the bounded input prefix in-process. |
| `head -n <positive> [<file>] | grep <literal> | sort | uniq` | Sorts and de-duplicates literal-filtered bounded-prefix lines in-process. |
| `head -n <positive> [<file>] | grep <literal> | sort | uniq | wc -l` | Counts unique sorted literal-filtered bounded-prefix lines in-process. |
| `head -n <positive> [<file>] | grep <literal> | sort | wc -l` | Counts sorted literal-filtered bounded-prefix lines in-process. |
| `head -n <positive> [<file>] | grep <literal> | sort | head -n <positive>` | Emits the requested sorted literal-filtered bounded-prefix prefix. |
| `head -n <positive> [<file>] | grep <literal> | sort | tail -n <nonnegative>` | Emits the requested sorted literal-filtered bounded-prefix suffix. |
| `head -n <positive> [<file>] | grep <literal> | xargs echo` | Streams literal-filtered bounded-prefix tokens into one native `xargs echo` line. |
| `head -n <positive> [<path-list-file>] | grep <literal> | xargs wc -l` | Streams literal-filtered bounded path-list tokens into the native `xargs wc -l` path. |
| `head -n <positive> [<file>] | grep <literal> | sort | xargs echo` | Sorts literal-filtered bounded-prefix tokens before batching the `xargs echo` line. |
| `head -n <positive> [<path-list-file>] | grep <literal> | sort | xargs wc -l` | Sorts literal-filtered bounded path-list tokens before the native `xargs wc -l` path. |
| `tail -n <nonnegative> [<file>] | wc -l` | Streams the bounded input suffix and counts newline-terminated output lines without launching a shell pipe. |
| `tail -n <nonnegative> [<file>] | head -n <positive>` | Streams the bounded input suffix and emits the downstream prefix directly. |
| `tail -n <nonnegative> [<file>] | tail -n <nonnegative>` | Streams the bounded input suffix while keeping only the downstream tail window. |
| `tail -n <nonnegative> [<file>] | sort` | Sorts the bounded input suffix in-process. |
| `tail -n <nonnegative> [<file>] | sort | uniq` | Sorts and de-duplicates the bounded input suffix in-process. |
| `tail -n <nonnegative> [<file>] | sort | uniq | wc -l` | Counts unique sorted lines from the bounded input suffix in-process. |
| `tail -n <nonnegative> [<file>] | sort | wc -l` | Counts sorted lines from the bounded input suffix in-process. |
| `tail -n <nonnegative> [<file>] | sort | head -n <positive>` | Emits the requested sorted prefix of the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | sort | tail -n <nonnegative>` | Emits the requested sorted suffix of the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | xargs echo` | Streams bounded input-suffix tokens into one native `xargs echo` line. |
| `tail -n <nonnegative> [<path-list-file>] | xargs wc -l` | Streams bounded path-list suffix tokens into the native `xargs wc -l` path. |
| `tail -n <nonnegative> [<file>] | sort | xargs echo` | Sorts bounded input-suffix tokens before batching the `xargs echo` line. |
| `tail -n <nonnegative> [<path-list-file>] | sort | xargs wc -l` | Sorts bounded path-list suffix tokens before the native `xargs wc -l` path. |
| `tail -n <nonnegative> [<file>] | grep <literal>` | Streams the bounded input suffix through literal grep and preserves no-match status. |
| `tail -n <nonnegative> [<file>] | grep <literal> | wc -l` | Counts literal-filtered lines from the bounded input suffix in-process. |
| `tail -n <nonnegative> [<file>] | grep <literal> | head -n <positive>` | Emits the requested literal-filtered prefix from the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | tail -n <nonnegative>` | Emits the requested literal-filtered suffix from the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort` | Sorts literal-filtered lines from the bounded input suffix in-process. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | uniq` | Sorts and de-duplicates literal-filtered bounded-suffix lines in-process. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | uniq | wc -l` | Counts unique sorted literal-filtered bounded-suffix lines in-process. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | wc -l` | Counts sorted literal-filtered bounded-suffix lines in-process. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | head -n <positive>` | Emits the requested sorted literal-filtered bounded-suffix prefix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | tail -n <nonnegative>` | Emits the requested sorted literal-filtered bounded-suffix suffix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | xargs echo` | Streams literal-filtered bounded-suffix tokens into one native `xargs echo` line. |
| `tail -n <nonnegative> [<path-list-file>] | grep <literal> | xargs wc -l` | Streams literal-filtered bounded path-list suffix tokens into the native `xargs wc -l` path. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | xargs echo` | Sorts literal-filtered bounded-suffix tokens before batching the `xargs echo` line. |
| `tail -n <nonnegative> [<path-list-file>] | grep <literal> | sort | xargs wc -l` | Sorts literal-filtered bounded path-list suffix tokens before the native `xargs wc -l` path. |
| `cat <file> | wc -l` | One in-process file line-count emits the stdin-style `wc -l` count. |
| `cat <file> | head -n <positive>` | One in-process file stream stops after the requested number of lines. |
| `cat <file> | head [-n] <positive> | ...` | Treats stdin-style bounded head output as a finite producer for supported count/head/tail/sort/sort|uniq/xargs and grep downstreams. |
| `cat <file> | tail -n <nonnegative>` | One in-process stream keeps only the requested tail line window. |
| `cat <file> | tail [-n] <nonnegative> | ...` | Treats stdin-style bounded tail output as a finite producer for supported count/head/tail/sort/sort|uniq/xargs and grep downstreams. |
| `cat <file> | grep <literal>` | One in-process scan emits matching lines and preserves grep-style no-match status. |
| `cat <file> | grep <literal> | wc -l` | One in-process scan counts matching output lines without launching a shell pipe. |
| `cat <file> | grep <literal> | head -n <positive>` | One in-process scan emits the requested matching-line prefix. |
| `cat <file> | grep <literal> | tail -n <nonnegative>` | One in-process scan emits the requested matching-line suffix. |
| `cat <file> | grep <literal> | sort` | One in-process scan sorts matching output lines. |
| `cat <file> | grep <literal> | sort | uniq` | One in-process scan emits unique sorted matching output lines. |
| `cat <file> | grep <literal> | sort | uniq | wc -l` | One in-process scan counts unique sorted matching output lines. |
| `cat <file> | grep <literal> | sort | uniq | ...` | Treats unique sorted matching output as a finite producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `cat <file> | grep <literal> | sort | wc -l` | One in-process scan counts matching sorted output lines. |
| `cat <file> | grep <literal> | sort | head -n <positive>` | One in-process scan emits the requested sorted matching-line prefix. |
| `cat <file> | grep <literal> | sort | tail -n <nonnegative>` | One in-process scan emits the requested sorted matching-line suffix. |
| `cat <file> | grep <literal> | xargs echo` | One in-process scan filters matching lines and batches tokens into the `xargs echo` line. |
| `cat <path-list-file> | grep <literal> | xargs wc -l` | One in-process scan filters path tokens into the native `xargs wc -l` path. |
| `cat <file> | grep <literal> | sort | xargs echo` | One in-process scan filters and sorts matching lines before batching tokens into the `xargs echo` line. |
| `cat <path-list-file> | grep <literal> | sort | xargs wc -l` | One in-process scan filters and sorts path tokens into the native `xargs wc -l` path. |
| `cat <file> | cut -d <char> -f <field>` | One in-process file scan extracts the selected field without launching a shell pipe. |
| `cat <file> | tr <set1> <set2>` | One in-process file stream transforms bytes without launching a shell pipe. |
| `cat <file> | tr <set1> <set2> | ...` | Streams the file through narrow tr and fuses supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | tr <set1> <set2> | grep <literal> | ...` | Filters cat/tr output and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `cat <file> | xargs echo` | One in-process file read batches whitespace tokens into the `xargs echo` line. |
| `cat <path-list-file> | xargs wc -l` | One in-process file read batches path tokens into the native `xargs wc -l` path. |
| `cat <file> | uniq` | One in-process file stream de-duplicates adjacent lines. |
| `cat <file> | uniq | wc -l` | One in-process file stream counts adjacent unique lines. |
| `cat <file> | uniq | ...` | Treats adjacent unique lines as a finite producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `cat <file> | uniq | grep <literal> | ...` | Filters adjacent unique lines and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `cat <file> | sort` | One in-process file sort avoids the redundant producer process. |
| `cat <file> | sort | uniq` | One in-process file sort de-duplicates adjacent sorted lines. |
| `cat <file> | sort | uniq | wc -l` | One in-process file sort counts unique sorted lines. |
| `cat <file> | sort | uniq | ...` | Treats unique sorted file lines as a finite producer for supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | sort | uniq | grep <literal> | ...` | Filters unique sorted file lines and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `cat <file> | sort | grep <literal> | ...` | Filters sorted file lines through the cat-sort alias and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `cat <file> | sort | wc -l` | One in-process file sort counts sorted output lines. |
| `cat <file> | sort | head -n <positive>` | One in-process file sort emits the requested sorted prefix. |
| `cat <file> | sort | tail -n <nonnegative>` | One in-process file sort emits the requested sorted suffix. |
| `cat <file> | sort | xargs echo` | One in-process file sort batches sorted tokens into the `xargs echo` line. |
| `cat <path-list-file> | sort | xargs wc -l` | One in-process file sort batches sorted path tokens into the native `xargs wc -l` path. |
| `grep <literal> | ...` | Reads stdin as a literal grep producer and keeps supported downstream count/head/tail/sort/xargs modes in-process. |
| `grep <literal> <file> | wc -l` | One in-process single-file grep stage counts matching output lines without a shell pipe. |
| `grep <literal> <file> | head -n <positive>` | One in-process single-file grep stage emits the requested matching-line prefix. |
| `grep <literal> <file> | tail -n <positive>` | One in-process single-file grep stage emits the requested matching-line suffix. |
| `grep <literal> <file> | sort` | One in-process single-file grep stage sorts matching output lines without a shell pipe. |
| `grep <literal> <file> | sort | uniq` | One in-process single-file grep stage emits unique sorted matching output lines. |
| `grep <literal> <file> | sort | uniq | wc -l` | One in-process single-file grep stage counts unique sorted matching output lines. |
| `grep <literal> <file> | sort | uniq | ...` | Treats unique sorted single-file grep output as a finite producer for supported count/head/tail/sort/xargs downstreams. |
| `grep <literal> <file> | sort | wc -l` | One in-process single-file grep stage counts matching sorted output lines without launching sort or wc. |
| `grep <literal> <file> | sort | head -n <positive>` | One in-process single-file grep stage emits the requested sorted matching-line prefix. |
| `grep <literal> <file> | sort | tail -n <positive>` | One in-process single-file grep stage emits the requested sorted matching-line suffix. |
| `grep <literal> <file> | xargs echo` | One in-process single-file grep stage batches matching tokens into the `xargs echo` line. |
| `grep <literal> <path-list-file> | xargs wc -l` | One in-process single-file grep stage filters path tokens into the native `xargs wc -l` path. |
| `grep <literal> <file> | sort | xargs echo` | One in-process single-file grep stage sorts matching tokens before batching the `xargs echo` line. |
| `grep <literal> <path-list-file> | sort | xargs wc -l` | One in-process single-file grep stage filters and sorts path tokens before the native `xargs wc -l` path. |
| `grep <literal> <file> | cut -d <char> -f <field>` | One in-process single-file grep stage applies a narrow cut field extraction without a shell pipe. |
| `grep <literal> <file> | cut -d <char> -f <field> | ...` | Treats cut single-file grep output as a finite producer for supported grep/count/head/tail/sort/xargs downstreams. |
| `grep <literal> <file> | awk '{ print $<field> }'` | One in-process single-file grep stage applies narrow awk fixed-field extraction without a shell pipe. |
| `grep <literal> <file> | awk '{ print $<field> }' | ...` | Treats awk fixed-field single-file grep output as a finite producer for supported grep/count/head/tail/sort/xargs downstreams. |
| `grep -R <pattern> <root> | head -n <positive>` | One in-process recursive grep stage stops after the requested number of matching lines. |
| `grep -R <pattern> <root> | tail -n <positive>` | One in-process recursive grep stage keeps only the requested matching-line suffix. |
| `grep -R <pattern> <root> | sort` | One in-process recursive grep stage sorts matching output lines without a shell pipe. |
| `grep -R <pattern> <root> | sort | uniq` | One in-process recursive grep stage emits unique sorted matching output lines. |
| `grep -R <pattern> <root> | sort | uniq | wc -l` | One in-process recursive grep stage counts unique sorted matching output lines. |
| `grep -R <pattern> <root> | sort | uniq | ...` | Treats unique sorted recursive grep output as a finite producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `grep -R <pattern> <root> | sort | wc -l` | One in-process recursive grep stage counts matching sorted output lines without launching sort or wc. |
| `grep -R <pattern> <root> | sort | head -n <positive>` | One in-process recursive grep stage emits the requested sorted matching-line prefix. |
| `grep -R <pattern> <root> | sort | tail -n <positive>` | One in-process recursive grep stage emits the requested sorted matching-line suffix. |
| `grep -R <pattern> <root> | wc -l` | One in-process recursive grep stage counts matching output lines without a shell pipe. |
| `awk '{ print $<field> }' <file> | ...` | Treats unfiltered fixed-field output, including action whitespace variants, as a finite producer for supported count/head/tail/sort/sort|uniq/xargs downstreams. |
| `awk '{ print $<field> }' <file> | grep <literal> | ...` | Filters narrow awk fixed-field output by literal grep and applies supported count/head/tail/sort/xargs downstreams in-process. |
| `awk '/NEEDLE/ { print $<field> }' <file> | wc -l` | One in-process scan counts emitted fixed-field lines. |
| `awk '/NEEDLE/ { print $<field> }' <file> | head -n <positive>` | One in-process scan emits the requested fixed-field prefix. |
| `awk '/NEEDLE/ { print $<field> }' <file> | tail -n <nonnegative>` | One in-process scan emits the requested fixed-field suffix. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort` | One in-process scan sorts emitted fixed-field lines. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | uniq` | One in-process scan emits unique sorted fixed-field lines. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | uniq | wc -l` | One in-process scan counts unique sorted fixed-field lines. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | uniq | ...` | Treats unique sorted fixed-field output as a finite producer for `wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l`. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | wc -l` | One in-process scan counts sorted fixed-field lines. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | head -n <positive>` | One in-process scan emits the requested sorted fixed-field prefix. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | tail -n <nonnegative>` | One in-process scan emits the requested sorted fixed-field suffix. |
| `awk '/NEEDLE/ { print $<field> }' <file> | xargs echo` | One in-process scan emits matching fixed-field tokens as the `xargs echo` line. |
| `awk '/NEEDLE/ { print $<field> }' <path-list-file> | xargs wc -l` | One in-process scan emits matching fixed-field path tokens into the native `xargs wc -l` path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | xargs echo` | One in-process scan sorts matching fixed-field tokens before batching the `xargs echo` line. |
| `awk '/NEEDLE/ { print $<field> }' <path-list-file> | sort | xargs wc -l` | One in-process scan sorts matching fixed-field path tokens before the native `xargs wc -l` path. |
| `which [-a] <name...> | wc -l` | One in-process PATH lookup counts resolved executable output lines. |
| `which [-a] <name...> | head -n <positive>` | One in-process PATH lookup emits the requested prefix of resolved executable paths. |
| `which [-a] <name...> | tail -n <nonnegative>` | One in-process PATH lookup emits the requested suffix of resolved executable paths. |
| `which [-a] <name...> | grep <literal> | ...` | Treats filtered PATH lookup output as a finite producer for supported count/head/tail/sort/xargs downstreams. |
| `which [-a] <name...> | xargs echo` | Batches resolved executable paths through native `xargs echo` without launching a shell pipe. |
| `which [-a] <name...> | sort | ...` | Sorts resolved executable paths and feeds supported count/head/tail/xargs downstreams in-process. |
| `command -v <name...> | wc -l` | One in-process Bash-style lookup counts resolved shell-word/PATH output lines. |
| `command -v <name...> | head -n <positive>` | One in-process Bash-style lookup emits the requested prefix. |
| `command -v <name...> | tail -n <nonnegative>` | One in-process Bash-style lookup emits the requested suffix. |
| `command -v <name...> | grep <literal> | ...` | Treats filtered Bash-style lookup output as a finite producer for supported count/head/tail/sort/xargs downstreams. |
| `command -v <name...> | xargs echo` | Batches resolved Bash shell words and PATH entries through native `xargs echo`. |
| `command -v <name...> | sort | ...` | Sorts resolved Bash shell words and PATH entries before supported count/head/tail/xargs downstreams. |
| `printenv <name> | wc -l` | One in-process single-value environment lookup counts emitted value lines. |
| `printenv <name> | head -n <positive>` | One in-process single-value environment lookup emits the requested prefix. |
| `printenv <name> | tail -n <nonnegative>` | One in-process single-value environment lookup emits the requested suffix. |
| `printenv <name> | grep <literal>` | One in-process single-value environment lookup filters by literal substring and preserves grep no-match status. |
| `printenv <name> | grep <literal> | ...` | Treats the filtered single environment value as a finite producer for supported count/head/tail/sort/xargs downstreams. |
| `printenv <name> | sort` | One in-process single-value environment lookup sorts the generated value line without launching a shell pipe. |
| `printenv <name> | xargs echo` | One in-process single-value environment lookup batches the value through native `xargs echo`, with missing values treated as an empty downstream input. |
| `printenv <name> | sort | xargs echo` | One in-process single-value environment lookup sorts and batches the value through native `xargs echo`. |
| `true|false | ...` | Treats zero-output primitive commands as empty finite producers for supported count/head/tail/sort/xargs downstreams while preserving last-stage pipeline exit behavior. |
| `true|false | grep <literal> | ...` | Preserves direct grep no-match status for empty primitive output and keeps supported downstream commands on the empty stream in-process. |
| `mkdir [-p] <path...> | ...` | Runs the narrow side-effecting directory creation first, then feeds its empty stdout into supported count/head/tail/sort/xargs and grep downstreams while preserving Bash last-stage pipeline exit behavior. |
| `touch <path...> | ...` | Runs the narrow timestamp/file creation first, then feeds its empty stdout into supported count/head/tail/sort/xargs and grep downstreams while preserving Bash last-stage pipeline exit behavior. |
| `test <predicate> | ...` / `[ <predicate> ] | ...` | Evaluates narrow predicates first, then feeds empty stdout into supported count/head/tail/sort/xargs and grep downstreams while preserving Bash last-stage pipeline exit behavior. |
| `wc -l/-c/-w [regular-file...] | ...` | Counts stdin when no operands are present or regular-file operands otherwise, then feeds the `wc` stdout rows into supported count/head/tail/sort/xargs and literal grep downstreams while preserving Bash last-stage pipeline exit behavior. |
| `du -sk <existing-path> | ...` | Computes the one-row disk-usage result once, then feeds it into supported count/head/tail/sort/xargs and literal grep downstreams while preserving Bash last-stage pipeline exit behavior. |
| `hostname | wc -l` | One in-process hostname lookup counts the generated hostname line. |
| `hostname | head -n <positive>` | One in-process hostname lookup emits the requested prefix. |
| `hostname | tail -n <nonnegative>` | One in-process hostname lookup emits the requested suffix. |
| `hostname | grep <literal>` | One in-process hostname lookup filters the generated line by literal substring and preserves grep no-match status. |
| `hostname | grep <literal> | ...` | Treats the filtered hostname line as a finite producer for supported count/head/tail/sort/xargs downstreams. |
| `hostname | sort` | One in-process hostname lookup keeps the single generated line without launching a shell pipe. |
| `hostname | xargs echo` | One in-process hostname lookup batches the generated hostname line through native `xargs echo`. |
| `hostname | sort | xargs echo` | One in-process hostname lookup keeps the generated line in-process through sort and native `xargs echo`. |
| `find <root> -maxdepth <positive> -type f [-name <glob>] | ...` | One bounded in-process tree walk preserves Bash maxdepth traversal and feeds supported count/head/tail/sort/sort|uniq/xargs/grep downstreams. |
| `find <root> -type f [-name <glob>] | xargs wc -l` | One in-process tree walk counts matching file lines and emits `wc -l`-style rows. |
| `find <root> -type f [-name <glob>] | xargs echo` | One in-process tree walk emits matching path tokens as the `xargs echo` line. |
| `find <root> -type f [-name <glob>] | xargs` | One in-process tree walk emits matching path tokens through default `xargs` echo semantics. |
| `find <root> -type f [-name <glob>] | grep <literal>` | One in-process tree walk filters matching paths by literal substring and preserves grep no-match exit behavior. |
| `find <root> -type f [-name <glob>] | grep <literal> | ...` | Treats filtered find paths as a finite producer for supported count/head/tail/sort/sort|uniq/xargs downstreams. |
| `find <root> -type f [-name <glob>] | grep <literal> | xargs echo` | One in-process tree walk filters matching paths by literal substring before batching the `xargs echo` line. |
| `find <root> -type f [-name <glob>] | grep <literal> | xargs wc -l` | One in-process tree walk filters matching paths by literal substring before the native `xargs wc -l` path. |
| `find <root> -type f [-name <glob>] | grep <literal> | sort | xargs echo` | One in-process tree walk filters and sorts matching paths before batching the `xargs echo` line. |
| `find <root> -type f [-name <glob>] | grep <literal> | sort | xargs wc -l` | One in-process tree walk filters and sorts matching paths before the native `xargs wc -l` path. |
| `find <root> -type f [-name <glob>] | wc -l` | One in-process tree walk counts matching result paths. |
| `find <root> -type f [-name <glob>] | head -n <positive>` | One in-process tree walk emits matching paths until the requested limit. |
| `find <root> -type f [-name <glob>] | tail -n <nonnegative>` | One in-process tree walk keeps only the requested matching path suffix. |
| `find <root> -type f [-name <glob>] | sort` | One in-process tree walk sorts matching paths without launching a shell pipe. |
| `find <root> -type f [-name <glob>] | sort | uniq` | One in-process tree walk sorts and de-duplicates matching paths. |
| `find <root> -type f [-name <glob>] | sort | uniq | wc -l` | One in-process tree walk counts unique sorted matching paths. |
| `find <root> -type f [-name <glob>] | sort | uniq | ...` | Fuses supported downstream count/head/tail/sort/`xargs echo`/`xargs wc -l` modes after sorted unique matching paths. |
| `find <root> -type f [-name <glob>] | sort | uniq | grep <literal> | ...` | Filters sorted unique matching paths and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `find <root> -type f [-name <glob>] | sort | wc -l` | One in-process tree walk sorts matching paths and counts the sorted output lines. |
| `find <root> -type f [-name <glob>] | sort | xargs echo` | One in-process tree walk sorts matching paths and emits the sorted path tokens as the `xargs echo` line. |
| `find <root> -type f [-name <glob>] | sort | xargs wc -l` | One in-process tree walk sorts matching paths and emits `wc -l`-style rows for the sorted path list. |
| `find <root> -type f [-name <glob>] | sort | head -n <positive>` | One in-process tree walk sorts matching paths and emits the requested prefix. |
| `find <root> -type f [-name <glob>] | sort | tail -n <nonnegative>` | One in-process tree walk sorts matching paths and emits the requested suffix. |

Native dispatch requires behavior parity. The installed binary shape is tested
against system commands for successful stdout, nonzero exit codes, missing-path
stderr diagnostics, and quiet nonzero cases such as recursive `grep` no-match.
Unsupported options, unsupported stdin-dependent forms, shell syntax, and risky
path shapes keep the original command path.

In the fused pipe tables, `<single-line-cmd>` currently means supported
`pwd`, `basename`, `dirname`, `whoami`, `id`, or `uname` forms that produce one
stdout line.

For fused `find ... -type f [-name <glob>]` pipelines, omitted `-name` matches
all regular files. When `-name` is present, `<glob>` is a single safe basename
pattern. Empty patterns, option-looking patterns, path separators, and bracket
classes stay on the original shell path.
The same grammar also supports `-maxdepth <positive>` immediately after the
root for bounded file scans.

Pipe behavior is deliberately shape-aware:

| Input shape | Current hook rewrite | Replacement behavior |
|---|---|---|
| `echo <args...> | wc -l` | `cap run '<original>'` | Cap counts the generated newline directly. |
| `echo [-n] <args...> | head -n <positive>` | `cap run '<original>'` | Cap emits the supported echo line directly without a shell pipe. |
| `echo [-n] <args...> | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the supported echo line directly when the tail window is non-empty. |
| `echo [-n] <args...> | tr <set1> <set2>` | `cap run '<original>'` | Cap fuses echo generation and byte translation into one native path. |
| `echo [-n] <args...> | awk '{ print $<field> }' [| ...]` | `cap run '<original>'` | Cap fuses echo generation, narrow awk fixed-field extraction, and supported count/head/tail/sort/xargs downstreams. |
| `echo [-n] <args...> | xargs echo` | `cap run '<original>'` | Cap batches generated echo tokens into one native `xargs echo` line. |
| `echo [-n] <paths...> | xargs wc -l` | `cap run '<original>'` | Cap batches generated path tokens into one native `xargs wc -l` line-count path. |
| `printf <literal-format> | ...` | `cap run '<original>'` | Cap emits one no-conversion literal format arg with `\\`, `\n`, `\t`, or `\r` escapes, then fuses supported count/head/tail/sort/xargs plus literal-grep downstreams. |
| `printf '%s\n' <args...> | wc -l` | `cap run '<original>'` | Cap counts generated printf lines directly. |
| `printf '%s\n' <args...> | head -n <positive>` | `cap run '<original>'` | Cap emits the requested generated-line prefix directly. |
| `printf '%s\n' <args...> | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested generated-line suffix directly. |
| `printf '%s\n' <args...> | awk '{ print $<field> }' [| ...]` | `cap run '<original>'` | Cap fuses printf generation, narrow awk fixed-field extraction, and supported count/head/tail/sort/xargs downstreams. |
| `printf '%s\n' <args...> | grep <literal>` | `cap run '<original>'` | Cap filters generated printf lines by literal substring in one native path. |
| `printf '%s\n' <args...> | grep <literal> | wc -l` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, and line counting into one native path. |
| `printf '%s\n' <args...> | grep <literal> | head -n <positive>` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, and prefix limiting into one native path. |
| `printf '%s\n' <args...> | grep <literal> | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, and suffix limiting into one native path. |
| `printf '%s\n' <args...> | grep <literal> | sort` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, and sorting into one native path. |
| `printf '%s\n' <args...> | grep <literal> | sort | uniq` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, sort, and unique filtering into one native path. |
| `printf '%s\n' <args...> | grep <literal> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, sort, unique filtering, and line counting into one native path. |
| `printf '%s\n' <args...> | grep <literal> | sort | wc -l` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, sort, and line counting into one native path. |
| `printf '%s\n' <args...> | grep <literal> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, sort, and prefix limiting into one native path. |
| `printf '%s\n' <args...> | grep <literal> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, sort, and suffix limiting into one native path. |
| `printf '%s\n' <args...> | grep <literal> | sort | xargs echo` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, sort, and token batching into one native path. |
| `printf '%s\n' <args...> | grep <literal> | xargs echo` | `cap run '<original>'` | Cap fuses printf generation, literal filtering, and token batching into one native path. |
| `printf '%s\n' <args...> | tr <set1> <set2>` | `cap run '<original>'` | Cap fuses printf generation and byte translation into one native path. |
| `printf '%s\n' <args...> | sort` | `cap run '<original>'` | Cap sorts generated printf lines in one native path. |
| `printf '%s\n' <args...> | sort | uniq` | `cap run '<original>'` | Cap fuses printf generation, sort, and unique filtering into one native path. |
| `printf '%s\n' <args...> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses printf generation, sort, unique filtering, and line counting into one native path. |
| `printf '%s\n' <args...> | sort | wc -l` | `cap run '<original>'` | Cap fuses printf generation, sort, and line counting into one native path. |
| `printf '%s\n' <args...> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses printf generation, sort, and prefix limiting into one native path. |
| `printf '%s\n' <args...> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses printf generation, sort, and suffix limiting into one native path. |
| `printf '%s\n' <args...> | sort | xargs echo` | `cap run '<original>'` | Cap fuses printf generation, sort, and token batching into one native path. |
| `printf '%s\n' <paths...> | sort | xargs wc -l` | `cap run '<original>'` | Cap fuses printf generation, sort, and path-token line counting into one native path. |
| `printf '%s\n' <args...> | xargs echo` | `cap run '<original>'` | Cap batches generated printf tokens into one native `xargs echo` line. |
| `printf '%s\n' <paths...> | xargs wc -l` | `cap run '<original>'` | Cap batches generated path tokens into one native `xargs wc -l` line-count path. |
| `seq <integer args...> | wc -l` | `cap run '<original>'` | Cap computes the range length directly. |
| `seq <integer args...> | head -n <positive>` | `cap run '<original>'` | Cap emits only the requested range prefix. |
| `seq <integer args...> | tail -n <nonnegative>` | `cap run '<original>'` | Cap computes and emits only the requested range suffix. |
| `seq <integer args...> | sort` | `cap run '<original>'` | Cap sorts generated range lines in one native path. |
| `seq <integer args...> | sort | uniq` | `cap run '<original>'` | Cap fuses range generation, sort, and unique filtering into one native path. |
| `seq <integer args...> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses range generation, sort, unique filtering, and line counting into one native path. |
| `seq <integer args...> | sort | wc -l` | `cap run '<original>'` | Cap fuses range generation, sort, and line counting into one native path. |
| `seq <integer args...> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses range generation, sort, and prefix limiting into one native path. |
| `seq <integer args...> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses range generation, sort, and suffix limiting into one native path. |
| `seq <integer args...> | sort | xargs echo` | `cap run '<original>'` | Cap fuses range generation, sort, and token batching into one native path. |
| `seq <integer args...> | grep <literal>` | `cap run '<original>'` | Cap filters generated range lines by literal substring in one native path. |
| `seq <integer args...> | grep <literal> | wc -l` | `cap run '<original>'` | Cap fuses range generation, literal filtering, and line counting into one native path. |
| `seq <integer args...> | grep <literal> | head -n <positive>` | `cap run '<original>'` | Cap fuses range generation, literal filtering, and prefix limiting into one native path. |
| `seq <integer args...> | grep <literal> | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses range generation, literal filtering, and suffix limiting into one native path. |
| `seq <integer args...> | grep <literal> | sort` | `cap run '<original>'` | Cap fuses range generation, literal filtering, and sort into one native path. |
| `seq <integer args...> | grep <literal> | sort | uniq` | `cap run '<original>'` | Cap fuses range generation, literal filtering, sort, and unique filtering into one native path. |
| `seq <integer args...> | grep <literal> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses range generation, literal filtering, sort, unique filtering, and line counting into one native path. |
| `seq <integer args...> | grep <literal> | sort | wc -l` | `cap run '<original>'` | Cap fuses range generation, literal filtering, sort, and line counting into one native path. |
| `seq <integer args...> | grep <literal> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses range generation, literal filtering, sort, and prefix limiting into one native path. |
| `seq <integer args...> | grep <literal> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses range generation, literal filtering, sort, and suffix limiting into one native path. |
| `seq <integer args...> | grep <literal> | sort | xargs echo` | `cap run '<original>'` | Cap fuses range generation, literal filtering, sort, and token batching into one native path. |
| `seq <integer args...> | grep <literal> | xargs echo` | `cap run '<original>'` | Cap fuses range generation, literal filtering, and token batching into one native path. |
| `seq <integer args...> | xargs echo` | `cap run '<original>'` | Cap batches generated range tokens into one native `xargs echo` line. |
| `xargs [echo] | wc -l` | `cap run '<original>'` | Cap counts whether stdin token batching would emit one default/echo xargs line without launching a second shell process. |
| `yes [word] | head -n <positive>` | `cap run '<original>'` | Cap emits the bounded repeated output directly without launching an infinite producer. |
| `<single-line-cmd> | wc/head/tail/sort/xargs echo/xargs wc -l` | `cap run '<original>'` | Cap fuses supported one-line producers such as `pwd`, `basename`, `dirname`, `whoami`, `id`, and `uname` into the downstream stage directly. |
| `<single-line-cmd> | grep <literal> [| downstream]` | `cap run '<original>'` | Cap filters supported one-line producer output in-process and then applies the supported downstream mode. |
| `ls [-1|-a|-A] <dir> | wc -l` | `cap run '<original>'` | Cap counts selected directory entries without a shell pipe; omitted flag and `-1` keep visible entries, `-a` includes dot entries plus `.` and `..`, and `-A` includes dot entries except `.` and `..`. |
| `ls [-1|-a|-A] <dir> | head -n <positive>` | `cap run '<original>'` | Cap emits the requested selected-entry prefix from sorted `ls` output. |
| `ls [-1|-a|-A] <dir> | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested selected-entry suffix from sorted `ls` output. |
| `ls [-1|-a|-A] <dir> | sort` | `cap run '<original>'` | Cap emits sorted selected directory entries without launching a second process. |
| `ls [-1|-a|-A] <dir> | sort | uniq` | `cap run '<original>'` | Cap fuses sorting and de-duplication of selected directory entries. |
| `ls [-1|-a|-A] <dir> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses sorting, de-duplication, and line counting for selected directory entries. |
| `ls [-1|-a|-A] <dir> | sort | uniq | ...` | `cap run '<original>'` | Cap fuses supported downstream count/head/tail/sort/`xargs echo` modes after sorted unique selected entries. |
| `ls [-1|-a|-A] <dir> | sort | uniq | grep <literal> | ...` | `cap run '<original>'` | Cap filters sorted unique selected entries and applies supported grep downstream modes in-process. |
| `ls [-1|-a|-A] <dir> | sort | wc -l` | `cap run '<original>'` | Cap counts sorted selected directory entries without launching sort or wc. |
| `ls [-1|-a|-A] <dir> | sort | head -n <positive>` | `cap run '<original>'` | Cap emits the requested sorted selected-entry prefix. |
| `ls [-1|-a|-A] <dir> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested sorted selected-entry suffix. |
| `ls [-1|-a|-A] <dir> | sort | xargs echo` | `cap run '<original>'` | Cap fuses sorted directory-entry output and `xargs echo` token batching into one native path. |
| `ls [-1|-a|-A] <dir> | grep <literal>` | `cap run '<original>'` | Cap filters sorted selected directory entries by literal substring. |
| `ls [-1|-a|-A] <dir> | grep <literal> | wc -l` | `cap run '<original>'` | Cap counts sorted selected directory entries matching a literal substring. |
| `ls [-1|-a|-A] <dir> | grep <literal> | ...` | `cap run '<original>'` | Cap treats literal-filtered selected entries as a finite producer for supported count, prefix, suffix, sort, unique, and token-batching downstreams. |
| `ls [-1|-a|-A] <dir> | grep <literal> | xargs echo` | `cap run '<original>'` | Cap fuses literal entry filtering and token batching into one native path. |
| `ls [-1|-a|-A] <dir> | grep <literal> | sort | xargs echo` | `cap run '<original>'` | Cap fuses literal entry filtering, sorting, and token batching into one native path. |
| `ls [-1|-a|-A] <dir> | grep <literal> | xargs wc -l` | shell fallback | Cap preserves Bash cwd-sensitive path semantics because `ls` emits entry names relative to the caller's working directory. |
| `ls [-1|-a|-A] <dir> | xargs echo` | `cap run '<original>'` | Cap batches sorted selected directory entries into the `xargs echo` token line. |
| `sort <file> | uniq` | `cap run '<original>'` | Cap sorts and de-duplicates lines without a shell pipe. |
| `sort <file> | uniq | wc -l` | `cap run '<original>'` | Cap sorts, de-duplicates, and counts unique lines without a shell pipe. |
| `sort <file> | uniq | ...` | `cap run '<original>'` | Cap treats unique sorted lines as a finite producer for count/head/tail/sort/xargs downstreams. |
| `sort <file> | uniq | grep <literal> | ...` | `cap run '<original>'` | Cap filters unique sorted lines and keeps supported grep downstreams in-process. |
| `sort <file> | head -n <positive>` | `cap run '<original>'` | Cap sorts and emits only the requested prefix. |
| `sort <file> | tail -n <nonnegative>` | `cap run '<original>'` | Cap sorts and emits only the requested suffix. |
| `sort <file> | wc -l` | `cap run '<original>'` | Cap counts sorted output lines without a shell pipe. |
| `sort <file> | xargs echo` | `cap run '<original>'` | Cap sorts file lines and batches sorted tokens into one native `xargs echo` line. |
| `sort <path-list-file> | xargs wc -l` | `cap run '<original>'` | Cap sorts path-list lines and counts each referenced file through the native `xargs wc -l` path. |
| `sort <path-list-file> | xargs wc -l | ...` | `cap run '<original>'` | Cap keeps supported downstream count/head/tail/sort modes on direct sorted path-list xargs-wc output. |
| `sed -n <range>p <file> | ...` | `cap run '<original>'` | Cap treats ranged sed output as a bounded producer for count/head/tail/sort/xargs downstreams. |
| `sed -n <range>p <file> | grep <literal> | ...` | `cap run '<original>'` | Cap filters ranged sed output and keeps supported grep downstreams in-process. |
| `cat <file> | sed -n <range>p` | `cap run '<original>'` | Cap treats stdin-style ranged sed as the same in-process ranged file read. |
| `cat <file> | sed -n <range>p | ...` | `cap run '<original>'` | Cap fuses file streaming, ranged sed, and supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | sed -n <range>p | grep <literal> | ...` | `cap run '<original>'` | Cap filters cat/sed output and keeps supported grep downstreams in-process. |
| `awk '{ print $<field> }' | ...` | `cap run '<original>'` | Cap reads the current stdin through the narrow awk fixed-field producer and applies supported count/head/tail/sort/xargs downstreams in-process. |
| `cat <file> | awk '{ print $<field> }'` | `cap run '<original>'` | Cap treats stdin-style narrow awk fixed-field extraction as the same in-process file scan. |
| `cat <file> | awk '{ print $<field> }' | ...` | `cap run '<original>'` | Cap fuses file streaming, narrow awk fixed-field extraction, and supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | awk '{ print $<field> }' | grep <literal> | ...` | `cap run '<original>'` | Cap fuses file streaming, narrow awk fixed-field extraction, literal grep filtering, and supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | awk '/NEEDLE/ { print $<field> }'` | `cap run '<original>'` | Cap treats stdin-style narrow awk field extraction as the same in-process file scan. |
| `cat <file> | awk '/NEEDLE/ { print $<field> }' | ...` | `cap run '<original>'` | Cap fuses file streaming, narrow awk field extraction, and supported count/head/tail/sort/xargs downstreams. |
| `cut -d <char> -f <field> [<file>] | ...` | `cap run '<original>'` | Cap treats narrow cut output from one file or stdin as a producer for count/head/tail/sort/xargs downstreams; direct `wc -l` streams the record count. |
| `cut -d <char> -f <field> [<file>] | grep <literal> | ...` | `cap run '<original>'` | Cap filters narrow cut output from one file or stdin and keeps supported grep downstreams in-process. |
| `cat <file> | cut -d <char> -f <field> | ...` | `cap run '<original>'` | Cap fuses file streaming, field extraction, and supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | cut -d <char> -f <field> | grep <literal> | ...` | `cap run '<original>'` | Cap filters cat/cut output and keeps supported grep downstreams in-process. |
| `cat <file> | tr <set1> <set2> | ...` | `cap run '<original>'` | Cap fuses file streaming, byte translation, and supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | tr <set1> <set2> | grep <literal> | ...` | `cap run '<original>'` | Cap filters cat/tr output and keeps supported grep downstreams in-process. |
| `head -n <positive> [<file>] | wc -l` | `cap run '<original>'` | Cap streams the bounded input prefix and counts newline-terminated output lines. |
| `head -n <positive> [<file>] | head -n <positive>` | `cap run '<original>'` | Cap streams the bounded input prefix and emits the downstream prefix directly. |
| `head -n <positive> [<file>] | tail -n <nonnegative>` | `cap run '<original>'` | Cap streams the bounded input prefix and keeps only the downstream tail window. |
| `head -n <positive> [<file>] | sort` | `cap run '<original>'` | Cap sorts the bounded input prefix without launching a shell pipe. |
| `head -n <positive> [<file>] | sort | uniq` | `cap run '<original>'` | Cap sorts and de-duplicates the bounded input prefix in one native path. |
| `head -n <positive> [<file>] | sort | uniq | wc -l` | `cap run '<original>'` | Cap counts unique sorted lines from the bounded input prefix. |
| `head -n <positive> [<file>] | sort | wc -l` | `cap run '<original>'` | Cap counts sorted lines from the bounded input prefix. |
| `head -n <positive> [<file>] | sort | head -n <positive>` | `cap run '<original>'` | Cap emits the requested sorted prefix of the bounded input prefix. |
| `head -n <positive> [<file>] | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested sorted suffix of the bounded input prefix. |
| `head -n <positive> [<file>] | xargs echo` | `cap run '<original>'` | Cap streams bounded input-prefix tokens into one native `xargs echo` line. |
| `head -n <positive> [<path-list-file>] | xargs wc -l` | `cap run '<original>'` | Cap streams bounded path-list tokens into the native `xargs wc -l` path. |
| `head -n <positive> [<file>] | sort | xargs echo` | `cap run '<original>'` | Cap sorts bounded input-prefix tokens before batching the `xargs echo` line. |
| `head -n <positive> [<path-list-file>] | sort | xargs wc -l` | `cap run '<original>'` | Cap sorts bounded path-list tokens before the native `xargs wc -l` path. |
| `head -n <positive> [<file>] | grep <literal>` | `cap run '<original>'` | Cap streams the bounded input prefix through literal grep and preserves no-match status. |
| `head -n <positive> [<file>] | grep <literal> | wc -l` | `cap run '<original>'` | Cap counts literal-filtered lines from the bounded input prefix. |
| `head -n <positive> [<file>] | grep <literal> | head -n <positive>` | `cap run '<original>'` | Cap emits the requested literal-filtered prefix from the bounded input prefix. |
| `head -n <positive> [<file>] | grep <literal> | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested literal-filtered suffix from the bounded input prefix. |
| `head -n <positive> [<file>] | grep <literal> | sort` | `cap run '<original>'` | Cap sorts literal-filtered lines from the bounded input prefix. |
| `head -n <positive> [<file>] | grep <literal> | sort | uniq` | `cap run '<original>'` | Cap sorts and de-duplicates literal-filtered bounded-prefix lines. |
| `head -n <positive> [<file>] | grep <literal> | sort | uniq | wc -l` | `cap run '<original>'` | Cap counts unique sorted literal-filtered bounded-prefix lines. |
| `head -n <positive> [<file>] | grep <literal> | sort | wc -l` | `cap run '<original>'` | Cap counts sorted literal-filtered bounded-prefix lines. |
| `head -n <positive> [<file>] | grep <literal> | sort | head -n <positive>` | `cap run '<original>'` | Cap emits the requested sorted literal-filtered bounded-prefix prefix. |
| `head -n <positive> [<file>] | grep <literal> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested sorted literal-filtered bounded-prefix suffix. |
| `head -n <positive> [<file>] | grep <literal> | xargs echo` | `cap run '<original>'` | Cap streams literal-filtered bounded-prefix tokens into one native `xargs echo` line. |
| `head -n <positive> [<path-list-file>] | grep <literal> | xargs wc -l` | `cap run '<original>'` | Cap streams literal-filtered bounded path-list tokens into the native `xargs wc -l` path. |
| `head -n <positive> [<file>] | grep <literal> | sort | xargs echo` | `cap run '<original>'` | Cap sorts literal-filtered bounded-prefix tokens before batching the `xargs echo` line. |
| `head -n <positive> [<path-list-file>] | grep <literal> | sort | xargs wc -l` | `cap run '<original>'` | Cap sorts literal-filtered bounded path-list tokens before the native `xargs wc -l` path. |
| `tail -n <nonnegative> [<file>] | wc -l` | `cap run '<original>'` | Cap streams the bounded input suffix and counts newline-terminated output lines. |
| `tail -n <nonnegative> [<file>] | head -n <positive>` | `cap run '<original>'` | Cap streams the bounded input suffix and emits the downstream prefix directly. |
| `tail -n <nonnegative> [<file>] | tail -n <nonnegative>` | `cap run '<original>'` | Cap streams the bounded input suffix and keeps only the downstream tail window. |
| `tail -n <nonnegative> [<file>] | sort` | `cap run '<original>'` | Cap sorts the bounded input suffix without launching a shell pipe. |
| `tail -n <nonnegative> [<file>] | sort | uniq` | `cap run '<original>'` | Cap sorts and de-duplicates the bounded input suffix in one native path. |
| `tail -n <nonnegative> [<file>] | sort | uniq | wc -l` | `cap run '<original>'` | Cap counts unique sorted lines from the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | sort | wc -l` | `cap run '<original>'` | Cap counts sorted lines from the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | sort | head -n <positive>` | `cap run '<original>'` | Cap emits the requested sorted prefix of the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested sorted suffix of the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | xargs echo` | `cap run '<original>'` | Cap streams bounded input-suffix tokens into one native `xargs echo` line. |
| `tail -n <nonnegative> [<path-list-file>] | xargs wc -l` | `cap run '<original>'` | Cap streams bounded path-list suffix tokens into the native `xargs wc -l` path. |
| `tail -n <nonnegative> [<file>] | sort | xargs echo` | `cap run '<original>'` | Cap sorts bounded input-suffix tokens before batching the `xargs echo` line. |
| `tail -n <nonnegative> [<path-list-file>] | sort | xargs wc -l` | `cap run '<original>'` | Cap sorts bounded path-list suffix tokens before the native `xargs wc -l` path. |
| `tail -n <nonnegative> [<file>] | grep <literal>` | `cap run '<original>'` | Cap streams the bounded input suffix through literal grep and preserves no-match status. |
| `tail -n <nonnegative> [<file>] | grep <literal> | wc -l` | `cap run '<original>'` | Cap counts literal-filtered lines from the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | head -n <positive>` | `cap run '<original>'` | Cap emits the requested literal-filtered prefix from the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested literal-filtered suffix from the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort` | `cap run '<original>'` | Cap sorts literal-filtered lines from the bounded input suffix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | uniq` | `cap run '<original>'` | Cap sorts and de-duplicates literal-filtered bounded-suffix lines. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | uniq | wc -l` | `cap run '<original>'` | Cap counts unique sorted literal-filtered bounded-suffix lines. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | wc -l` | `cap run '<original>'` | Cap counts sorted literal-filtered bounded-suffix lines. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | head -n <positive>` | `cap run '<original>'` | Cap emits the requested sorted literal-filtered bounded-suffix prefix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap emits the requested sorted literal-filtered bounded-suffix suffix. |
| `tail -n <nonnegative> [<file>] | grep <literal> | xargs echo` | `cap run '<original>'` | Cap streams literal-filtered bounded-suffix tokens into one native `xargs echo` line. |
| `tail -n <nonnegative> [<path-list-file>] | grep <literal> | xargs wc -l` | `cap run '<original>'` | Cap streams literal-filtered bounded path-list suffix tokens into the native `xargs wc -l` path. |
| `tail -n <nonnegative> [<file>] | grep <literal> | sort | xargs echo` | `cap run '<original>'` | Cap sorts literal-filtered bounded-suffix tokens before batching the `xargs echo` line. |
| `tail -n <nonnegative> [<path-list-file>] | grep <literal> | sort | xargs wc -l` | `cap run '<original>'` | Cap sorts literal-filtered bounded path-list suffix tokens before the native `xargs wc -l` path. |
| `cat <file> | wc -l` | `cap run '<original>'` | Cap fuses file streaming and line counting into one native path. |
| `cat <file> | head -n <positive>` | `cap run '<original>'` | Cap fuses file streaming and the head limit so reads stop in-process. |
| `cat <file> | head [-n] <positive> | ...` | `cap run '<original>'` | Cap treats stdin-style bounded head output as a finite producer for supported count/head/tail/sort/xargs and grep downstreams. |
| `cat <file> | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses file streaming and keeps only a tail ring buffer. |
| `cat <file> | tail [-n] <nonnegative> | ...` | `cap run '<original>'` | Cap treats stdin-style bounded tail output as a finite producer for supported count/head/tail/sort/xargs and grep downstreams. |
| `cat <file> | grep <literal>` | `cap run '<original>'` | Cap fuses file streaming and literal line matching into one native path. |
| `cat <file> | grep <literal> | wc -l` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, and line counting into one native path. |
| `cat <file> | grep <literal> | head -n <positive>` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, and prefix limiting into one native path. |
| `cat <file> | grep <literal> | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, and suffix limiting into one native path. |
| `cat <file> | grep <literal> | sort` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, and sorting into one native path. |
| `cat <file> | grep <literal> | sort | uniq` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, sorting, and de-duplication into one native path. |
| `cat <file> | grep <literal> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, sorting, de-duplication, and line counting into one native path. |
| `cat <file> | grep <literal> | sort | uniq | ...` | `cap run '<original>'` | Cap treats unique sorted literal-matching lines as a finite producer for supported downstream count/head/tail/sort/`xargs echo`/`xargs wc -l` modes. |
| `cat <file> | grep <literal> | sort | wc -l` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, sorting, and line counting into one native path. |
| `cat <file> | grep <literal> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, sorting, and prefix limiting into one native path. |
| `cat <file> | grep <literal> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, sorting, and suffix limiting into one native path. |
| `cat <file> | grep <literal> | xargs echo` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, and token batching into one native path. |
| `cat <path-list-file> | grep <literal> | xargs wc -l` | `cap run '<original>'` | Cap fuses file streaming, literal path-token filtering, and line counting into one native path. |
| `cat <file> | grep <literal> | sort | xargs echo` | `cap run '<original>'` | Cap fuses file streaming, literal line matching, sorting, and token batching into one native path. |
| `cat <path-list-file> | grep <literal> | sort | xargs wc -l` | `cap run '<original>'` | Cap fuses file streaming, literal path-token filtering, sorting, and line counting into one native path. |
| `cat <file> | cut -d <char> -f <field>` | `cap run '<original>'` | Cap fuses file streaming and single-field extraction into one native path. |
| `cat <file> | tr <set1> <set2>` | `cap run '<original>'` | Cap fuses file streaming and byte translation into one native path. |
| `cat <file> | tr <set1> <set2> | ...` | `cap run '<original>'` | Cap fuses file streaming, byte translation, and supported count/head/tail/sort/xargs downstreams. |
| `cat <file> | tr <set1> <set2> | grep <literal> | ...` | `cap run '<original>'` | Cap filters cat/tr output and keeps supported grep downstreams in-process. |
| `cat <file> | xargs echo` | `cap run '<original>'` | Cap batches file whitespace tokens into one native `xargs echo` line. |
| `cat <path-list-file> | xargs wc -l` | `cap run '<original>'` | Cap batches file path tokens into one native `xargs wc -l` line-count path. |
| `cat <path-list-file> | xargs wc -l | ...` | `cap run '<original>'` | Cap treats direct path-list xargs-wc output as a finite producer for supported count/head/tail/sort downstreams. |
| `cat <file> | uniq` | `cap run '<original>'` | Cap fuses file streaming and adjacent de-duplication into one native path. |
| `cat <file> | uniq | wc -l` | `cap run '<original>'` | Cap fuses file streaming, adjacent de-duplication, and line counting into one native path. |
| `cat <file> | uniq | ...` | `cap run '<original>'` | Cap treats adjacent unique lines as a finite producer for count/head/tail/sort/xargs downstreams. |
| `cat <file> | uniq | grep <literal> | ...` | `cap run '<original>'` | Cap filters adjacent unique lines and keeps supported grep downstreams in-process. |
| `uniq <file> | ...` | `cap run '<original>'` | Cap treats adjacent unique file lines as a finite producer for supported count/head/tail/sort/xargs downstreams. |
| `uniq <file> | grep <literal> | ...` | `cap run '<original>'` | Cap filters adjacent unique file lines and keeps supported grep downstream count/head/tail/sort/xargs modes in-process. |
| `cat <file> | sort` | `cap run '<original>'` | Cap fuses the redundant file producer into the native sort path. |
| `cat <file> | sort | uniq` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted de-duplication path. |
| `cat <file> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses the four-stage pipeline into one sorted unique line-count path. |
| `cat <file> | sort | uniq | ...` | `cap run '<original>'` | Cap treats unique sorted file lines as a finite producer for count/head/tail/sort/xargs downstreams. |
| `cat <file> | sort | uniq | grep <literal> | ...` | `cap run '<original>'` | Cap filters unique sorted file lines and keeps supported grep downstreams in-process. |
| `cat <file> | sort | wc -l` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted line-count path. |
| `cat <file> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted prefix path. |
| `cat <file> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted suffix path. |
| `cat <file> | sort | xargs echo` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted token batching path. |
| `cat <path-list-file> | sort | xargs wc -l` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted path-list line-count path. |
| `cat <path-list-file> | sort | xargs wc -l | ...` | `cap run '<original>'` | Cap keeps supported downstream count/head/tail/sort modes on sorted direct path-list xargs-wc output. |
| `find <root> -maxdepth <positive> -type f [-name <glob>] | ...` | `cap run '<original>'` | Cap fuses a bounded file walk with supported count/head/tail/sort/sort|uniq/xargs and literal grep downstreams. |
| `find <root> -type f [-name <glob>] | xargs wc -l` | `cap run '<original>'` | Cap fuses the pipeline into one native tree-walk plus line-count path. |
| `find <root> -type f [-name <glob>] | xargs wc -l | ...` | `cap run '<original>'` | Cap treats native xargs-wc output as a finite producer for supported count/head/tail/sort/sort|uniq downstreams. |
| `find <root> -type f [-name <glob>] | xargs echo` | `cap run '<original>'` | Cap fuses the pipeline into one native tree-walk plus xargs-token output path. |
| `find <root> -type f [-name <glob>] | xargs` | `cap run '<original>'` | Cap fuses the pipeline into one native tree-walk plus default-xargs echo output path. |
| `find <root> -type f [-name <glob>] | grep <literal>` | `cap run '<original>'` | Cap filters matching paths by literal substring and returns grep-style no-match status. |
| `find <root> -type f [-name <glob>] | grep <literal> | ...` | `cap run '<original>'` | Cap treats filtered find paths as a finite producer for supported count/head/tail/sort/sort|uniq/xargs downstreams. |
| `find <root> -type f [-name <glob>] | grep <literal> | xargs echo` | `cap run '<original>'` | Cap fuses the tree walk, literal path filter, and xargs-token output path. |
| `find <root> -type f [-name <glob>] | grep <literal> | xargs wc -l` | `cap run '<original>'` | Cap fuses the tree walk, literal path filter, and line-count path. |
| `find <root> -type f [-name <glob>] | grep <literal> | sort | xargs echo` | `cap run '<original>'` | Cap fuses the tree walk, literal path filter, sort, and xargs-token output path. |
| `find <root> -type f [-name <glob>] | grep <literal> | sort | xargs wc -l` | `cap run '<original>'` | Cap fuses the tree walk, literal path filter, sort, and line-count path. |
| `find <root> -type f [-name <glob>] | grep <literal> | sort | xargs wc -l | ...` | `cap run '<original>'` | Cap treats filtered sorted native xargs-wc output as a finite producer for supported count/head/tail/sort/sort|uniq downstreams. |
| `find <root> -type f [-name <glob>] | grep <literal> | sort | uniq | xargs wc -l | ...` | `cap run '<original>'` | Cap keeps supported downstream count/head/tail/sort/sort|uniq modes on filtered sorted-unique xargs-wc output in both the planner and cap-fast frontend. |
| `find <root> -type f [-name <glob>] | wc -l` | `cap run '<original>'` | Cap fuses the tree walk and result counting into one native path. |
| `find <root> -type f [-name <glob>] | head -n <positive>` | `cap run '<original>'` | Cap fuses the tree walk and head limit so the walk stops in-process. |
| `find <root> -type f [-name <glob>] | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses the tree walk and tail window into one native path. |
| `find <root> -type f [-name <glob>] | sort` | `cap run '<original>'` | Cap fuses the tree walk and sort into one native path. |
| `find <root> -type f [-name <glob>] | sort | uniq` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted de-duplication path. |
| `find <root> -type f [-name <glob>] | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses the four-stage pipeline into one sorted unique path-count path. |
| `find <root> -type f [-name <glob>] | sort | uniq | ...` | `cap run '<original>'` | Cap fuses supported downstream count/head/tail/sort/`xargs echo`/`xargs wc -l` modes after sorted unique matching paths. |
| `find <root> -type f [-name <glob>] | sort | uniq | xargs wc -l | ...` | `cap run '<original>'` | Cap keeps supported count/head/tail/sort/sort|uniq modes on native sorted-unique xargs-wc output in both the planner and cap-fast frontend. |
| `find <root> -type f [-name <glob>] | sort | uniq | grep <literal> | ...` | `cap run '<original>'` | Cap filters sorted unique matching paths and applies supported grep downstream modes in-process. |
| `find <root> -type f [-name <glob>] | sort | wc -l` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted path-count path. |
| `find <root> -type f [-name <glob>] | sort | xargs echo` | `cap run '<original>'` | Cap fuses the four-stage pipeline into one sorted tree-walk plus xargs-token output path. |
| `find <root> -type f [-name <glob>] | sort | xargs wc -l` | `cap run '<original>'` | Cap fuses the four-stage pipeline into one sorted tree-walk plus line-count path. |
| `find <root> -type f [-name <glob>] | sort | xargs wc -l | ...` | `cap run '<original>'` | Cap keeps downstream count/head/tail/sort/sort|uniq modes on native sorted xargs-wc output. |
| `find <root> -type f [-name <glob>] | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted prefix path. |
| `find <root> -type f [-name <glob>] | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted suffix path. |
| `xargs [echo] | grep <literal> [| ...]` | `cap run '<original>'` | Cap treats default/echo xargs stdin output as a literal grep producer and keeps supported downstream count/head/tail/sort/xargs modes in-process. |
| `grep <literal> | ...` | `cap run '<original>'` | Cap reads stdin as a literal grep producer and keeps supported downstream count/head/tail/sort/xargs modes in-process. |
| `grep <literal> <file> | wc -l` | `cap run '<original>'` | Cap fuses single-file grep and line counting into one native path. |
| `grep <literal> <file> | head -n <positive>` | `cap run '<original>'` | Cap fuses single-file grep and the head limit into one native path. |
| `grep <literal> <file> | tail -n <positive>` | `cap run '<original>'` | Cap fuses single-file grep and the tail window into one native path. |
| `grep <literal> <file> | sort` | `cap run '<original>'` | Cap fuses single-file grep and sort into one native path. |
| `grep <literal> <file> | sort | uniq` | `cap run '<original>'` | Cap fuses single-file grep, sort, and unique filtering into one native path. |
| `grep <literal> <file> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses single-file grep, sort, unique filtering, and line counting into one native path. |
| `grep <literal> <file> | sort | uniq | ...` | `cap run '<original>'` | Cap treats unique sorted single-file grep output as a finite producer for supported downstream count/head/tail/sort/xargs modes. |
| `grep <literal> <file> | sort | wc -l` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted match-count path. |
| `grep <literal> <file> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted prefix path. |
| `grep <literal> <file> | sort | tail -n <positive>` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted suffix path. |
| `grep <literal> <file> | xargs echo` | `cap run '<original>'` | Cap fuses single-file grep and token batching into one native path. |
| `grep <literal> <path-list-file> | xargs wc -l` | `cap run '<original>'` | Cap fuses single-file grep and path-token line counting into one native path. |
| `grep <literal> <path-list-file> | xargs wc -l | ...` | `cap run '<original>'` | Cap treats the xargs-wc rows emitted from filtered path tokens as a finite producer for supported count/head/tail/sort downstreams. |
| `grep <literal> <file> | sort | xargs echo` | `cap run '<original>'` | Cap fuses grep, sort, and token batching into one native path. |
| `grep <literal> <path-list-file> | sort | xargs wc -l` | `cap run '<original>'` | Cap fuses grep, sort, and path-token line counting into one native path. |
| `grep <literal> <path-list-file> | sort | xargs wc -l | ...` | `cap run '<original>'` | Cap keeps downstream count/head/tail/sort modes on sorted grep path-token xargs-wc output without a second shell round trip. |
| `grep <literal> <file> | cut -d <char> -f <field>` | `cap run '<original>'` | Cap fuses single-file grep and narrow cut field extraction into one native path. |
| `grep <literal> <file> | cut -d <char> -f <field> | ...` | `cap run '<original>'` | Cap keeps supported grep/count/head/tail/sort/xargs downstreams on the cut single-file grep producer. |
| `grep <literal> <file> | awk '{ print $<field> }'` | `cap run '<original>'` | Cap fuses single-file grep and narrow awk fixed-field extraction into one native path. |
| `grep <literal> <file> | awk '{ print $<field> }' | ...` | `cap run '<original>'` | Cap keeps supported grep/count/head/tail/sort/xargs downstreams on the awk fixed-field single-file grep producer. |
| `grep -R <pattern> <root> | head -n <positive>` | `cap run '<original>'` | Cap fuses recursive grep and the head limit so upstream stops in-process. |
| `grep -R <pattern> <root> | tail -n <positive>` | `cap run '<original>'` | Cap fuses recursive grep and the tail window into one native path. |
| `grep -R <pattern> <root> | sort` | `cap run '<original>'` | Cap fuses recursive grep and sort into one native path. |
| `grep -R <pattern> <root> | sort | uniq` | `cap run '<original>'` | Cap fuses recursive grep, sort, and unique filtering into one native path. |
| `grep -R <pattern> <root> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses recursive grep, sort, unique filtering, and line counting into one native path. |
| `grep -R <pattern> <root> | sort | uniq | ...` | `cap run '<original>'` | Cap treats unique sorted recursive grep output as a finite producer for supported downstream count/head/tail/sort/`xargs echo`/`xargs wc -l` modes. |
| `grep -R <pattern> <root> | sort | wc -l` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted match-count path. |
| `grep -R <pattern> <root> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted prefix path. |
| `grep -R <pattern> <root> | sort | tail -n <positive>` | `cap run '<original>'` | Cap fuses the three-stage pipeline into one sorted suffix path. |
| `grep -R <pattern> <root> | wc -l` | `cap run '<original>'` | Cap fuses recursive grep and line counting into one native path. |
| `awk '{ print $<field> }' <file> | ...` | `cap run '<original>'` | Cap treats unfiltered fixed-field output, including action whitespace variants, as a finite producer for supported count/head/tail/sort/sort|uniq/xargs downstreams. |
| `awk '{ print $<field> }' <file> | grep <literal> | ...` | `cap run '<original>'` | Cap filters narrow awk fixed-field output by literal grep and applies supported count/head/tail/sort/xargs downstreams in-process. |
| `awk '/NEEDLE/ { print $<field> }' <file> | wc -l` | `cap run '<original>'` | Cap fuses the scan and fixed-field line counting into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | head -n <positive>` | `cap run '<original>'` | Cap fuses the scan and fixed-field prefix limiting into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses the scan and fixed-field suffix limiting into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort` | `cap run '<original>'` | Cap fuses the scan and fixed-field sorting into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | uniq` | `cap run '<original>'` | Cap fuses the scan, fixed-field sorting, and de-duplication into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | uniq | wc -l` | `cap run '<original>'` | Cap fuses the scan, fixed-field sorting, de-duplication, and line counting into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | uniq | ...` | `cap run '<original>'` | Cap treats unique sorted fixed-field output as a finite producer for supported downstream count/head/tail/sort/`xargs echo`/`xargs wc -l` modes. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | wc -l` | `cap run '<original>'` | Cap fuses the scan, fixed-field sorting, and line counting into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | head -n <positive>` | `cap run '<original>'` | Cap fuses the scan, fixed-field sorting, and prefix limiting into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | tail -n <nonnegative>` | `cap run '<original>'` | Cap fuses the scan, fixed-field sorting, and suffix limiting into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <file> | xargs echo` | `cap run '<original>'` | Cap fuses the scan and token batching into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <path-list-file> | xargs wc -l` | `cap run '<original>'` | Cap fuses the scan and path-token line counting into one in-process path. |
| `awk '{ print $<field> }' <path-list-file> | xargs wc -l | ...` | `cap run '<original>'` | Cap treats xargs-wc output from narrow awk fixed-field path tokens as a finite producer for supported count/head/tail/sort downstreams. |
| `awk '/NEEDLE/ { print $<field> }' <file> | sort | xargs echo` | `cap run '<original>'` | Cap fuses the scan, fixed-field sorting, and token batching into one in-process path. |
| `awk '/NEEDLE/ { print $<field> }' <path-list-file> | sort | xargs wc -l` | `cap run '<original>'` | Cap fuses the scan, fixed-field path sorting, and line counting into one in-process path. |
| `awk '{ print $<field> }' <path-list-file> | sort | xargs wc -l | ...` | `cap run '<original>'` | Cap keeps supported downstream count/head/tail/sort modes on sorted awk path-token xargs-wc output. |
| `which [-a] <name...> | wc -l` | `cap run '<original>'` | Cap resolves PATH executables and counts emitted lookup lines in-process. |
| `which [-a] <name...> | head -n <positive>` | `cap run '<original>'` | Cap resolves PATH executables and emits the requested lookup prefix. |
| `which [-a] <name...> | tail -n <nonnegative>` | `cap run '<original>'` | Cap resolves PATH executables and emits the requested lookup suffix. |
| `which [-a] <name...> | grep <literal> | ...` | `cap run '<original>'` | Cap keeps supported count/head/tail/sort/`xargs echo` downstreams on filtered PATH lookup output. |
| `which [-a] <name...> | xargs echo` | `cap run '<original>'` | Cap resolves PATH executables and batches them through native `xargs echo`. |
| `which [-a] <name...> | sort | ...` | `cap run '<original>'` | Cap sorts resolved PATH executables and keeps supported count/head/tail/`xargs echo` downstreams in-process. |
| `command -v <name...> | wc -l` | `cap run '<original>'` | Cap resolves Bash shell words and PATH entries, then counts emitted lookup lines. |
| `command -v <name...> | head -n <positive>` | `cap run '<original>'` | Cap resolves Bash shell words and PATH entries, then emits the requested prefix. |
| `command -v <name...> | tail -n <nonnegative>` | `cap run '<original>'` | Cap resolves Bash shell words and PATH entries, then emits the requested suffix. |
| `command -v <name...> | grep <literal> | ...` | `cap run '<original>'` | Cap keeps supported count/head/tail/sort/`xargs echo` downstreams on filtered Bash-style lookup output. |
| `command -v <name...> | xargs echo` | `cap run '<original>'` | Cap batches resolved Bash shell words and PATH entries through native `xargs echo`. |
| `command -v <name...> | sort | ...` | `cap run '<original>'` | Cap sorts resolved Bash shell words and PATH entries before supported count/head/tail/`xargs echo` downstreams. |
| `printenv <name> | wc -l` | `cap run '<original>'` | Cap reads one environment value and counts emitted value lines in-process. |
| `printenv <name> | head -n <positive>` | `cap run '<original>'` | Cap reads one environment value and emits the requested prefix. |
| `printenv <name> | tail -n <nonnegative>` | `cap run '<original>'` | Cap reads one environment value and emits the requested suffix. |
| `printenv <name> | grep <literal>` | `cap run '<original>'` | Cap reads one environment value and filters it by literal substring. |
| `printenv <name> | grep <literal> | ...` | `cap run '<original>'` | Cap keeps supported count/head/tail/sort/`xargs echo` downstreams on the filtered single-value environment producer. |
| `printenv <name> | sort` | `cap run '<original>'` | Cap reads one environment value and sorts the generated value line in-process. |
| `printenv <name> | xargs echo` | `cap run '<original>'` | Cap reads one environment value and batches it through native `xargs echo`. |
| `printenv <name> | sort | xargs echo` | `cap run '<original>'` | Cap keeps the single environment value in-process through sort and native `xargs echo`. |
| `true|false | ...` | `cap run '<original>'` | Cap treats zero-output primitives as empty finite producers for supported count/head/tail/sort/`xargs echo` downstreams. |
| `true|false | grep <literal> | ...` | `cap run '<original>'` | Cap preserves direct grep no-match status and fuses supported downstream commands on the empty stream. |
| `mkdir [-p] <path...> | ...` | `cap run '<original>'` | Cap runs the directory creation side effect and keeps the empty downstream stream in-process for supported count/head/tail/sort/`xargs echo` modes. |
| `touch <path...> | ...` | `cap run '<original>'` | Cap runs the timestamp/file side effect and keeps the empty downstream stream in-process for supported count/head/tail/sort/`xargs echo` modes. |
| `test <predicate> | ...` / `[ <predicate> ] | ...` | `cap run '<original>'` | Cap evaluates the predicate and keeps the empty downstream stream in-process for supported count/head/tail/sort/`xargs echo` modes. |
| `wc -l/-c/-w [regular-file...] | ...` | `cap run '<original>'` | Cap emits native stdin or regular-file `wc` rows and keeps supported count/head/tail/sort/`xargs echo` plus literal-grep downstreams in-process. |
| `printf <literal-format> | ...` | `cap run '<original>'` | Cap emits one no-conversion literal format arg once and keeps supported count/head/tail/sort/`xargs echo` plus literal-grep downstreams in-process. |
| `du -sk <existing-path> | ...` | `cap run '<original>'` | Cap emits the native `du -sk` row once and keeps supported count/head/tail/sort/`xargs echo` plus literal-grep downstreams in-process. |
| `hostname | wc -l` | `cap run '<original>'` | Cap reads the kernel hostname and counts the generated line in-process. |
| `hostname | head -n <positive>` | `cap run '<original>'` | Cap reads the kernel hostname and emits the requested prefix. |
| `hostname | tail -n <nonnegative>` | `cap run '<original>'` | Cap reads the kernel hostname and emits the requested suffix. |
| `hostname | grep <literal>` | `cap run '<original>'` | Cap reads the kernel hostname and filters it by literal substring. |
| `hostname | grep <literal> | ...` | `cap run '<original>'` | Cap keeps supported count/head/tail/sort/`xargs echo` downstreams on the filtered hostname line. |
| `hostname | sort` | `cap run '<original>'` | Cap reads the kernel hostname and keeps the single sorted output line in-process. |
| `hostname | xargs echo` | `cap run '<original>'` | Cap reads the kernel hostname and batches it through native `xargs echo`. |
| `hostname | sort | xargs echo` | `cap run '<original>'` | Cap keeps the hostname in-process through sort and native `xargs echo`. |
| other pipe-shaped commands | `cap run '<original>'` | Cap falls back to `bash -c` to preserve shell semantics. |

The hook does not currently rewrite each pipeline segment into `cap find |
cap xargs`, no matter how many commands appear in the pipe. Instead, cap either
recognizes the whole command string as one fused pipeline or keeps the whole
string under `bash -c`. That avoids subtle shell-behavior drift around quoting,
file descriptors, SIGPIPE, xargs batching, empty input, and per-segment exit
status.

Still not replaced by default:

| Command / shape | Status | Reason |
|---|---|---|
| option-sensitive `echo` and conversion/unsupported `printf` formats | compatibility fallback | Shells differ on echo flags and printf is a formatting language; only exact `%s`/`%s\n` and one-arg no-conversion literal formats with supported escapes are native. |
| non-integer or zero-step `seq` | compatibility fallback | General seq formatting/floating-point behavior stays with the system command. |
| unsupported `id` flags, unsupported `uname` flags, and hostname flags/arguments | compatibility fallback | Platform-specific fields beyond the listed `uname` subset and hostname mutation/format variants stay with the system commands until explicitly parity-covered. |
| compound `test` / `[` expressions, shell-string `[ ... ]`, and missing `]` argv forms | compatibility fallback | The native path only owns simple predicates; shell syntax and compound boolean semantics stay with Bash/system `test`. |
| unsupported `wc` pipe forms and non-regular explicit operands | compatibility fallback | The native pipe path owns stdin or explicit regular-file operands only for supported downstream shapes; richer `wc` options and non-regular operands stay with Bash/system `wc`. |
| `du` forms outside `du -sk <existing-path> | ...` | compatibility fallback | Missing paths, option variants, multiple operands, and stdin-like shell behavior stay with the system `du` pipeline until explicitly parity-covered. |
| general `awk` programs and xargs options outside the listed narrow subsets | compatibility fallback | These are language / option surfaces; only the listed narrow subsets are native. |
| `which` flags other than `-a` and general `command` builtin forms | compatibility fallback | The native path owns plain PATH lookup, `which -a`, and `command -v`; execution and environment-sensitive command builtin behavior stay with Bash/system commands. |
| `env` assignments/options/command-wrapper forms, `printenv` flags/multiple names, and full-environment pipes such as `env | wc -l` or `printenv | sort` | compatibility fallback | Full-environment shell pipelines observe Bash-mutated variables such as `_` and `SHLVL`; cap only owns direct environment listing plus single-name `printenv` and `printenv|grep` producer pipe shapes with parity coverage. |
| `ls <dir> | xargs wc -l` and other cwd-sensitive `ls|xargs` consumers | compatibility fallback | `ls <dir>` emits basenames while downstream commands resolve them in the current working directory, so cap only owns the safe `xargs echo` token-batching shape. |
| sort options and sort pipelines outside the listed `uniq`/`head`/`tail`/`wc -l`/`xargs` producer forms | compatibility fallback | Sort flags, locale-specific behavior, and richer downstream commands stay with the system pipeline until parity-covered. |
| general `cut` forms | compatibility fallback | Byte/char extraction, ranges, multi-field lists, suppress-undelimited mode, and multiple files stay with the system command. |
| general `tr` forms | compatibility fallback | Character classes other than exact `[:lower:]`/`[:upper:]`/`[:digit:]`, escape sequences, complement, squeeze, descending ranges, duplicate source sets, and string2 padding stay with the system command. |
| multi-operand or option-looking `yes` pipeline producers | compatibility fallback | Platform `yes` operand semantics differ; only bare `yes` and single non-option operand forms fuse with `head`. |
| unsupported shell pipes | compatibility fallback | The hook emits `cap run '<original>'`; cap internally keeps shell semantics through `bash -c` unless a fused whole-pipeline shape matches. |

Use `cap explain -- <command> ...` to see whether a command will use a native
implementation or the original command.

## Deferred and planned direction

A pass over the resource gates surfaced where the 1:1 same-name replacement
model runs out of road, and where the real wins actually are. The following are
the planned edges beyond conservative same-name takeover:

- **Pure-Rust front-end (removing the C dispatcher) — deferred.** A `no_std`,
  no-startfiles Rust front-end was prototyped and is functionally identical to
  `cap_frontend.c` (full parity) with the same direct-syscall shape. It lands
  within ~16 KB of the C binary — but that ~16 KB is exactly one 16 KiB page,
  and it is enough to lose the razor-thin `cat` gate (Rust *ties* system `cat`
  at 1.00x where C *wins* at 0.99x). The page is Rust-runtime/dyld overhead that
  survives `fixup_chains`, LTO, strip, reloc, and zero-import builds. Conclusion:
  the C hot path stays for now, and the language question becomes moot once the
  work moves to pipeline scale — 16 KB is <2% of one process floor.

- **Pipeline fusion — active for narrow shapes; broader support planned.** The large
  savings are not in replacing individual cheap commands but in collapsing a
  shell pipeline `A | B | C` into a **single in-process streaming pipeline**. A
  shell pipeline pays N process floors (~1.3 MiB each) plus OS pipe overhead and
  N fork/execs; one fused process pays one floor and none of the pipe cost.
  Rough envelope: `A | B` ≈ 0.54x RSS, `A | B | C` ≈ 0.36x — a 2–3x win, not a
  0.99x near-tie. cap now fuses representative `cat|wc`, `cat|head`,
  `cat|tail`, `cat|grep`, `cat|cut`, `cat|cut|wc/head/tail/sort`,
  `cat|cut|sort|uniq`, `cat|cut|sort|uniq|wc`, `cat|cut|xargs`,
  `cat|cut|grep|wc/head/tail/sort`, `cat|cut|grep|sort|xargs`, `cat|tr`,
  `cat|tr|wc/head/tail/sort`, `cat|tr|sort|uniq`, `cat|tr|xargs`,
  `cat|tr|grep|wc/head/tail/sort`, `cat|tr|grep|sort|xargs`, `cat|sort`,
  `cat|uniq`, `cat|uniq|wc`, `cat|uniq|head/tail/sort`, `cat|uniq|xargs`,
  `cat|uniq|grep|wc/head/tail/sort`, `cat|uniq|grep|sort|xargs`,
  `uniq|wc/head/tail/sort/xargs`, `uniq|grep|wc/head/tail/sort`,
  `uniq|grep|sort|xargs`,
  `cat|xargs|echo`, `cat|xargs|wc`, `cat|sort|uniq`, `cat|sort|uniq|wc`,
  `cat|sort|uniq|head/tail/sort`, `cat|sort|uniq|xargs`,
  `cat|sort|uniq|grep|wc/head/tail/sort`, `cat|sort|uniq|grep|sort|xargs`,
  `cat|sort|wc`, `cat|sort|head`, `cat|sort|tail`, `cat|grep|wc/head/tail/sort`,
  `cat|grep|xargs|echo`, `cat|grep|xargs|wc`, `cat|grep|sort|xargs`,
  `cat|sort|xargs|echo`, `cat|sort|xargs|wc`,
  `echo|wc`, `echo|head`,
  `echo|tail`, `echo|tr`, `echo|xargs|echo`, `echo|xargs|wc`, `printf|wc`, `printf|head`, `printf|tail`,
  `printf|grep`, `printf|grep|wc/head/tail/sort`, `printf|grep|sort|uniq`,
  `printf|grep|sort|uniq|wc`, `printf|grep|sort|wc/head/tail`,
  `printf|grep|sort|xargs|echo`, `printf|grep|xargs|echo`,
  `printf|tr`, `printf|sort`, `printf|sort|uniq`, `printf|sort|uniq|wc`,
  `printf|sort|wc`, `printf|sort|head`, `printf|sort|tail`, `printf|sort|xargs|echo`,
  `printf|sort|xargs|wc`, `printf|xargs|echo`, `printf|xargs|wc`, literal `printf` producers,
  `seq|wc`, `seq|head`, `seq|tail`,
  `seq|xargs|echo`, `xargs|wc`, `xargs|grep|wc/head/tail/sort/xargs`,
  `yes|head`, `ls|wc`, `ls|head`, `ls|tail`, `ls|sort`,
  `ls|sort|uniq`, `ls|sort|uniq|wc`, `ls|sort|wc`,
  `ls|sort|head`, `ls|sort|tail`, `ls|grep`, `ls|grep|wc`,
  `ls|grep|xargs|echo`, `ls|grep|sort|xargs|echo`, `ls|xargs|echo`,
  `sort|uniq`, `sort|uniq|wc`, `sort|uniq|head/tail/sort`,
  `sort|uniq|xargs`, `sort|uniq|grep|wc/head/tail/sort`,
  `sort|uniq|grep|sort|xargs`, `sort|head`, `sort|tail`, `sort|wc`,
  `sort|xargs|echo`, `sort|xargs|wc`,
  `head|wc`, `head|head`, `head|tail`, `head|sort`, `head|sort|uniq`,
  `head|sort|uniq|wc`, `head|sort|wc`, `head|sort|head`, `head|sort|tail`,
  `head|xargs|echo`, `head|xargs|wc`, `head|sort|xargs|echo`,
  `head|sort|xargs|wc`, `head|grep`, `head|grep|wc`, `head|grep|head`,
  `head|grep|tail`, `head|grep|sort`, `head|grep|sort|uniq`,
  `head|grep|sort|uniq|wc`, `head|grep|sort|wc`, `head|grep|sort|head`,
  `head|grep|sort|tail`, `head|grep|xargs|echo`, `head|grep|xargs|wc`,
  `head|grep|sort|xargs|echo`, `head|grep|sort|xargs|wc`,
  stdin, single-file, and recursive `grep|head`, `grep|tail`, `grep|sort`,
  `grep|sort|uniq`, `grep|sort|uniq|wc`, `grep|sort|wc`,
  `grep|sort|head`, `grep|sort|tail`, `grep|wc`, `grep|xargs|echo`, `grep|xargs|wc`,
  `grep|sort|xargs|echo`, `grep|sort|xargs|wc`,
  `sed|wc/head/tail/sort`, `sed|sort|uniq`, `sed|sort|uniq|wc`,
  `sed|sort|wc/head/tail`, `sed|xargs|echo`, `sed|xargs|wc`,
  `sed|grep`, `sed|grep|wc/head/tail/sort`, `sed|grep|sort|uniq`,
  `sed|grep|sort|uniq|wc`, `sed|grep|sort|wc/head/tail`,
  `sed|grep|xargs|echo`, `sed|grep|xargs|wc`, `sed|grep|sort|xargs`,
  `cut|wc/head/tail/sort`, `cut|sort|uniq`, `cut|sort|uniq|wc`,
  `cut|sort|wc/head/tail`, `cut|xargs|echo`, `cut|xargs|wc`,
  `cut|grep`, `cut|grep|wc/head/tail/sort`, `cut|grep|sort|uniq`,
  `cut|grep|sort|uniq|wc`, `cut|grep|sort|wc/head/tail`,
  `cut|grep|xargs|echo`, `cut|grep|xargs|wc`, `cut|grep|sort|xargs`,
  `awk|wc/head/tail/sort`, `awk|xargs|echo`, `awk|xargs|wc`, `awk|sort|xargs`,
  `which|wc`, `which|head`, `which|tail`, `which|xargs`, `which|sort`,
  `which|grep|wc/head/tail/sort/xargs`,
  `which-a|wc/xargs/sort-xargs`,
  `command-v|wc`, `command-v|head`, `command-v|tail`,
  `command-v|xargs`, `command-v|sort`, `command-v|grep|wc/head/tail/sort/xargs`,
  `env`, `printenv`, `hostname`,
  `printenv|wc`, `printenv|head`, `printenv|tail`, `printenv|grep`,
  `printenv|grep|wc/head/tail/sort/xargs`, `printenv|sort`, `printenv|xargs`,
  `printenv|sort|xargs`, `hostname|wc`, `hostname|head`, `hostname|tail`,
  `hostname|grep`, `hostname|grep|wc/head/tail/sort/xargs`, `hostname|sort`,
  `hostname|xargs`, `hostname|sort|xargs`,
  `mkdir|wc/head/tail/sort/xargs/grep`, `touch|wc/head/tail/sort/xargs/grep`,
  `test|wc/head/tail/sort/xargs/grep`, `bracket-test|wc/head/tail/sort/xargs/grep`,
  `wc-file|wc/head/tail/sort/xargs/grep`,
  `find|xargs|wc`, `find|xargs|wc|sort`,
  `find|xargs|echo`, `find|grep|xargs|echo`, `find|grep|xargs|wc`,
  `find|grep|sort|xargs|echo`, `find|grep|sort|xargs|wc`,
  `find|grep|sort|xargs|wc|sort`, `find|grep|sort|uniq|xargs|wc|sort`,
  `find|wc`, `find|head`, `find|tail`,
  `find|sort`, `find|sort|uniq`, `find|sort|uniq|wc`, `find|sort|uniq|xargs|wc|sort`,
  `find|sort|wc`, `find|sort|xargs|echo`, `find|sort|xargs|wc`,
  `find|sort|head`, and `find|sort|tail` shapes, with `bash -c`
  fallback for unfusable stages and byte-for-byte parity tests guarding
  correctness. Broader fusion work should factor more commands into reusable
  pull-based stream stages.

Command replacement is resource-benchmarked, not assumed. The benchmark compares
both public surfaces, `cap <cmd>` and hook-emitted `cap run "<command string>"`,
against the original system command with CPU time (`user + system`) and peak
RSS as the decision metrics:

```bash
cargo bench -p cap --bench command_resources
```

The latest baseline and interpretation live in `apps/cap/BENCHMARKS.md`.

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
* Same-name command replacement is the early model; marginal single-command
  gates (e.g. `cat`) are being retired toward native passthrough, and the next
  real win is resident light-shell pipeline fusion with dynamic Bash fallback — see
  [Deferred and planned direction](#deferred-and-planned-direction).
</content>
</invoke>
