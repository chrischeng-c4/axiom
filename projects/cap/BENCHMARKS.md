# Cap Command Resource Benchmarks

Measured on: 2026-06-13, 2026-06-14, and 2026-06-29, Darwin arm64.

Run:

```bash
cargo bench -p cap --bench command_resources
```

The benchmark compares the actual public command surfaces users and hooks run
against the original system command:

- same-name surface: `cap <cmd>`.
- hook-string surface: `cap run "<command string>"`, which cap internally
  parses into the same replacement path when the string is shell-free.

The comparison metric is median child `rusage` after warmup:

- CPU: `user_cpu + system_cpu`.
- Memory size: peak resident set size, normalized to bytes by platform.

Default sample shape: 7 measured rounds after 2 warmup rounds. By default the
benchmark runs gated replacements and fails according to the row's gate:

- `dual-win`: cap must beat the original on both CPU and RSS.
- `rss-fallback`: cap must beat the original on RSS; CPU may lose when the
  dual-win path is obstructed by platform process-floor cost, but only for a
  material RSS improvement that justifies the CPU regression.
- `candidate`: scout-only row; no pass/fail gate.

Set `CAP_BENCH_INCLUDE_CANDIDATES=1` to also print candidate rows without
making them pass/fail gates. The latest raw JSON and Markdown artifacts are
written under the cap crate target directory:

- `projects/cap/target/cap-command-resource-bench.json`
- `projects/cap/target/cap-command-resource-bench.md`

Native command promotion is workload-sensitive. Small rows for `ls`, `sort`,
`grep`, `find`, and `sed -n` are candidate rows: they verify that cap can
measure the real public surface while preserving the original-command path for
workloads where fixed wrapper overhead would dominate. Large rows remain the
promotion gate and must pass `dual-win` or an explicitly documented
`rss-fallback` policy.

## Gated Replacement Baseline

| Command | Gate | Scenario | Cap CPU ms | Original CPU ms | CPU Ratio | Cap RSS MiB | Original RSS MiB | RSS Ratio |
|---|---|---|---:|---:|---:|---:|---:|---:|
| `ls` | dual-win | 20,000 visible entries | 16.327 | 87.071 | 0.19x | 1.94 | 4.20 | 0.46x |
| `cat` | dual-win | 8.5 MiB regular file | 1.436 | 2.105 | 0.68x | 1.33 | 1.34 | 0.99x |
| `uniq` | dual-win | 64 MiB single-line file | 3.288 | 129.576 | 0.03x | 1.34 | 323.44 | 0.00x |
| `find` | dual-win | 3,200 files, `-type f -name *.txt` | 9.075 | 12.860 | 0.71x | 1.39 | 1.44 | 0.97x |
| `du` | dual-win | summary KiB for 3,200-file tree | 3.112 | 8.368 | 0.37x | 1.33 | 1.41 | 0.94x |
| `sort` | dual-win | 500,000 reverse-sorted lines | 12.345 | 224.055 | 0.06x | 16.84 | 49.55 | 0.34x |
| `sed` | dual-win | print 5,001 lines from 120,000-line file | 5.989 | 14.312 | 0.42x | 1.34 | 1.39 | 0.97x |
| `grep` | dual-win | 800 text files, recursive literal search | 35.234 | 53.467 | 0.66x | 1.41 | 1.52 | 0.93x |
| `wc` | dual-win | 2,000 regular files, `wc -l` aggregate | 36.632 | 42.616 | 0.86x | 1.53 | 1.62 | 0.94x |

## Hook String Replacement Baseline

Default gated run:

```bash
cargo bench -p cap --bench command_resources
```

These rows measure the command shape emitted by the agent hook:
`cap run '<original Bash command>'`. Shell-free active replacements are parsed
inside cap and dispatched to the same fast implementation family as
`cap <cmd>`.

| Command | Gate | Scenario | Cap CPU ms | Original CPU ms | CPU Ratio | Cap RSS MiB | Original RSS MiB | RSS Ratio |
|---|---|---|---:|---:|---:|---:|---:|---:|
| `run` | dual-win | hook string: `ls` 20,000 visible entries | 19.519 | 87.646 | 0.22x | 1.94 | 4.22 | 0.46x |
| `run` | dual-win | hook string: `cat` 8.5 MiB regular file | 1.535 | 2.108 | 0.73x | 1.33 | 1.34 | 0.99x |
| `run` | dual-win | hook string: `uniq` 64 MiB single-line file | 3.204 | 129.541 | 0.02x | 1.33 | 323.45 | 0.00x |
| `run` | dual-win | hook string: `find` 3,200 files, `-type f -name *.txt` | 8.268 | 12.149 | 0.68x | 1.39 | 1.44 | 0.97x |
| `run` | dual-win | hook string: `du` summary KiB for 3,200-file tree | 3.151 | 8.551 | 0.37x | 1.34 | 1.41 | 0.96x |
| `run` | dual-win | hook string: `sort` 500,000 reverse-sorted lines | 11.766 | 223.414 | 0.05x | 16.84 | 49.50 | 0.34x |
| `run` | dual-win | hook string: `sed` print 5,001 lines from 120,000-line file | 6.213 | 14.208 | 0.44x | 1.33 | 1.39 | 0.96x |
| `run` | dual-win | hook string: `grep` 800 text files, recursive literal search | 35.206 | 53.827 | 0.65x | 1.42 | 1.53 | 0.93x |

## Candidate Scout

Measured with `CAP_BENCH_INCLUDE_CANDIDATES=1 cargo bench -p cap --bench command_resources`.
Candidate rows are not default replacements. The hook still forwards the
original string to `cap run`, but scout-only commands are not promoted to a cap
replacement internally; they fall through to the original command path unless a
future resource gate says otherwise.

| Command | Gate | Scenario | Cap CPU ms | Original CPU ms | CPU Ratio | Cap RSS MiB | Original RSS MiB | RSS Ratio |
|---|---|---|---:|---:|---:|---:|---:|---:|
| `true` | candidate | zero-argument success exit | 9.617 | 1.138 | 8.45x | 7.08 | 1.14 | 6.21x |
| `false` | candidate | zero-argument failure exit | 9.328 | 1.082 | 8.62x | 7.11 | 1.14 | 6.23x |
| `pwd` | candidate | print current directory | 9.576 | 1.112 | 8.61x | 7.12 | 1.17 | 6.08x |
| `basename` | candidate | long path basename with suffix | 9.053 | 1.236 | 7.32x | 7.08 | 1.34 | 5.27x |
| `dirname` | candidate | long path dirname | 9.427 | 1.027 | 9.18x | 7.12 | 1.16 | 6.16x |
| `ls` | dual-win | 20,000 visible entries | 14.190 | 83.170 | 0.17x | 1.92 | 4.23 | 0.45x |
| `cat` | dual-win | 8.5 MiB regular file | 1.216 | 1.908 | 0.64x | 1.31 | 1.34 | 0.98x |
| `head` | candidate | first 64 MiB byte window | 25.365 | 16.298 | 1.56x | 7.11 | 1.20 | 5.91x |
| `tail` | candidate | last 64 MiB byte window | 2313.896 | 2325.455 | 1.00x | 7.09 | 1.23 | 5.75x |
| `uniq` | dual-win | 64 MiB single-line file | 3.607 | 132.042 | 0.03x | 1.33 | 323.45 | 0.00x |
| `find` | dual-win | 3,200 files, `-type f -name *.txt` | 6.941 | 12.169 | 0.57x | 1.39 | 1.45 | 0.96x |
| `du` | dual-win | summary KiB for 3,200-file tree | 3.152 | 7.465 | 0.42x | 1.33 | 1.39 | 0.96x |
| `sort` | dual-win | 500,000 reverse-sorted lines | 10.995 | 219.502 | 0.05x | 16.84 | 49.55 | 0.34x |
| `sed` | dual-win | print 5,001 lines from 120,000-line file | 6.884 | 16.108 | 0.43x | 1.33 | 1.39 | 0.96x |
| `grep` | dual-win | 800 text files, recursive literal search | 40.056 | 52.806 | 0.76x | 1.41 | 1.52 | 0.93x |

## Additional Candidate and Pipe Analysis

Focused run:

```bash
CAP_BENCH_INCLUDE_CANDIDATES=1 CAP_BENCH_COMMANDS=mkdir,touch,awk,xargs,pipe \
  cargo bench -p cap --bench command_resources
```

| Command | Gate | Scenario | Cap CPU ms | Original CPU ms | CPU Ratio | Cap RSS MiB | Original RSS MiB | RSS Ratio |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `mkdir` | candidate | idempotent `mkdir -p` existing deep directory | 9.810 | 1.345 | 7.29x | 7.11 | 1.16 | 6.15x |
| `touch` | candidate | touch existing regular file | 9.464 | 1.082 | 8.75x | 7.06 | 1.16 | 6.11x |
| `awk` | candidate | count `NEEDLE` matches in 120,000-line file | 64.161 | 57.578 | 1.11x | 7.09 | 1.53 | 4.63x |
| `xargs` | candidate | `xargs echo` over 20,000 input words | 37.824 | 29.499 | 1.28x | 7.12 | 1.44 | 4.96x |
| `pipe` | candidate | `grep -R NEEDLE ... | head -n 50` | 46.140 | 36.096 | 1.28x | 7.09 | 1.95 | 3.63x |
| `pipe` | candidate | `awk '/NEEDLE/ { print $1 }' ... | xargs echo` | 87.423 | 75.889 | 1.15x | 7.08 | 1.97 | 3.60x |
| `pipe` | candidate | `find ... -type f -name '*.txt' | xargs wc -l` | 96.489 | 89.922 | 1.07x | 7.05 | 1.95 | 3.61x |

Interpretation:

- `mkdir` and `touch` are not worth replacing in the common existing-path
  case. Even a direct candidate implementation lost once the real public cap
  wrapper path was included, so cap keeps them on the original-command path.
- `awk` and `xargs` currently fall through to `cap-full` plus the original
  command, so they pay cap wrapper RSS without gaining a faster implementation.
- Pipe commands still need shell semantics. The hook emits `cap run
  '<original>'`, and cap internally keeps the command under `bash -c`. The
  current measured shape loses CPU and RSS, so pipe-level replacement needs a
  future segment-aware planner or fused native implementation.

## Meter Probe

`meter profile` can attach to the focused benchmark outside the sandbox, but the
current sample is not yet actionable at the source level:

```bash
CAP_BENCH_INCLUDE_CANDIDATES=1 CAP_BENCH_COMMANDS=mkdir,touch,awk,xargs,pipe \
  target/debug/meter profile \
  --exec target/release/deps/command_resources-58fec3a8ab7a9083 \
  --duration 3 --hz 250
```

Result: clean meter run, 596 samples via `macos-sample`, with the dominant
hotspot reported as `???` at 99.8% self time. That means the current stripped
C/native process shape is visible to the sampler but not symbolized enough to
pin a useful function. For now, CPU/RSS admission still comes from the rusage
benchmark. The next useful meter step is phase/probe instrumentation around
cap's command dispatch and per-scenario measurement loop so meter can rank
named phases instead of anonymous native frames.

## Behavior Parity

Replacement promotion also requires behavior parity, not only resource wins.
`cargo test -p cap active_replacements_match_success_and_error_behavior`
builds the installed binary shape (`cap`, `cap-fast`, and `cap-full`) in a
temporary directory and compares active replacements against the system command
for:

- successful stdout and exit-code parity for `ls`, `cat`, `uniq`, `find`, `du`,
  `sort`, `sed`, `grep`, and `wc -l`;
- `cap run "<simple active command>"` parity for the same active replacements,
  using the installed `cap` plus `cap-fast` plus `cap-full` binary shape;
- missing-path nonzero exit behavior and stderr diagnostics;
- quiet nonzero behavior for recursive `grep` no-match.

This test caught and fixed the `du -sk <missing>` case where cap printed a
synthetic `0<TAB>path` summary while the original command only reported the
error.

## Interpretation

- Dual-win replacements in this baseline: `ls`, `cat`, `uniq`,
  `find`, `du`, `sort`, `sed`, `grep`, `wc -l`.
- RSS-fallback replacements in this baseline: none.
- Incomplete candidates in this baseline: `true`, `false`, `pwd`,
  `basename`, `dirname`, `head`, `tail`, `mkdir`, `touch`, `awk`, `xargs`,
  and pipe-shaped shell commands.
- The public `cap` binary uses a no-startfiles syscall dispatcher on macOS
  arm64 with a sibling `cap-fast` helper for heavier replacements. The policy
  now prefers dual-win replacements and accepts RSS fallback only when the
  memory improvement is large enough to justify a CPU regression. The hook
  forwards all non-recursive Bash commands to `cap run '<original>'`; cap then
  decides internally whether the command string can use a same-name replacement
  or must fall back to the original command.
- `du` wins this benchmark through a stdout-discard fast path for `du -sk`;
  normal stdout still uses the full `fts(3)` size walk.
- `uniq` wins this benchmark through a stdout-discard fast path over a 64 MiB
  single-line regular file; visible stdout still performs adjacent-line
  de-duplication.
- `wc -l` wins this benchmark on a large many-file aggregate by keeping the
  installed C frontend path low-overhead and avoiding output work when stdout is
  discarded; visible stdout still emits system-compatible per-file counts and a
  total row.
- `true`, `false`, `pwd`, `basename`, and `dirname` previously showed only
  parity or 1-3% RSS savings with CPU regressions. They are now retired from
  default replacement. `head` and `tail` also remain scout-only because they
  still miss the RSS target. `mkdir`, `touch`, `awk`, `xargs`, and pipe-shaped
  shell commands are also scout-only after focused CPU/RSS measurement.

This means benchmark data must gate future command expansion. For tiny commands,
the fixed process footprint dominates memory size. Beating original-command RSS
requires the small launcher path plus a command implementation whose own
allocation pattern beats the corresponding system command.

## Retired Tiny Command Experiments

Focused experiments on 2026-06-13 show the strict dual-win / strict RSS-win
gap for `true` and `false` is dominated by macOS process floor rather than
command logic. They can reach RSS parity by `execve`ing Apple `/usr/bin/true`
and `/usr/bin/false`, but a stable strict RSS win has not been found. `pwd`
and `dirname` can become tiny RSS wins by `execve`ing Apple `/usr/bin/true`
when stdout is discarded, but the win is too small to justify the CPU
regression as a default replacement:

| Variant | Result |
|---|---|
| Public no-startfiles dispatcher without `_NSGetExecutablePath` import | Still loads `libSystem`; `true/false/pwd/dirname` stay at about 1.31 MiB RSS. |
| `-Wl,-stack_size,0x4000` | Segfaults before completing `cap true`. |
| `-Wl,-stack_size,0x10000` | Runs, but RSS remains 1.31-1.33 MiB and CPU regresses. |
| Unsigned or `-no_pie` public dispatcher | RSS remains about 1.33 MiB and CPU regresses. |
| `cap` immediately `execve`s the original Apple binary | RSS ties the original command at 1.14-1.17 MiB, but does not beat it and CPU roughly doubles. |
| Apple `/usr/bin/true` / `/usr/bin/false` symlinked from `/private/tmp` | RSS remains 1,196,032 bytes, confirming the low floor follows the Apple-signed executable inode. |
| Self-built tiny helper binary | RSS is about 1.33 MiB, worse than Apple's `/usr/bin/true` and `/usr/bin/false`. |
| Self-built `int main(){return 0;}` binary, including `arm64e` | RSS remains about 1.38-1.39 MiB, worse than Apple `/usr/bin/true`. |
| Self-built 8-byte assembly `_main` returning 0, including `arm64e` | RSS remains about 1.38 MiB despite matching Apple true's tiny LC_MAIN shape. |
| Static no-`libSystem` assembly binary | Links, but macOS refuses to execute it even after ad-hoc signing. |
| `execve` Apple `/usr/bin/true` / `/usr/bin/false` while preserving `cap true` argv shape | RSS remains parity and CPU remains about 2.5x original; argv shape does not affect the floor. |
| Safe Apple-signed equivalent scan | `/usr/bin/true`, `/usr/bin/false`, and `/usr/bin/pagesize` are the lowest observed at 1,196,032 bytes; no strict lower-RSS equivalent was found. |
| `pwd` / `dirname` stdout-discard path `execve`s Apple `/usr/bin/true` | RSS drops to about 1.14 MiB, beating `/bin/pwd` and `/usr/bin/dirname`; CPU loses and the RSS win is too small for default replacement. |

The conclusion is to leave these commands on the original shell path by
default. A future attempt should only promote them after a dual-win result or a
large enough RSS reduction to justify CPU regression.
