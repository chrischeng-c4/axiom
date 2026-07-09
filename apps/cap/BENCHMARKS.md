# Cap Command Resource Benchmarks

Measured on: 2026-06-13, 2026-06-14, 2026-06-29, 2026-06-30, 2026-07-01, and 2026-07-02, Darwin arm64.

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
benchmark runs active replacements and fails only for rows with a resource
policy:

- `dual-win`: cap must beat the original on both CPU and RSS.
- `cpu-win`: cap must beat the original on CPU; RSS is recorded but not an
  admission gate for shapes where native streaming avoids process work but can
  tie or slightly exceed the platform RSS floor.
- `rss-fallback`: cap must beat the original on RSS; CPU may lose when the
  dual-win path is obstructed by platform process-floor cost, but only for a
  material RSS improvement that justifies the CPU regression.
- `takeover`: cap intentionally owns the safe shell-free subset even when small
  workloads lose CPU/RSS. This is an observability row, not a pass/fail resource
  gate.
- `candidate`: scout-only row; no pass/fail gate.

Set `CAP_BENCH_INCLUDE_CANDIDATES=1` to also print candidate rows without
making them pass/fail gates. The latest raw JSON and Markdown artifacts are
written under the cap crate target directory:

- `apps/cap/target/cap-command-resource-bench.json`
- `apps/cap/target/cap-command-resource-bench.md`

Native command promotion is now shape-sensitive rather than size-sensitive.
Small safe subsets use the same native path as large workloads; benchmarks keep
large resource-gated rows so cap can still prove that high-volume cases do not regress.

## Gated Replacement Baseline

| Command | Gate | Scenario | Cap CPU ms | Original CPU ms | CPU Ratio | Cap RSS MiB | Original RSS MiB | RSS Ratio |
|---|---|---|---:|---:|---:|---:|---:|---:|
| `ls` | dual-win | 20,000 visible entries | 16.328 | 87.065 | 0.19x | 1.95 | 4.22 | 0.46x |
| `cat` | dual-win | 8.5 MiB regular file | 1.326 | 1.889 | 0.70x | 1.31 | 1.33 | 0.99x |
| `wc` | dual-win | 2,000 regular files, `wc -l` aggregate | 35.136 | 40.944 | 0.86x | 1.56 | 1.61 | 0.97x |
| `wc` | takeover | 2,000 regular files, default `wc` aggregate | 32.771 | 43.884 | 0.75x | 1.56 | 1.61 | 0.97x |
| `uniq` | dual-win | 64 MiB single-line file | 2.912 | 126.441 | 0.02x | 1.36 | 323.45 | 0.00x |
| `find` | dual-win | 3,200 files, `-type f -name *.txt` | 6.904 | 11.188 | 0.62x | 1.42 | 1.45 | 0.98x |
| `du` | dual-win | summary KiB for 3,200-file tree | 2.986 | 9.802 | 0.30x | 1.36 | 1.38 | 0.99x |
| `sort` | dual-win | 500,000 reverse-sorted lines | 12.057 | 222.480 | 0.05x | 16.88 | 49.58 | 0.34x |
| `cut` | takeover | first CSV field from 200,000-line file | 15.740 | 103.399 | 0.15x | 1.38 | 1.36 | 1.01x |
| `cut` | takeover | first CSV field from 200,000-line stdin stream | 15.086 | 102.997 | 0.15x | 1.38 | 1.36 | 1.01x |
| `tr` | cpu-win | uppercase 8.6 MiB stdin stream | 10.790 | 369.272 | 0.03x | 1.39 | 1.36 | 1.02x |
| `tr` | cpu-win | class uppercase 8.6 MiB stdin stream | 9.461 | 375.501 | 0.03x | 1.39 | 1.45 | 0.96x |
| `tr` | cpu-win | delete digit class from 8.6 MiB stdin stream | 9.345 | 363.505 | 0.03x | 1.39 | 1.38 | 1.01x |
| `sed` | dual-win | print 5,001 lines from 120,000-line file | 6.020 | 14.115 | 0.43x | 1.36 | 1.38 | 0.99x |
| `awk` | dual-win | count `NEEDLE` matches in 120,000-line file | 8.589 | 56.277 | 0.15x | 1.36 | 1.50 | 0.91x |
| `awk` | takeover | count `NEEDLE` matches from stdin over 120,000 lines | 7.885 | 57.348 | 0.14x | 1.36 | 1.53 | 0.89x |
| `awk` | takeover | first-field extraction from stdin over 120,000 lines | 6.821 | 50.471 | 0.14x | 1.36 | 1.50 | 0.91x |
| `awk` | takeover | second-field extraction from stdin over 120,000 lines | 8.363 | 51.028 | 0.16x | 1.36 | 1.50 | 0.91x |
| `xargs` | dual-win | `xargs echo` over 20,000 input words | 3.365 | 32.417 | 0.10x | 1.39 | 1.44 | 0.97x |
| `xargs` | dual-win | default `xargs` echo over 20,000 input words | 3.446 | 29.006 | 0.12x | 1.39 | 1.44 | 0.97x |
| `xargs` | dual-win | `xargs -n 1 echo` over 20,000 input words | 3.626 | 43214.712 | 0.00x | 1.39 | 53.64 | 0.03x |
| `xargs` | dual-win | `xargs -n 2 echo` over 20,000 input words | 3.550 | 27076.772 | 0.00x | 1.39 | 27.50 | 0.05x |
| `xargs` | dual-win | `xargs wc -l` over 2,000 input paths | 34.653 | 50.750 | 0.68x | 1.39 | 1.64 | 0.85x |
| `which` | takeover | path lookup over external and shell builtin names | 2.694 | 1.197 | 2.25x | 1.36 | 1.19 | 1.14x |
| `which` | takeover | which -a path lookup over external and shell builtin names | 3.870 | 1.631 | 2.37x | 1.41 | 1.19 | 1.18x |
| `command` | takeover | `command -v` lookup over external and shell builtin names | 2.708 | 1.585 | 1.71x | 1.36 | 1.94 | 0.70x |
| `env` | takeover | environment listing | 2.658 | 1.047 | 2.54x | 1.36 | 1.16 | 1.18x |
| `printenv` | takeover | print all environment values | 2.488 | 1.010 | 2.46x | 1.36 | 1.16 | 1.18x |
| `printenv` | takeover | print one environment value | 2.641 | 1.004 | 2.63x | 1.36 | 1.16 | 1.18x |
| `hostname` | takeover | kernel hostname | 2.807 | 1.069 | 2.63x | 1.38 | 1.17 | 1.17x |
| `pipe` | dual-win | `cat ... | wc -l` | 7.049 | 17.170 | 0.41x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `echo ... | wc -l` | 3.347 | 7.107 | 0.47x | 1.47 | 2.62 | 0.56x |
| `pipe` | dual-win | `echo -n ... | head -n 1` | 3.000 | 6.838 | 0.44x | 1.47 | 2.59 | 0.57x |
| `pipe` | dual-win | `echo -n ... | tail -n 1` | 2.732 | 6.721 | 0.41x | 1.50 | 2.59 | 0.58x |
| `pipe` | dual-win | `echo ... | tr a-z A-Z` | 2.986 | 8.167 | 0.37x | 1.47 | 2.59 | 0.57x |
| `pipe` | dual-win | `echo ... | awk '{ print $1 }' | xargs` | 3.511 | 11.033 | 0.32x | 1.58 | 2.59 | 0.61x |
| `pipe` | dual-win | `echo ... | xargs echo` | 2.731 | 11.474 | 0.24x | 1.50 | 2.59 | 0.58x |
| `pipe` | dual-win | `echo ... | xargs wc -l` | 33.982 | 63.429 | 0.54x | 1.83 | 3.95 | 0.46x |
| `pipe` | dual-win | `xargs echo | wc -l` over stdin | 13.127 | 970.843 | 0.01x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `xargs echo | grep item-19999 | wc -l` over stdin | 3.904 | 47.901 | 0.08x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `grep NEEDLE | wc -l` over stdin | 8.267 | 14.821 | 0.56x | 1.61 | 1.94 | 0.83x |
| `pipe` | dual-win | `printf '%s\n' ... | wc -l` | 3.420 | 9.606 | 0.36x | 1.56 | 2.80 | 0.56x |
| `pipe` | dual-win | `printf '%s\n' ... | head -n 50` | 3.951 | 9.028 | 0.44x | 1.56 | 2.86 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | tail -n 50` | 3.933 | 10.143 | 0.39x | 1.58 | 2.80 | 0.56x |
| `pipe` | dual-win | `printf '%s\n' ... | awk '{ print $1 }' | wc -l` | 3.753 | 11.839 | 0.32x | 1.89 | 2.86 | 0.66x |
| `pipe` | dual-win | `printf '%s\n' ... | awk '{ print $1 }' | sort | uniq` | 3.594 | 17.009 | 0.21x | 1.89 | 2.84 | 0.66x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE` | 3.828 | 9.907 | 0.39x | 1.55 | 2.86 | 0.54x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | wc -l` | 5.003 | 14.046 | 0.36x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | head -n 50` | 5.009 | 14.751 | 0.34x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | tail -n 50` | 5.227 | 14.776 | 0.35x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort` | 4.737 | 14.488 | 0.33x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | uniq` | 4.871 | 17.832 | 0.27x | 1.55 | 2.83 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | uniq | wc -l` | 4.408 | 20.486 | 0.22x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | wc -l` | 4.475 | 18.597 | 0.24x | 1.55 | 2.86 | 0.54x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | head -n 50` | 5.410 | 16.972 | 0.32x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | tail -n 50` | 4.677 | 17.683 | 0.26x | 1.55 | 2.86 | 0.54x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | xargs echo` | 4.935 | 24.382 | 0.20x | 1.55 | 2.94 | 0.53x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | xargs echo` | 5.031 | 19.764 | 0.25x | 1.55 | 2.81 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | tr a-z A-Z` | 3.853 | 10.961 | 0.35x | 1.58 | 2.86 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | sort` | 4.230 | 10.905 | 0.39x | 1.69 | 2.80 | 0.60x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | uniq` | 4.278 | 13.758 | 0.31x | 1.70 | 2.80 | 0.61x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | uniq | wc -l` | 4.170 | 15.739 | 0.26x | 1.72 | 2.86 | 0.60x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | wc -l` | 3.317 | 12.499 | 0.27x | 1.70 | 2.86 | 0.60x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | head -n 50` | 3.823 | 12.141 | 0.31x | 1.72 | 2.83 | 0.61x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | tail -n 50` | 3.822 | 13.060 | 0.29x | 1.70 | 2.80 | 0.61x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | xargs echo` | 4.011 | 19.403 | 0.21x | 1.72 | 2.84 | 0.60x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | xargs wc -l` | 36.549 | 74.917 | 0.49x | 2.14 | 3.98 | 0.54x |
| `pipe` | dual-win | `printf '%s\n' ... | xargs echo` | 3.601 | 15.065 | 0.24x | 1.55 | 2.92 | 0.53x |
| `pipe` | dual-win | `printf '%s\n' ... | xargs wc -l` | 39.033 | 69.008 | 0.57x | 1.84 | 3.98 | 0.46x |
| `pipe` | dual-win | `seq 1 200000 | wc -l` | 3.102 | 38.523 | 0.08x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `seq 1 200000 | head -n 50` | 3.541 | 7.091 | 0.50x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `seq 1 200000 | tail -n 50` | 2.518 | 60.242 | 0.04x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `seq 1 200000 | sort` | 42.833 | 112.427 | 0.38x | 9.70 | 15.42 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | uniq` | 44.031 | 150.780 | 0.29x | 9.73 | 15.36 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | uniq | wc -l` | 43.010 | 154.456 | 0.28x | 9.70 | 15.33 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | wc -l` | 20.915 | 116.331 | 0.18x | 9.70 | 15.39 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | head -n 50` | 41.846 | 90.523 | 0.46x | 9.70 | 15.38 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | tail -n 50` | 41.859 | 140.890 | 0.30x | 9.70 | 15.39 | 0.63x |
| `pipe` | dual-win | `seq 1 5000 | sort | xargs echo` | 4.136 | 17.501 | 0.24x | 1.67 | 1.94 | 0.86x |
| `pipe` | dual-win | `seq 1 200000 | grep 199` | 15.109 | 58.200 | 0.26x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | wc -l` | 14.906 | 59.835 | 0.25x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | head -n 50` | 15.201 | 59.215 | 0.26x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | tail -n 50` | 14.614 | 59.537 | 0.25x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort` | 15.223 | 60.421 | 0.25x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | uniq` | 15.208 | 67.069 | 0.23x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | uniq | wc -l` | 15.579 | 66.738 | 0.23x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | wc -l` | 14.914 | 62.166 | 0.24x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | head -n 50` | 14.723 | 61.592 | 0.24x | 1.56 | 1.94 | 0.81x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | tail -n 50` | 14.451 | 61.493 | 0.24x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `seq 1 5000 | grep 199 | sort | xargs echo` | 3.219 | 14.412 | 0.22x | 1.39 | 1.94 | 0.72x |
| `pipe` | dual-win | `seq 1 5000 | grep 199 | xargs echo` | 3.111 | 11.678 | 0.27x | 1.39 | 1.94 | 0.72x |
| `pipe` | dual-win | `seq 1 5000 | xargs echo` | 2.788 | 12.359 | 0.23x | 1.38 | 1.94 | 0.71x |
| `pipe` | dual-win | `yes READY | head -n 20000` | 3.029 | 6.618 | 0.46x | 1.36 | 1.94 | 0.70x |
| `pipe` | takeover | `which ... | wc -l` | 2.908 | 5.337 | 0.54x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which ... | head -n 1` | 2.933 | 4.807 | 0.61x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which ... | tail -n 1` | 2.533 | 4.933 | 0.51x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which ... | grep / | wc -l` | 2.516 | 7.154 | 0.35x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which ... | xargs echo` | 2.781 | 7.580 | 0.37x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `which ... | sort | wc -l` | 2.886 | 7.314 | 0.39x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `which ... | sort | xargs echo` | 2.912 | 10.061 | 0.29x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `which -a ... | wc -l` | 2.782 | 5.302 | 0.52x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which -a ... | xargs echo` | 2.740 | 7.819 | 0.35x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which -a ... | sort | xargs echo` | 2.920 | 9.969 | 0.29x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `command -v ... | wc -l` | 2.498 | 4.147 | 0.60x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `command -v ... | head -n 1` | 2.847 | 3.459 | 0.82x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `command -v ... | tail -n 1` | 2.498 | 3.476 | 0.72x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `command -v ... | grep / | wc -l` | 2.507 | 5.784 | 0.43x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `command -v ... | xargs echo` | 2.781 | 6.677 | 0.42x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `command -v ... | sort | wc -l` | 2.791 | 6.121 | 0.46x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `command -v ... | sort | xargs echo` | 2.943 | 8.688 | 0.34x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `printenv PATH | wc -l` | 2.492 | 5.089 | 0.49x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `printenv PATH | grep /` | 2.514 | 5.247 | 0.48x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `printenv PATH | grep / | wc -l` | 2.573 | 7.550 | 0.34x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `printenv PATH | xargs echo` | 3.576 | 9.339 | 0.38x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `printenv PATH | sort | xargs echo` | 3.763 | 11.746 | 0.32x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `true | wc -l` | 2.791 | 3.997 | 0.70x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `false | wc -l` | 2.673 | 3.853 | 0.69x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `false | grep NEEDLE | wc -l` | 2.930 | 6.144 | 0.48x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `true | xargs echo` | 2.802 | 4.348 | 0.64x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `mkdir -p ... | wc -l` | 3.201 | 5.376 | 0.60x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `mkdir -p ... | xargs echo` | 3.403 | 6.093 | 0.56x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `touch ... | wc -l` | 3.346 | 5.479 | 0.61x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `touch ... | sort | xargs echo` | 3.003 | 7.355 | 0.41x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `test -f ... | wc -l` | 2.769 | 4.468 | 0.62x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `test ! -e ... | xargs echo` | 3.150 | 4.855 | 0.65x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `[ -d ... ] | sort | xargs echo` | 2.838 | 6.511 | 0.44x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `test -d ... | grep <literal> | wc -l` | 3.037 | 6.301 | 0.48x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `wc -l ... | xargs echo` | 2.547 | 7.061 | 0.36x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `wc -c ... ... | wc -l` | 2.690 | 4.829 | 0.56x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `wc -l ... ... | grep <literal> | wc -l` | 2.550 | 7.595 | 0.34x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `wc -w ... | sort | xargs echo` | 2.638 | 9.619 | 0.27x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `wc -l | wc -l` | 7.917 | 11.695 | 0.68x | 1.41 | 2.42 | 0.58x |
| `pipe` | takeover | `wc -w | grep <literal> | wc -l` | 15.336 | 24.146 | 0.64x | 1.41 | 2.42 | 0.58x |
| `pipe` | takeover | `wc -l | sort | xargs echo` | 7.344 | 16.604 | 0.44x | 1.41 | 2.42 | 0.58x |
| `pipe` | takeover | `printf <literal> | wc -l` | 2.536 | 3.548 | 0.71x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `printf <literal> | grep <literal> | wc -l` | 2.481 | 6.098 | 0.41x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `printf <literal> | sort | xargs echo` | 2.759 | 8.630 | 0.32x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `du -sk ... | wc -l` | 8.183 | 12.791 | 0.64x | 1.48 | 1.94 | 0.77x |
| `pipe` | takeover | `du -sk ... | xargs echo` | 8.448 | 13.405 | 0.63x | 1.48 | 1.94 | 0.77x |
| `pipe` | takeover | `du -sk ... | grep <literal> | wc -l` | 8.268 | 13.463 | 0.61x | 1.48 | 1.94 | 0.77x |
| `pipe` | takeover | `hostname | wc -l` | 2.640 | 5.160 | 0.51x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | head -n 1` | 2.555 | 4.660 | 0.55x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | tail -n 1` | 2.511 | 4.819 | 0.52x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | grep <literal> | wc -l` | 2.489 | 7.284 | 0.34x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | sort` | 2.626 | 5.141 | 0.51x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `hostname | xargs echo` | 3.832 | 9.596 | 0.40x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | sort | xargs echo` | 4.951 | 12.321 | 0.40x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `ls -1 ... | wc -l` | 24.578 | 93.775 | 0.26x | 2.00 | 4.16 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | head -n 50` | 25.484 | 88.518 | 0.29x | 1.97 | 4.17 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | tail -n 50` | 26.159 | 99.196 | 0.26x | 1.97 | 4.17 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort` | 28.677 | 101.536 | 0.28x | 1.97 | 4.16 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort | uniq` | 27.738 | 109.440 | 0.25x | 2.00 | 4.19 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | sort | uniq | wc -l` | 19.586 | 108.003 | 0.18x | 1.97 | 4.16 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort | wc -l` | 27.286 | 103.354 | 0.26x | 1.97 | 4.16 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort | head -n 50` | 23.979 | 98.686 | 0.24x | 1.97 | 4.16 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort | tail -n 50` | 20.146 | 105.043 | 0.19x | 1.97 | 4.17 | 0.47x |
| `pipe` | takeover | `ls -1 ... | sort | xargs echo` | 2.777 | 9.668 | 0.29x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `ls -1 ... | grep file-19` | 18.241 | 93.322 | 0.20x | 2.00 | 4.17 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | grep file-19 | wc -l` | 18.742 | 94.033 | 0.20x | 1.97 | 4.14 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | grep file-19 | xargs echo` | 25.210 | 102.625 | 0.25x | 1.98 | 4.17 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | grep file-19 | sort | xargs echo` | 22.252 | 104.589 | 0.21x | 1.98 | 4.17 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | xargs echo` | 28.453 | 148.660 | 0.19x | 2.03 | 4.20 | 0.48x |
| `pipe` | takeover | `ls -a ... | wc -l` | 3.884 | 7.660 | 0.51x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -a ... | grep hidden | wc -l` | 3.521 | 9.626 | 0.37x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -a ... | sort | tail -n 1` | 3.443 | 8.929 | 0.39x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `ls -a ... | sort | xargs echo` | 2.587 | 9.234 | 0.28x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -a ... | xargs echo` | 4.042 | 9.920 | 0.41x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -A ... | wc -l` | 4.216 | 6.718 | 0.63x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -A ... | grep hidden | wc -l` | 4.667 | 12.463 | 0.37x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -A ... | sort | tail -n 1` | 5.189 | 8.678 | 0.60x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `ls -A ... | sort | xargs echo` | 2.954 | 9.370 | 0.32x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -A ... | xargs echo` | 4.663 | 10.720 | 0.43x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `sort ... | uniq` | 15.362 | 344.213 | 0.04x | 16.88 | 49.50 | 0.34x |
| `pipe` | dual-win | `sort ... | uniq | wc -l` | 13.558 | 348.138 | 0.04x | 16.88 | 49.52 | 0.34x |
| `pipe` | dual-win | `sort ... | head -n 50` | 9.500 | 151.359 | 0.06x | 16.88 | 49.50 | 0.34x |
| `pipe` | dual-win | `sort ... | tail -n 50` | 9.443 | 335.503 | 0.03x | 16.89 | 49.50 | 0.34x |
| `pipe` | dual-win | `sort ... | wc -l` | 9.442 | 232.446 | 0.04x | 16.88 | 49.50 | 0.34x |
| `pipe` | dual-win | `sort ... | xargs echo` | 2.858 | 16.962 | 0.17x | 1.67 | 2.03 | 0.82x |
| `pipe` | dual-win | `sort ... | xargs wc -l` | 33.512 | 56.826 | 0.59x | 1.67 | 2.20 | 0.76x |
| `pipe` | dual-win | `head -n 50000 ... | wc -l` | 4.408 | 11.449 | 0.39x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `head -n 50000 | wc -l` over stdin | 3.256 | 8.490 | 0.38x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `head -n 50000 ... | head -n 50` | 4.846 | 6.973 | 0.69x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `head -n 50000 ... | tail -n 50` | 7.779 | 23.044 | 0.34x | 1.47 | 1.97 | 0.75x |
| `pipe` | dual-win | `head -n 50000 ... | sort` | 9.491 | 32.193 | 0.29x | 3.06 | 6.20 | 0.49x |
| `pipe` | dual-win | `head -n 50000 ... | sort | uniq` | 9.277 | 46.656 | 0.20x | 3.06 | 6.22 | 0.49x |
| `pipe` | dual-win | `head -n 50000 ... | sort | uniq | wc -l` | 9.920 | 49.369 | 0.20x | 3.09 | 6.14 | 0.50x |
| `pipe` | dual-win | `head -n 50000 ... | sort | wc -l` | 9.215 | 35.128 | 0.26x | 3.06 | 6.14 | 0.50x |
| `pipe` | dual-win | `head -n 50000 ... | sort | head -n 50` | 9.188 | 27.531 | 0.33x | 3.09 | 6.14 | 0.50x |
| `pipe` | dual-win | `head -n 50000 ... | sort | tail -n 50` | 8.704 | 45.739 | 0.19x | 3.06 | 6.14 | 0.50x |
| `pipe` | dual-win | `head -n 50000 ... | xargs echo` | 6.626 | 99.185 | 0.07x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `head -n 500 ... | xargs wc -l` | 12.030 | 22.136 | 0.54x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `head -n 50000 ... | sort | xargs echo` | 11.105 | 123.492 | 0.09x | 3.06 | 6.14 | 0.50x |
| `pipe` | dual-win | `head -n 500 ... | sort | xargs wc -l` | 12.464 | 25.866 | 0.48x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499` | 7.501 | 15.709 | 0.48x | 1.39 | 1.94 | 0.72x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | wc -l` | 5.787 | 19.150 | 0.30x | 1.41 | 1.97 | 0.71x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | head -n 50` | 5.667 | 17.627 | 0.32x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | tail -n 50` | 7.105 | 18.592 | 0.38x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | sort` | 6.243 | 19.352 | 0.32x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | sort | uniq` | 6.849 | 21.439 | 0.32x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | sort | uniq | wc -l` | 7.292 | 23.488 | 0.31x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | sort | wc -l` | 7.291 | 20.439 | 0.36x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | sort | head -n 50` | 7.099 | 21.474 | 0.33x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | sort | tail -n 50` | 7.106 | 21.930 | 0.32x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | xargs echo` | 6.068 | 23.173 | 0.26x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `head -n 500 ... | grep count-0 | xargs wc -l` | 12.461 | 23.098 | 0.54x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `head -n 50000 ... | grep 499 | sort | xargs echo` | 6.967 | 25.796 | 0.27x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `head -n 500 ... | grep count-0 | sort | xargs wc -l` | 12.120 | 28.739 | 0.42x | 1.50 | 1.94 | 0.77x |
| `pipe` | dual-win | `tail -n 50000 ... | wc -l` | 4.520 | 9.778 | 0.46x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 50000 | wc -l` over stdin | 5.558 | 8.309 | 0.67x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 50000 ... | head -n 50` | 4.760 | 7.810 | 0.61x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 50000 ... | tail -n 50` | 7.467 | 20.859 | 0.36x | 1.47 | 1.97 | 0.75x |
| `pipe` | dual-win | `tail -n 50000 ... | sort` | 9.736 | 31.553 | 0.31x | 3.08 | 6.19 | 0.50x |
| `pipe` | dual-win | `tail -n 50000 ... | sort | uniq` | 10.172 | 46.300 | 0.22x | 3.12 | 6.11 | 0.51x |
| `pipe` | dual-win | `tail -n 50000 ... | sort | uniq | wc -l` | 9.948 | 49.169 | 0.20x | 3.09 | 6.09 | 0.51x |
| `pipe` | dual-win | `tail -n 50000 ... | sort | wc -l` | 9.561 | 34.816 | 0.27x | 3.08 | 6.09 | 0.51x |
| `pipe` | dual-win | `tail -n 50000 ... | sort | head -n 50` | 9.352 | 26.264 | 0.36x | 3.08 | 6.09 | 0.51x |
| `pipe` | dual-win | `tail -n 50000 ... | sort | tail -n 50` | 9.301 | 45.234 | 0.21x | 3.08 | 6.17 | 0.50x |
| `pipe` | dual-win | `tail -n 50000 ... | xargs echo` | 5.691 | 96.847 | 0.06x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 500 ... | xargs wc -l` | 11.936 | 21.751 | 0.55x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `tail -n 50000 ... | sort | xargs echo` | 11.684 | 119.462 | 0.10x | 3.05 | 6.19 | 0.49x |
| `pipe` | dual-win | `tail -n 500 ... | sort | xargs wc -l` | 12.131 | 25.745 | 0.47x | 1.56 | 1.94 | 0.81x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049` | 7.170 | 15.975 | 0.45x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | wc -l` | 7.209 | 17.894 | 0.40x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | head -n 50` | 7.062 | 17.719 | 0.40x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | tail -n 50` | 6.902 | 18.223 | 0.38x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | sort` | 7.196 | 18.660 | 0.39x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | sort | uniq` | 6.922 | 18.963 | 0.37x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | sort | uniq | wc -l` | 6.989 | 23.388 | 0.30x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | sort | wc -l` | 6.988 | 20.282 | 0.34x | 1.56 | 1.94 | 0.81x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | sort | head -n 50` | 7.039 | 20.170 | 0.35x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | sort | tail -n 50` | 7.424 | 20.918 | 0.35x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | xargs echo` | 7.060 | 22.607 | 0.31x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 500 ... | grep count-19 | xargs wc -l` | 5.563 | 15.559 | 0.36x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `tail -n 50000 ... | grep 049 | sort | xargs echo` | 7.292 | 25.169 | 0.29x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `tail -n 500 ... | grep count-19 | sort | xargs wc -l` | 5.487 | 18.465 | 0.30x | 1.44 | 1.97 | 0.73x |
| `pipe` | dual-win | `cat ... | head -n 50` | 2.803 | 5.543 | 0.51x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `cat ... | tail -n 50` | 6.848 | 34.381 | 0.20x | 1.39 | 1.97 | 0.71x |
| `pipe` | dual-win | `cat ... | grep NEEDLE` | 7.300 | 15.668 | 0.47x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | wc -l` | 7.306 | 16.938 | 0.43x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | head -n 50` | 7.386 | 16.886 | 0.44x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | tail -n 50` | 6.846 | 16.505 | 0.41x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort` | 7.067 | 17.565 | 0.40x | 1.56 | 1.94 | 0.81x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | uniq` | 7.391 | 20.157 | 0.37x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | uniq | wc -l` | 7.197 | 21.675 | 0.33x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | wc -l` | 7.012 | 19.925 | 0.35x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | head -n 50` | 7.305 | 19.450 | 0.38x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | tail -n 50` | 7.221 | 19.895 | 0.36x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | xargs echo` | 7.337 | 22.473 | 0.33x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep count-19 | xargs wc -l` | 4.727 | 13.110 | 0.36x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | xargs echo` | 7.183 | 24.595 | 0.29x | 1.59 | 1.94 | 0.82x |
| `pipe` | dual-win | `cat ... | grep count-19 | sort | xargs wc -l` | 4.658 | 15.267 | 0.31x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `cut -d, -f1 | wc -l` over stdin | 10.824 | 112.324 | 0.10x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `cat ... | cut -d, -f1` | 15.425 | 108.714 | 0.14x | 1.34 | 1.94 | 0.69x |
| `pipe` | dual-win | `cat ... | tr a-z A-Z` | 10.612 | 383.455 | 0.03x | 1.34 | 1.94 | 0.69x |
| `pipe` | dual-win | `cat ... | xargs echo` | 2.826 | 13.957 | 0.20x | 1.48 | 1.94 | 0.77x |
| `pipe` | dual-win | `cat ... | xargs wc -l` | 33.783 | 51.624 | 0.65x | 1.59 | 1.94 | 0.82x |
| `pipe` | dual-win | `cat ... | uniq` | 32.763 | 121.824 | 0.27x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `cat ... | uniq | wc -l` | 45.684 | 134.298 | 0.34x | 18.86 | 1.97 | 9.58x |
| `pipe` | dual-win | `cat ... | sort` | 13.384 | 220.348 | 0.06x | 16.88 | 49.59 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | uniq` | 16.733 | 342.942 | 0.05x | 16.88 | 49.56 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | uniq | wc -l` | 13.424 | 350.750 | 0.04x | 16.89 | 49.59 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | wc -l` | 10.531 | 234.342 | 0.04x | 16.89 | 49.59 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | head -n 50` | 10.596 | 149.660 | 0.07x | 16.88 | 49.56 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | tail -n 50` | 10.353 | 337.075 | 0.03x | 16.88 | 49.58 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | xargs echo` | 2.912 | 18.589 | 0.16x | 1.66 | 2.09 | 0.79x |
| `pipe` | dual-win | `cat ... | sort | xargs wc -l` | 33.434 | 58.787 | 0.57x | 1.67 | 2.27 | 0.74x |
| `pipe` | dual-win | `grep -R NEEDLE ... | head -n 50` | 4.663 | 25.156 | 0.19x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `grep -R NEEDLE ... | tail -n 50` | 22.479 | 52.241 | 0.43x | 1.48 | 1.94 | 0.77x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort` | 22.050 | 53.750 | 0.41x | 1.61 | 1.94 | 0.83x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | uniq` | 22.500 | 56.823 | 0.40x | 1.58 | 1.94 | 0.81x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | uniq | wc -l` | 22.038 | 59.772 | 0.37x | 1.59 | 1.94 | 0.82x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | wc -l` | 22.015 | 55.861 | 0.39x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | head -n 50` | 22.551 | 47.324 | 0.48x | 1.64 | 1.94 | 0.85x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | tail -n 50` | 22.250 | 48.293 | 0.46x | 1.59 | 1.94 | 0.82x |
| `pipe` | dual-win | `grep -R NEEDLE ... | wc -l` | 21.547 | 40.901 | 0.53x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `grep NEEDLE ... | wc -l` | 7.401 | 14.831 | 0.50x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `grep NEEDLE ... | head -n 50` | 7.051 | 14.179 | 0.50x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep NEEDLE ... | tail -n 50` | 7.307 | 15.083 | 0.48x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `grep NEEDLE ... | sort` | 7.201 | 15.848 | 0.45x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | uniq` | 7.223 | 18.015 | 0.40x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | uniq | wc -l` | 7.161 | 20.507 | 0.35x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | wc -l` | 7.293 | 17.339 | 0.42x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | head -n 50` | 7.623 | 17.775 | 0.43x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | tail -n 50` | 7.336 | 17.863 | 0.41x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `grep NEEDLE ... | xargs echo` | 7.093 | 20.355 | 0.35x | 1.56 | 1.94 | 0.81x |
| `pipe` | dual-win | `grep count- ... | xargs wc -l` | 34.063 | 53.475 | 0.64x | 1.69 | 1.94 | 0.87x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | xargs echo` | 7.267 | 23.324 | 0.31x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep count- ... | sort | xargs wc -l` | 34.856 | 59.008 | 0.59x | 1.70 | 2.22 | 0.77x |
| `pipe` | takeover | `awk '{ print $1 }' | wc -l` over stdin | 10.696 | 57.294 | 0.19x | 5.30 | 1.94 | 2.73x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | wc -l` | 6.700 | 59.661 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | head -n 50` | 6.838 | 59.712 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | tail -n 50` | 6.808 | 60.087 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort` | 6.633 | 60.458 | 0.11x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | uniq` | 6.735 | 62.373 | 0.11x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | uniq | wc -l` | 6.922 | 66.190 | 0.10x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | wc -l` | 7.215 | 64.000 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | head -n 50` | 7.139 | 64.123 | 0.11x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | tail -n 50` | 7.183 | 64.326 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | xargs echo` | 7.035 | 65.131 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | xargs wc -l` | 34.615 | 57.144 | 0.61x | 1.69 | 1.94 | 0.87x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | xargs echo` | 7.108 | 67.527 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | xargs wc -l` | 34.388 | 62.972 | 0.55x | 1.69 | 2.23 | 0.76x |
| `pipe` | dual-win | `find ... -type f | xargs wc -l` | 87.442 | 128.775 | 0.68x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `find ... -type f | xargs echo` | 8.080 | 36.945 | 0.22x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `find ... -type f | xargs` | 8.500 | 41.883 | 0.20x | 1.48 | 1.94 | 0.77x |
| `pipe` | dual-win | `find ... -type f | wc -l` | 7.143 | 14.792 | 0.48x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `find ... -type f | head -n 50` | 2.549 | 6.096 | 0.42x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `find ... -type f | tail -n 50` | 7.317 | 22.077 | 0.33x | 1.47 | 1.94 | 0.76x |
| `pipe` | dual-win | `find ... -type f | sort | wc -l` | 7.716 | 45.939 | 0.17x | 2.05 | 3.44 | 0.60x |
| `pipe` | takeover | `find ... -maxdepth 1 -type f | wc -l` | 2.600 | 5.277 | 0.49x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 1 -type f | head -n 5` | 2.556 | 5.355 | 0.48x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 1 -type f | grep <literal> | wc -l` | 2.596 | 7.274 | 0.36x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 1 -type f | xargs echo` | 2.583 | 8.025 | 0.32x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 2 -type f | sort | tail -n 1` | 2.588 | 7.696 | 0.34x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 2 -type f -name '*.rs' | grep <literal> | wc -l` | 2.641 | 7.268 | 0.36x | 1.47 | 1.94 | 0.76x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | xargs wc -l` | 31.415 | 53.331 | 0.59x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | xargs echo` | 6.842 | 23.961 | 0.29x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | xargs` | 7.426 | 25.468 | 0.29x | 1.48 | 1.94 | 0.77x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | grep item-019 | xargs echo` | 7.616 | 21.159 | 0.36x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | grep item-019 | xargs wc -l` | 8.225 | 22.808 | 0.36x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | grep item-019 | sort | xargs echo` | 9.827 | 25.670 | 0.38x | 1.67 | 1.94 | 0.86x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | grep item-019 | sort | xargs wc -l` | 10.066 | 27.874 | 0.36x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | wc -l` | 6.848 | 15.214 | 0.45x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | head -n 50` | 2.770 | 7.302 | 0.38x | 1.39 | 1.94 | 0.72x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | tail -n 50` | 6.791 | 17.246 | 0.39x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort` | 7.125 | 24.807 | 0.29x | 1.69 | 2.14 | 0.79x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | uniq` | 7.122 | 28.483 | 0.25x | 1.66 | 2.06 | 0.80x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | uniq | wc -l` | 6.770 | 30.495 | 0.22x | 1.66 | 2.11 | 0.79x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | wc -l` | 7.250 | 26.830 | 0.27x | 1.67 | 2.06 | 0.81x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | xargs echo` | 7.377 | 35.894 | 0.21x | 1.66 | 2.06 | 0.80x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | xargs wc -l` | 32.737 | 64.631 | 0.51x | 1.69 | 2.06 | 0.82x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | head -n 50` | 7.213 | 26.230 | 0.27x | 1.66 | 2.06 | 0.80x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | tail -n 50` | 7.073 | 29.019 | 0.24x | 1.67 | 2.09 | 0.80x |
| `grep` | dual-win | 800 text files, recursive literal search | 23.054 | 41.692 | 0.55x | 1.44 | 1.50 | 0.96x |
| `grep` | dual-win | single large text file, literal search | 7.350 | 10.562 | 0.70x | 1.38 | 1.42 | 0.97x |

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
| `run` | dual-win | hook string: `ls` 20,000 visible entries | 17.094 | 86.053 | 0.20x | 1.95 | 4.22 | 0.46x |
| `run` | dual-win | hook string: `cat` 8.5 MiB regular file | 1.283 | 1.953 | 0.66x | 1.31 | 1.33 | 0.99x |
| `run` | dual-win | hook string: `uniq` 64 MiB single-line file | 3.084 | 127.043 | 0.02x | 1.36 | 323.45 | 0.00x |
| `run` | dual-win | hook string: `find` 3,200 files, `-type f -name *.txt` | 9.658 | 13.975 | 0.69x | 1.42 | 1.48 | 0.96x |
| `run` | dual-win | hook string: `du` summary KiB for 3,200-file tree | 2.999 | 9.907 | 0.30x | 1.36 | 1.39 | 0.98x |
| `run` | dual-win | hook string: `sort` 500,000 reverse-sorted lines | 12.320 | 222.394 | 0.06x | 16.88 | 49.53 | 0.34x |
| `run` | dual-win | hook string: `cut` first CSV field from 200,000-line file | 14.665 | 102.088 | 0.14x | 1.34 | 1.36 | 0.99x |
| `run` | cpu-win | hook string: `tr` uppercase 8.6 MiB stdin stream | 9.207 | 365.656 | 0.03x | 1.34 | 1.36 | 0.99x |
| `run` | dual-win | hook string: `sed` print 5,001 lines from 120,000-line file | 5.627 | 13.990 | 0.40x | 1.36 | 1.38 | 0.99x |
| `run` | dual-win | hook string: `grep` 800 text files, recursive literal search | 23.682 | 49.601 | 0.48x | 1.42 | 1.52 | 0.94x |

## Takeover And Fused Pipeline Focus

These rows highlight the two intentional policy edges:

- `takeover` rows are default same-name replacements without a CPU/RSS gate.
- `cpu-win` `tr` rows are active replacements that must beat the original on CPU.
- `dual-win` `xargs`, `awk`, and `pipe` rows are active replacements that must
  beat the original on both CPU and RSS.

Pipe rows are measured through the hook-string surface `cap run '<original>'`;
matching shapes are fused inside cap instead of being delegated to `bash -c`.

| Command | Gate | Scenario | Cap CPU ms | Original CPU ms | CPU Ratio | Cap RSS MiB | Original RSS MiB | RSS Ratio |
|---|---|---|---:|---:|---:|---:|---:|---:|
| `true` | takeover | zero-argument success exit | 2.885 | 1.156 | 2.50x | 1.36 | 1.16 | 1.18x |
| `false` | takeover | zero-argument failure exit | 3.214 | 1.153 | 2.79x | 1.36 | 1.16 | 1.18x |
| `pwd` | takeover | print current directory | 3.019 | 1.064 | 2.84x | 1.36 | 1.17 | 1.16x |
| `echo` | takeover | 2,000 plain words | 3.011 | 2.985 | 1.01x | 1.39 | 1.22 | 1.14x |
| `printf` | takeover | 2,000 `%s\n` arguments | 2.962 | 2.277 | 1.30x | 1.41 | 1.39 | 1.01x |
| `seq` | takeover | integer range 1 to 200,000 | 2.764 | 31.583 | 0.09x | 1.36 | 1.22 | 1.12x |
| `whoami` | takeover | effective user name | 3.003 | 1.517 | 1.98x | 1.44 | 1.25 | 1.15x |
| `id` | takeover | default identity summary | 3.326 | 1.672 | 1.99x | 1.59 | 1.41 | 1.13x |
| `id` | takeover | effective user id | 2.715 | 0.999 | 2.72x | 1.38 | 1.17 | 1.17x |
| `id` | takeover | effective user name | 3.239 | 1.421 | 2.28x | 1.47 | 1.23 | 1.19x |
| `id` | takeover | effective group id | 2.647 | 1.071 | 2.47x | 1.38 | 1.17 | 1.17x |
| `id` | takeover | effective group name | 2.868 | 1.328 | 2.16x | 1.47 | 1.23 | 1.19x |
| `id` | takeover | supplementary group id list | 2.546 | 1.276 | 2.00x | 1.39 | 1.27 | 1.10x |
| `id` | takeover | supplementary group name list | 3.511 | 1.675 | 2.10x | 1.61 | 1.39 | 1.16x |
| `uname` | takeover | machine architecture field | 3.314 | 1.521 | 2.18x | 1.38 | 1.17 | 1.17x |
| `uname` | takeover | processor architecture field | 3.092 | 1.325 | 2.33x | 1.38 | 1.17 | 1.17x |
| `uname` | takeover | all utsname fields | 2.782 | 1.021 | 2.72x | 1.36 | 1.17 | 1.16x |
| `test` | takeover | `test -f` regular file | 2.733 | 1.160 | 2.36x | 1.36 | 1.33 | 1.02x |
| `test` | takeover | integer comparison predicate | 2.753 | 1.134 | 2.43x | 1.36 | 1.33 | 1.02x |
| `[` | takeover | `[ -d directory ]` predicate | 2.777 | 1.149 | 2.42x | 1.36 | 1.33 | 1.02x |
| `basename` | takeover | long path basename with suffix | 2.683 | 1.308 | 2.05x | 1.36 | 1.33 | 1.02x |
| `dirname` | takeover | long path dirname | 2.656 | 1.075 | 2.47x | 1.36 | 1.17 | 1.16x |
| `mkdir` | takeover | idempotent `mkdir -p` existing deep directory | 2.874 | 1.223 | 2.35x | 1.36 | 1.16 | 1.18x |
| `touch` | takeover | touch existing regular file | 2.497 | 1.153 | 2.17x | 1.36 | 1.14 | 1.19x |
| `head` | takeover | first 64 MiB byte window | 10.668 | 16.821 | 0.63x | 1.36 | 1.22 | 1.12x |
| `tail` | takeover | last 64 MiB byte window | 10.726 | 2339.822 | 0.00x | 1.36 | 1.23 | 1.10x |
| `cut` | takeover | first CSV field from 200,000-line file | 15.740 | 103.399 | 0.15x | 1.38 | 1.36 | 1.01x |
| `cut` | takeover | first CSV field from 200,000-line stdin stream | 15.086 | 102.997 | 0.15x | 1.38 | 1.36 | 1.01x |
| `tr` | cpu-win | uppercase 8.6 MiB stdin stream | 10.790 | 369.272 | 0.03x | 1.39 | 1.36 | 1.02x |
| `tr` | cpu-win | class uppercase 8.6 MiB stdin stream | 9.461 | 375.501 | 0.03x | 1.39 | 1.45 | 0.96x |
| `tr` | cpu-win | delete digit class from 8.6 MiB stdin stream | 9.345 | 363.505 | 0.03x | 1.39 | 1.38 | 1.01x |
| `awk` | dual-win | count `NEEDLE` matches in 120,000-line file | 8.589 | 56.277 | 0.15x | 1.36 | 1.50 | 0.91x |
| `awk` | takeover | count `NEEDLE` matches from stdin over 120,000 lines | 7.885 | 57.348 | 0.14x | 1.36 | 1.53 | 0.89x |
| `awk` | takeover | first-field extraction from stdin over 120,000 lines | 6.821 | 50.471 | 0.14x | 1.36 | 1.50 | 0.91x |
| `awk` | takeover | second-field extraction from stdin over 120,000 lines | 8.363 | 51.028 | 0.16x | 1.36 | 1.50 | 0.91x |
| `xargs` | dual-win | `xargs echo` over 20,000 input words | 3.365 | 32.417 | 0.10x | 1.39 | 1.44 | 0.97x |
| `xargs` | dual-win | default `xargs` echo over 20,000 input words | 3.446 | 29.006 | 0.12x | 1.39 | 1.44 | 0.97x |
| `xargs` | dual-win | `xargs -n 1 echo` over 20,000 input words | 3.626 | 43214.712 | 0.00x | 1.39 | 53.64 | 0.03x |
| `xargs` | dual-win | `xargs -n 2 echo` over 20,000 input words | 3.550 | 27076.772 | 0.00x | 1.39 | 27.50 | 0.05x |
| `xargs` | dual-win | `xargs wc -l` over 2,000 input paths | 34.653 | 50.750 | 0.68x | 1.39 | 1.64 | 0.85x |
| `which` | takeover | path lookup over external and shell builtin names | 2.694 | 1.197 | 2.25x | 1.36 | 1.19 | 1.14x |
| `which` | takeover | which -a path lookup over external and shell builtin names | 3.870 | 1.631 | 2.37x | 1.41 | 1.19 | 1.18x |
| `command` | takeover | `command -v` lookup over external and shell builtin names | 2.708 | 1.585 | 1.71x | 1.36 | 1.94 | 0.70x |
| `env` | takeover | environment listing | 2.658 | 1.047 | 2.54x | 1.36 | 1.16 | 1.18x |
| `printenv` | takeover | print all environment values | 2.488 | 1.010 | 2.46x | 1.36 | 1.16 | 1.18x |
| `printenv` | takeover | print one environment value | 2.641 | 1.004 | 2.63x | 1.36 | 1.16 | 1.18x |
| `hostname` | takeover | kernel hostname | 2.807 | 1.069 | 2.63x | 1.38 | 1.17 | 1.17x |
| `pipe` | dual-win | `cat ... | wc -l` | 7.049 | 17.170 | 0.41x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `echo ... | wc -l` | 3.347 | 7.107 | 0.47x | 1.47 | 2.62 | 0.56x |
| `pipe` | dual-win | `echo -n ... | head -n 1` | 3.000 | 6.838 | 0.44x | 1.47 | 2.59 | 0.57x |
| `pipe` | dual-win | `echo -n ... | tail -n 1` | 2.732 | 6.721 | 0.41x | 1.50 | 2.59 | 0.58x |
| `pipe` | dual-win | `echo ... | tr a-z A-Z` | 2.986 | 8.167 | 0.37x | 1.47 | 2.59 | 0.57x |
| `pipe` | dual-win | `echo ... | awk '{ print $1 }' | xargs` | 3.511 | 11.033 | 0.32x | 1.58 | 2.59 | 0.61x |
| `pipe` | dual-win | `echo ... | xargs echo` | 2.731 | 11.474 | 0.24x | 1.50 | 2.59 | 0.58x |
| `pipe` | dual-win | `echo ... | xargs wc -l` | 33.982 | 63.429 | 0.54x | 1.83 | 3.95 | 0.46x |
| `pipe` | dual-win | `xargs echo | wc -l` over stdin | 13.127 | 970.843 | 0.01x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `xargs echo | grep item-19999 | wc -l` over stdin | 3.904 | 47.901 | 0.08x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `grep NEEDLE | wc -l` over stdin | 8.267 | 14.821 | 0.56x | 1.61 | 1.94 | 0.83x |
| `pipe` | dual-win | `printf '%s\n' ... | wc -l` | 3.420 | 9.606 | 0.36x | 1.56 | 2.80 | 0.56x |
| `pipe` | dual-win | `printf '%s\n' ... | head -n 50` | 3.951 | 9.028 | 0.44x | 1.56 | 2.86 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | tail -n 50` | 3.933 | 10.143 | 0.39x | 1.58 | 2.80 | 0.56x |
| `pipe` | dual-win | `printf '%s\n' ... | awk '{ print $1 }' | wc -l` | 3.753 | 11.839 | 0.32x | 1.89 | 2.86 | 0.66x |
| `pipe` | dual-win | `printf '%s\n' ... | awk '{ print $1 }' | sort | uniq` | 3.594 | 17.009 | 0.21x | 1.89 | 2.84 | 0.66x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE` | 3.828 | 9.907 | 0.39x | 1.55 | 2.86 | 0.54x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | wc -l` | 5.003 | 14.046 | 0.36x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | head -n 50` | 5.009 | 14.751 | 0.34x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | tail -n 50` | 5.227 | 14.776 | 0.35x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort` | 4.737 | 14.488 | 0.33x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | uniq` | 4.871 | 17.832 | 0.27x | 1.55 | 2.83 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | uniq | wc -l` | 4.408 | 20.486 | 0.22x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | wc -l` | 4.475 | 18.597 | 0.24x | 1.55 | 2.86 | 0.54x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | head -n 50` | 5.410 | 16.972 | 0.32x | 1.55 | 2.80 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | tail -n 50` | 4.677 | 17.683 | 0.26x | 1.55 | 2.86 | 0.54x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | sort | xargs echo` | 4.935 | 24.382 | 0.20x | 1.55 | 2.94 | 0.53x |
| `pipe` | dual-win | `printf '%s\n' ... | grep NEEDLE | xargs echo` | 5.031 | 19.764 | 0.25x | 1.55 | 2.81 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | tr a-z A-Z` | 3.853 | 10.961 | 0.35x | 1.58 | 2.86 | 0.55x |
| `pipe` | dual-win | `printf '%s\n' ... | sort` | 4.230 | 10.905 | 0.39x | 1.69 | 2.80 | 0.60x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | uniq` | 4.278 | 13.758 | 0.31x | 1.70 | 2.80 | 0.61x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | uniq | wc -l` | 4.170 | 15.739 | 0.26x | 1.72 | 2.86 | 0.60x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | wc -l` | 3.317 | 12.499 | 0.27x | 1.70 | 2.86 | 0.60x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | head -n 50` | 3.823 | 12.141 | 0.31x | 1.72 | 2.83 | 0.61x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | tail -n 50` | 3.822 | 13.060 | 0.29x | 1.70 | 2.80 | 0.61x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | xargs echo` | 4.011 | 19.403 | 0.21x | 1.72 | 2.84 | 0.60x |
| `pipe` | dual-win | `printf '%s\n' ... | sort | xargs wc -l` | 36.549 | 74.917 | 0.49x | 2.14 | 3.98 | 0.54x |
| `pipe` | dual-win | `printf '%s\n' ... | xargs echo` | 3.601 | 15.065 | 0.24x | 1.55 | 2.92 | 0.53x |
| `pipe` | dual-win | `printf '%s\n' ... | xargs wc -l` | 39.033 | 69.008 | 0.57x | 1.84 | 3.98 | 0.46x |
| `pipe` | dual-win | `seq 1 200000 | wc -l` | 3.102 | 38.523 | 0.08x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `seq 1 200000 | head -n 50` | 3.541 | 7.091 | 0.50x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `seq 1 200000 | tail -n 50` | 2.518 | 60.242 | 0.04x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `seq 1 200000 | sort` | 42.833 | 112.427 | 0.38x | 9.70 | 15.42 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | uniq` | 44.031 | 150.780 | 0.29x | 9.73 | 15.36 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | uniq | wc -l` | 43.010 | 154.456 | 0.28x | 9.70 | 15.33 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | wc -l` | 20.915 | 116.331 | 0.18x | 9.70 | 15.39 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | head -n 50` | 41.846 | 90.523 | 0.46x | 9.70 | 15.38 | 0.63x |
| `pipe` | dual-win | `seq 1 200000 | sort | tail -n 50` | 41.859 | 140.890 | 0.30x | 9.70 | 15.39 | 0.63x |
| `pipe` | dual-win | `seq 1 5000 | sort | xargs echo` | 4.136 | 17.501 | 0.24x | 1.67 | 1.94 | 0.86x |
| `pipe` | dual-win | `seq 1 200000 | grep 199` | 15.109 | 58.200 | 0.26x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | wc -l` | 14.906 | 59.835 | 0.25x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | head -n 50` | 15.201 | 59.215 | 0.26x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | tail -n 50` | 14.614 | 59.537 | 0.25x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort` | 15.223 | 60.421 | 0.25x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | uniq` | 15.208 | 67.069 | 0.23x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | uniq | wc -l` | 15.579 | 66.738 | 0.23x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | wc -l` | 14.914 | 62.166 | 0.24x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | head -n 50` | 14.723 | 61.592 | 0.24x | 1.56 | 1.94 | 0.81x |
| `pipe` | dual-win | `seq 1 200000 | grep 199 | sort | tail -n 50` | 14.451 | 61.493 | 0.24x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `seq 1 5000 | grep 199 | sort | xargs echo` | 3.219 | 14.412 | 0.22x | 1.39 | 1.94 | 0.72x |
| `pipe` | dual-win | `seq 1 5000 | grep 199 | xargs echo` | 3.111 | 11.678 | 0.27x | 1.39 | 1.94 | 0.72x |
| `pipe` | dual-win | `seq 1 5000 | xargs echo` | 2.788 | 12.359 | 0.23x | 1.38 | 1.94 | 0.71x |
| `pipe` | dual-win | `yes READY | head -n 20000` | 3.029 | 6.618 | 0.46x | 1.36 | 1.94 | 0.70x |
| `pipe` | takeover | `which ... | wc -l` | 2.908 | 5.337 | 0.54x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which ... | head -n 1` | 2.933 | 4.807 | 0.61x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which ... | tail -n 1` | 2.533 | 4.933 | 0.51x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which ... | grep / | wc -l` | 2.516 | 7.154 | 0.35x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which ... | xargs echo` | 2.781 | 7.580 | 0.37x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `which ... | sort | wc -l` | 2.886 | 7.314 | 0.39x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `which ... | sort | xargs echo` | 2.912 | 10.061 | 0.29x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `which -a ... | wc -l` | 2.782 | 5.302 | 0.52x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which -a ... | xargs echo` | 2.740 | 7.819 | 0.35x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `which -a ... | sort | xargs echo` | 2.920 | 9.969 | 0.29x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `command -v ... | wc -l` | 2.498 | 4.147 | 0.60x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `command -v ... | head -n 1` | 2.847 | 3.459 | 0.82x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `command -v ... | tail -n 1` | 2.498 | 3.476 | 0.72x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `command -v ... | grep / | wc -l` | 2.507 | 5.784 | 0.43x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `command -v ... | xargs echo` | 2.781 | 6.677 | 0.42x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `command -v ... | sort | wc -l` | 2.791 | 6.121 | 0.46x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `command -v ... | sort | xargs echo` | 2.943 | 8.688 | 0.34x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `printenv PATH | wc -l` | 2.492 | 5.089 | 0.49x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `printenv PATH | grep /` | 2.514 | 5.247 | 0.48x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `printenv PATH | grep / | wc -l` | 2.573 | 7.550 | 0.34x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `printenv PATH | xargs echo` | 3.576 | 9.339 | 0.38x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `printenv PATH | sort | xargs echo` | 3.763 | 11.746 | 0.32x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `true | wc -l` | 2.791 | 3.997 | 0.70x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `false | wc -l` | 2.673 | 3.853 | 0.69x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `false | grep NEEDLE | wc -l` | 2.930 | 6.144 | 0.48x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `true | xargs echo` | 2.802 | 4.348 | 0.64x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `mkdir -p ... | wc -l` | 3.201 | 5.376 | 0.60x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `mkdir -p ... | xargs echo` | 3.403 | 6.093 | 0.56x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `touch ... | wc -l` | 3.346 | 5.479 | 0.61x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `touch ... | sort | xargs echo` | 3.003 | 7.355 | 0.41x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `test -f ... | wc -l` | 2.769 | 4.468 | 0.62x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `test ! -e ... | xargs echo` | 3.150 | 4.855 | 0.65x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `[ -d ... ] | sort | xargs echo` | 2.838 | 6.511 | 0.44x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `test -d ... | grep <literal> | wc -l` | 3.037 | 6.301 | 0.48x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `wc -l ... | xargs echo` | 2.547 | 7.061 | 0.36x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `wc -c ... ... | wc -l` | 2.690 | 4.829 | 0.56x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `wc -l ... ... | grep <literal> | wc -l` | 2.550 | 7.595 | 0.34x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `wc -w ... | sort | xargs echo` | 2.638 | 9.619 | 0.27x | 1.41 | 1.94 | 0.73x |
| `pipe` | takeover | `wc -l | wc -l` | 7.917 | 11.695 | 0.68x | 1.41 | 2.42 | 0.58x |
| `pipe` | takeover | `wc -w | grep <literal> | wc -l` | 15.336 | 24.146 | 0.64x | 1.41 | 2.42 | 0.58x |
| `pipe` | takeover | `wc -l | sort | xargs echo` | 7.344 | 16.604 | 0.44x | 1.41 | 2.42 | 0.58x |
| `pipe` | takeover | `printf <literal> | wc -l` | 2.536 | 3.548 | 0.71x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `printf <literal> | grep <literal> | wc -l` | 2.481 | 6.098 | 0.41x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `printf <literal> | sort | xargs echo` | 2.759 | 8.630 | 0.32x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `du -sk ... | wc -l` | 8.183 | 12.791 | 0.64x | 1.48 | 1.94 | 0.77x |
| `pipe` | takeover | `du -sk ... | xargs echo` | 8.448 | 13.405 | 0.63x | 1.48 | 1.94 | 0.77x |
| `pipe` | takeover | `du -sk ... | grep <literal> | wc -l` | 8.268 | 13.463 | 0.61x | 1.48 | 1.94 | 0.77x |
| `pipe` | takeover | `hostname | wc -l` | 2.640 | 5.160 | 0.51x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | head -n 1` | 2.555 | 4.660 | 0.55x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | tail -n 1` | 2.511 | 4.819 | 0.52x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | grep <literal> | wc -l` | 2.489 | 7.284 | 0.34x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | sort` | 2.626 | 5.141 | 0.51x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `hostname | xargs echo` | 3.832 | 9.596 | 0.40x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `hostname | sort | xargs echo` | 4.951 | 12.321 | 0.40x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `ls -1 ... | wc -l` | 24.578 | 93.775 | 0.26x | 2.00 | 4.16 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | head -n 50` | 25.484 | 88.518 | 0.29x | 1.97 | 4.17 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | tail -n 50` | 26.159 | 99.196 | 0.26x | 1.97 | 4.17 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort` | 28.677 | 101.536 | 0.28x | 1.97 | 4.16 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort | uniq` | 27.738 | 109.440 | 0.25x | 2.00 | 4.19 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | sort | uniq | wc -l` | 19.586 | 108.003 | 0.18x | 1.97 | 4.16 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort | wc -l` | 27.286 | 103.354 | 0.26x | 1.97 | 4.16 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort | head -n 50` | 23.979 | 98.686 | 0.24x | 1.97 | 4.16 | 0.47x |
| `pipe` | dual-win | `ls -1 ... | sort | tail -n 50` | 20.146 | 105.043 | 0.19x | 1.97 | 4.17 | 0.47x |
| `pipe` | takeover | `ls -1 ... | sort | xargs echo` | 2.777 | 9.668 | 0.29x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `ls -1 ... | grep file-19` | 18.241 | 93.322 | 0.20x | 2.00 | 4.17 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | grep file-19 | wc -l` | 18.742 | 94.033 | 0.20x | 1.97 | 4.14 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | grep file-19 | xargs echo` | 25.210 | 102.625 | 0.25x | 1.98 | 4.17 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | grep file-19 | sort | xargs echo` | 22.252 | 104.589 | 0.21x | 1.98 | 4.17 | 0.48x |
| `pipe` | dual-win | `ls -1 ... | xargs echo` | 28.453 | 148.660 | 0.19x | 2.03 | 4.20 | 0.48x |
| `pipe` | takeover | `ls -a ... | wc -l` | 3.884 | 7.660 | 0.51x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -a ... | grep hidden | wc -l` | 3.521 | 9.626 | 0.37x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -a ... | sort | tail -n 1` | 3.443 | 8.929 | 0.39x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `ls -a ... | sort | xargs echo` | 2.587 | 9.234 | 0.28x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -a ... | xargs echo` | 4.042 | 9.920 | 0.41x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -A ... | wc -l` | 4.216 | 6.718 | 0.63x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -A ... | grep hidden | wc -l` | 4.667 | 12.463 | 0.37x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -A ... | sort | tail -n 1` | 5.189 | 8.678 | 0.60x | 1.44 | 1.94 | 0.74x |
| `pipe` | takeover | `ls -A ... | sort | xargs echo` | 2.954 | 9.370 | 0.32x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `ls -A ... | xargs echo` | 4.663 | 10.720 | 0.43x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `sort ... | uniq` | 15.362 | 344.213 | 0.04x | 16.88 | 49.50 | 0.34x |
| `pipe` | dual-win | `sort ... | uniq | wc -l` | 13.558 | 348.138 | 0.04x | 16.88 | 49.52 | 0.34x |
| `pipe` | dual-win | `sort ... | head -n 50` | 9.500 | 151.359 | 0.06x | 16.88 | 49.50 | 0.34x |
| `pipe` | dual-win | `sort ... | tail -n 50` | 9.443 | 335.503 | 0.03x | 16.89 | 49.50 | 0.34x |
| `pipe` | dual-win | `sort ... | wc -l` | 9.442 | 232.446 | 0.04x | 16.88 | 49.50 | 0.34x |
| `pipe` | dual-win | `sort ... | xargs echo` | 2.858 | 16.962 | 0.17x | 1.67 | 2.03 | 0.82x |
| `pipe` | dual-win | `sort ... | xargs wc -l` | 33.512 | 56.826 | 0.59x | 1.67 | 2.20 | 0.76x |
| `pipe` | dual-win | `head -n 50000 | wc -l` over stdin | 3.256 | 8.490 | 0.38x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `tail -n 50000 | wc -l` over stdin | 5.558 | 8.309 | 0.67x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `cat ... | head -n 50` | 2.803 | 5.543 | 0.51x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `cat ... | tail -n 50` | 6.848 | 34.381 | 0.20x | 1.39 | 1.97 | 0.71x |
| `pipe` | dual-win | `cat ... | grep NEEDLE` | 7.300 | 15.668 | 0.47x | 1.36 | 1.94 | 0.70x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | wc -l` | 7.306 | 16.938 | 0.43x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | head -n 50` | 7.386 | 16.886 | 0.44x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | tail -n 50` | 6.846 | 16.505 | 0.41x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort` | 7.067 | 17.565 | 0.40x | 1.56 | 1.94 | 0.81x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | uniq` | 7.391 | 20.157 | 0.37x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | uniq | wc -l` | 7.197 | 21.675 | 0.33x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | wc -l` | 7.012 | 19.925 | 0.35x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | head -n 50` | 7.305 | 19.450 | 0.38x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | tail -n 50` | 7.221 | 19.895 | 0.36x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | xargs echo` | 7.337 | 22.473 | 0.33x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `cat ... | grep count-19 | xargs wc -l` | 4.727 | 13.110 | 0.36x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `cat ... | grep NEEDLE | sort | xargs echo` | 7.183 | 24.595 | 0.29x | 1.59 | 1.94 | 0.82x |
| `pipe` | dual-win | `cat ... | grep count-19 | sort | xargs wc -l` | 4.658 | 15.267 | 0.31x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `cut -d, -f1 | wc -l` over stdin | 10.824 | 112.324 | 0.10x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `cat ... | cut -d, -f1` | 15.425 | 108.714 | 0.14x | 1.34 | 1.94 | 0.69x |
| `pipe` | dual-win | `cat ... | tr a-z A-Z` | 10.612 | 383.455 | 0.03x | 1.34 | 1.94 | 0.69x |
| `pipe` | dual-win | `cat ... | xargs echo` | 2.826 | 13.957 | 0.20x | 1.48 | 1.94 | 0.77x |
| `pipe` | dual-win | `cat ... | xargs wc -l` | 33.783 | 51.624 | 0.65x | 1.59 | 1.94 | 0.82x |
| `pipe` | dual-win | `cat ... | uniq` | 32.763 | 121.824 | 0.27x | 1.42 | 1.94 | 0.73x |
| `pipe` | takeover | `cat ... | uniq | wc -l` | 45.684 | 134.298 | 0.34x | 18.86 | 1.97 | 9.58x |
| `pipe` | dual-win | `cat ... | sort` | 13.384 | 220.348 | 0.06x | 16.88 | 49.59 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | uniq` | 16.733 | 342.942 | 0.05x | 16.88 | 49.56 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | uniq | wc -l` | 13.424 | 350.750 | 0.04x | 16.89 | 49.59 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | wc -l` | 10.531 | 234.342 | 0.04x | 16.89 | 49.59 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | head -n 50` | 10.596 | 149.660 | 0.07x | 16.88 | 49.56 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | tail -n 50` | 10.353 | 337.075 | 0.03x | 16.88 | 49.58 | 0.34x |
| `pipe` | dual-win | `cat ... | sort | xargs echo` | 2.912 | 18.589 | 0.16x | 1.66 | 2.09 | 0.79x |
| `pipe` | dual-win | `cat ... | sort | xargs wc -l` | 33.434 | 58.787 | 0.57x | 1.67 | 2.27 | 0.74x |
| `pipe` | dual-win | `grep -R NEEDLE ... | head -n 50` | 4.663 | 25.156 | 0.19x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `grep -R NEEDLE ... | tail -n 50` | 22.479 | 52.241 | 0.43x | 1.48 | 1.94 | 0.77x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort` | 22.050 | 53.750 | 0.41x | 1.61 | 1.94 | 0.83x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | uniq` | 22.500 | 56.823 | 0.40x | 1.58 | 1.94 | 0.81x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | uniq | wc -l` | 22.038 | 59.772 | 0.37x | 1.59 | 1.94 | 0.82x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | wc -l` | 22.015 | 55.861 | 0.39x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | head -n 50` | 22.551 | 47.324 | 0.48x | 1.64 | 1.94 | 0.85x |
| `pipe` | dual-win | `grep -R NEEDLE ... | sort | tail -n 50` | 22.250 | 48.293 | 0.46x | 1.59 | 1.94 | 0.82x |
| `pipe` | dual-win | `grep -R NEEDLE ... | wc -l` | 21.547 | 40.901 | 0.53x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `grep NEEDLE ... | wc -l` | 7.401 | 14.831 | 0.50x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `grep NEEDLE ... | head -n 50` | 7.051 | 14.179 | 0.50x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep NEEDLE ... | tail -n 50` | 7.307 | 15.083 | 0.48x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `grep NEEDLE ... | sort` | 7.201 | 15.848 | 0.45x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | uniq` | 7.223 | 18.015 | 0.40x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | uniq | wc -l` | 7.161 | 20.507 | 0.35x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | wc -l` | 7.293 | 17.339 | 0.42x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | head -n 50` | 7.623 | 17.775 | 0.43x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | tail -n 50` | 7.336 | 17.863 | 0.41x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `grep NEEDLE ... | xargs echo` | 7.093 | 20.355 | 0.35x | 1.56 | 1.94 | 0.81x |
| `pipe` | dual-win | `grep count- ... | xargs wc -l` | 34.063 | 53.475 | 0.64x | 1.69 | 1.94 | 0.87x |
| `pipe` | dual-win | `grep NEEDLE ... | sort | xargs echo` | 7.267 | 23.324 | 0.31x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `grep count- ... | sort | xargs wc -l` | 34.856 | 59.008 | 0.59x | 1.70 | 2.22 | 0.77x |
| `pipe` | takeover | `awk '{ print $1 }' | wc -l` over stdin | 10.696 | 57.294 | 0.19x | 5.30 | 1.94 | 2.73x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | wc -l` | 6.700 | 59.661 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | head -n 50` | 6.838 | 59.712 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | tail -n 50` | 6.808 | 60.087 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort` | 6.633 | 60.458 | 0.11x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | uniq` | 6.735 | 62.373 | 0.11x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | uniq | wc -l` | 6.922 | 66.190 | 0.10x | 1.53 | 1.94 | 0.79x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | wc -l` | 7.215 | 64.000 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | head -n 50` | 7.139 | 64.123 | 0.11x | 1.55 | 1.94 | 0.80x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | tail -n 50` | 7.183 | 64.326 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | xargs echo` | 7.035 | 65.131 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | xargs wc -l` | 34.615 | 57.144 | 0.61x | 1.69 | 1.94 | 0.87x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | xargs echo` | 7.108 | 67.527 | 0.11x | 1.52 | 1.94 | 0.78x |
| `pipe` | dual-win | `awk '/NEEDLE/ { print $1 }' ... | sort | xargs wc -l` | 34.388 | 62.972 | 0.55x | 1.69 | 2.23 | 0.76x |
| `pipe` | dual-win | `find ... -type f | xargs wc -l` | 87.442 | 128.775 | 0.68x | 1.45 | 1.94 | 0.75x |
| `pipe` | dual-win | `find ... -type f | xargs echo` | 8.080 | 36.945 | 0.22x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `find ... -type f | xargs` | 8.500 | 41.883 | 0.20x | 1.48 | 1.94 | 0.77x |
| `pipe` | dual-win | `find ... -type f | wc -l` | 7.143 | 14.792 | 0.48x | 1.44 | 1.94 | 0.74x |
| `pipe` | dual-win | `find ... -type f | head -n 50` | 2.549 | 6.096 | 0.42x | 1.41 | 1.94 | 0.73x |
| `pipe` | dual-win | `find ... -type f | tail -n 50` | 7.317 | 22.077 | 0.33x | 1.47 | 1.94 | 0.76x |
| `pipe` | dual-win | `find ... -type f | sort | wc -l` | 7.716 | 45.939 | 0.17x | 2.05 | 3.44 | 0.60x |
| `pipe` | takeover | `find ... -maxdepth 1 -type f | wc -l` | 2.600 | 5.277 | 0.49x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 1 -type f | head -n 5` | 2.556 | 5.355 | 0.48x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 1 -type f | grep <literal> | wc -l` | 2.596 | 7.274 | 0.36x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 1 -type f | xargs echo` | 2.583 | 8.025 | 0.32x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 2 -type f | sort | tail -n 1` | 2.588 | 7.696 | 0.34x | 1.45 | 1.94 | 0.75x |
| `pipe` | takeover | `find ... -maxdepth 2 -type f -name '*.rs' | grep <literal> | wc -l` | 2.641 | 7.268 | 0.36x | 1.47 | 1.94 | 0.76x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | xargs wc -l` | 66.926 | 91.402 | 0.73x | 1.42 | 1.94 | 0.73x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | xargs echo` | 10.230 | 34.847 | 0.29x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | xargs` | 7.426 | 25.468 | 0.29x | 1.48 | 1.94 | 0.77x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | grep item-019 | xargs echo` | 7.616 | 21.159 | 0.36x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | grep item-019 | xargs wc -l` | 8.225 | 22.808 | 0.36x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | grep item-019 | sort | xargs echo` | 9.827 | 25.670 | 0.38x | 1.67 | 1.94 | 0.86x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | grep item-019 | sort | xargs wc -l` | 10.066 | 27.874 | 0.36x | 1.66 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | wc -l` | 9.597 | 21.824 | 0.44x | 1.42 | 1.97 | 0.72x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | head -n 50` | 4.931 | 11.599 | 0.43x | 1.41 | 1.97 | 0.71x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | tail -n 50` | 10.224 | 25.794 | 0.40x | 1.64 | 1.94 | 0.85x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort` | 10.502 | 33.425 | 0.31x | 1.67 | 2.16 | 0.78x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | uniq` | 9.458 | 38.999 | 0.24x | 1.66 | 2.11 | 0.79x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | uniq | wc -l` | 13.044 | 43.764 | 0.30x | 1.67 | 2.09 | 0.80x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | wc -l` | 10.511 | 34.418 | 0.31x | 1.64 | 2.09 | 0.78x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | xargs echo` | 9.709 | 46.778 | 0.21x | 1.64 | 2.09 | 0.78x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | xargs wc -l` | 61.265 | 101.944 | 0.60x | 1.64 | 2.12 | 0.77x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | head -n 50` | 10.068 | 35.132 | 0.29x | 1.66 | 2.12 | 0.78x |
| `pipe` | dual-win | `find ... -type f -name '*.rs' | sort | tail -n 50` | 10.143 | 41.349 | 0.25x | 1.67 | 2.12 | 0.79x |

Interpretation:

- `test`/`[` plus `mkdir` and `touch` are safe takeover primitives even though
  micro-benchmarks can lose once the public cap wrapper path is included.
- The exact `awk` count and fixed-field subsets, including direct stdin forms,
  default `xargs` echo, `xargs echo`, `xargs wc -l`, stdin
  `xargs echo | wc -l`, stdin `xargs echo | grep ... | wc -l`, and
  `printenv NAME | grep ... | wc -l` now stay in the C fast path and beat the
  original on both CPU and RSS for the measured rows.
- The listed pipe shapes are fused as whole command strings inside `cap run`;
  unsupported pipe shapes still fall back to `bash -c`.

## Meter Probe

`meter profile` can attach to the focused benchmark outside the sandbox, but the
current sample is not yet actionable at the source level:

```bash
CAP_BENCH_COMMANDS=echo,printf,seq,whoami,id,uname,mkdir,touch,cut,tr,awk,xargs,pipe \
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

- successful stdout and exit-code parity for `true`, `false`, `pwd`,
  `echo`, `printf`, `seq`, `whoami`, narrow `id`, narrow `uname`, `hostname`,
  narrow `test`, bracket predicates, `basename`, `dirname`, `ls`, `cat`, `head`, `tail`,
  `mkdir`, `touch`, `uniq`, `find`, `du`, `sort`, `cut`, `tr`, `sed`, `grep`, `awk`, `xargs echo`,
  `xargs wc -l`, `which`, `command -v`, `env`, `printenv`, and `wc -l`;
- `cap run "<simple active command>"` parity for the same active replacements,
  using the installed `cap` plus `cap-fast` plus `cap-full` binary shape;
- fused `cap run` pipe parity for `cat ... | wc -l`,
  `echo ... | wc -l`, `echo ... | head -n`, `echo ... | tail -n`,
  `echo ... | tr`, `echo ... | xargs echo`, `echo ... | xargs wc -l`,
  `printf ... | wc -l`, `printf ... | head -n`, `printf ... | tail -n`, `printf ... | grep`,
  `printf ... | grep | wc -l`, `printf ... | grep | head -n`,
  `printf ... | grep | tail -n`, `printf ... | grep | sort`,
  `printf ... | grep | sort | uniq`, `printf ... | grep | sort | uniq | wc -l`,
  `printf ... | grep | sort | wc -l`, `printf ... | grep | sort | head -n`,
  `printf ... | grep | sort | tail -n`, `printf ... | grep | sort | xargs echo`,
  `printf ... | grep | sort | uniq | head/tail/sort/xargs echo/xargs wc -l`,
  `printf ... | grep | xargs echo`,
  `printf ... | tr`, `printf ... | sort`, `printf ... | sort | uniq`,
  `printf ... | sort | uniq | wc -l`, `printf ... | sort | wc -l`,
  `printf ... | sort | head -n`, `printf ... | sort | tail -n`,
  `printf ... | sort | xargs echo`, `printf ... | sort | xargs wc -l`,
  `printf ... | sort | uniq | head/tail/sort/xargs echo/xargs wc -l`,
  `printf ... | xargs echo`, `printf ... | xargs wc -l`,
  `seq ... | wc -l`, `seq ... | head -n`, `seq ... | tail -n`,
  `seq ... | sort`, `seq ... | sort | uniq`, `seq ... | sort | uniq | wc -l`,
  `seq ... | sort | wc -l`, `seq ... | sort | head -n`,
  `seq ... | sort | tail -n`, `seq ... | sort | xargs echo`,
  `seq ... | sort | uniq | head/tail/sort/xargs echo/xargs wc -l`,
  `seq ... | grep`, `seq ... | grep | wc -l`, `seq ... | grep | head -n`,
  `seq ... | grep | tail -n`, `seq ... | grep | sort`,
  `seq ... | grep | sort | uniq`, `seq ... | grep | sort | uniq | wc -l`,
  `seq ... | grep | sort | wc -l`, `seq ... | grep | sort | head -n`,
  `seq ... | grep | sort | tail -n`, `seq ... | grep | sort | xargs echo`,
  `seq ... | grep | sort | uniq | head/tail/sort/xargs echo/xargs wc -l`,
  `seq ... | grep | xargs echo`,
  `seq ... | xargs echo`,
  `yes ... | head -n`,
  `ls ... | wc -l`, `ls ... | head -n`, `ls ... | tail -n`,
  `ls ... | sort`, `ls ... | sort | uniq`, `ls ... | sort | uniq | wc -l`,
  `ls ... | sort | uniq | head/tail/sort/xargs echo`,
  `ls ... | sort | uniq | grep | wc/head/tail/sort/xargs echo`,
  `ls ... | sort | wc -l`, `ls ... | sort | head -n`, `ls ... | sort | tail -n`,
  `ls ... | grep`, `ls ... | grep | wc -l`, `ls ... | grep | head/tail/sort/sort|uniq`,
  `ls ... | grep | xargs echo`, `ls ... | grep | sort | xargs echo`, `ls ... | xargs echo`,
  `sort ... | uniq`, `sort ... | uniq | wc -l`,
  `sort ... | uniq | head/tail/sort/xargs`, `sort ... | uniq | grep | wc/head/tail/sort/xargs`,
  `sort ... | grep | wc/head/tail/sort/xargs`,
  `sort ... | head -n`, `sort ... | tail -n`, `sort ... | wc -l`, `sort ... | xargs wc -l`,
  `sort ... | xargs wc -l | sort/head/tail`,
  `head ... | wc -l`, `head ... | head -n`, `head ... | tail -n`,
  `head ... | sort`, `head ... | sort | uniq`, `head ... | sort | uniq | wc -l`,
  `head ... | sort | wc -l`, `head ... | sort | head -n`, `head ... | sort | tail -n`,
  `head ... | xargs echo`, `head ... | xargs wc -l`, `head ... | sort | xargs echo`,
  `head ... | sort | xargs wc -l`, `head ... | grep`, `head ... | grep | wc -l`,
  `head ... | grep | head -n`, `head ... | grep | tail -n`,
  `head ... | grep | sort`, `head ... | grep | sort | uniq`,
  `head ... | grep | sort | uniq | wc -l`, `head ... | grep | sort | wc -l`,
  `head ... | grep | sort | head -n`, `head ... | grep | sort | tail -n`,
  `head ... | grep | xargs echo`, `head ... | grep | xargs wc -l`,
  `head ... | grep | sort | xargs echo`, `head ... | grep | sort | xargs wc -l`,
  `tail ... | wc -l`, `tail ... | head -n`, `tail ... | tail -n`,
  `tail ... | sort`, `tail ... | sort | uniq`, `tail ... | sort | uniq | wc -l`,
  `tail ... | sort | wc -l`, `tail ... | sort | head -n`, `tail ... | sort | tail -n`,
  `tail ... | xargs echo`, `tail ... | xargs wc -l`, `tail ... | sort | xargs echo`,
  `tail ... | sort | xargs wc -l`, `tail ... | grep`, `tail ... | grep | wc -l`,
  `tail ... | grep | head -n`, `tail ... | grep | tail -n`,
  `tail ... | grep | sort`, `tail ... | grep | sort | uniq`,
  `tail ... | grep | sort | uniq | wc -l`, `tail ... | grep | sort | wc -l`,
  `tail ... | grep | sort | head -n`, `tail ... | grep | sort | tail -n`,
  `tail ... | grep | xargs echo`, `tail ... | grep | xargs wc -l`,
  `tail ... | grep | sort | xargs echo`, `tail ... | grep | sort | xargs wc -l`,
  `cat ... | head -n`, `cat ... | tail -n`, `cat ... | grep`,
  `cat ... | grep | wc -l`, `cat ... | grep | head -n`, `cat ... | grep | tail -n`,
  `cat ... | grep | sort`, `cat ... | grep | sort | uniq`,
  `cat ... | grep | sort | uniq | wc -l`, `cat ... | grep | sort | wc -l`,
  `cat ... | grep | sort | uniq | head/tail/sort/xargs echo`,
  `cat ... | grep | sort | head -n`, `cat ... | grep | sort | tail -n`,
  `cat ... | grep | xargs echo`, `cat ... | grep | xargs wc -l`,
  `cat ... | grep | sort | xargs echo`, `cat ... | grep | sort | xargs wc -l`,
  `cat ... | cut -d <char> -f <field>`,
  `cat ... | cut -d <char> -f <field> | wc/head/tail/sort/xargs`,
  `cat ... | cut -d <char> -f <field> | grep | wc/head/tail/sort/xargs`,
  `cat ... | tr`,
  `cat ... | tr | wc/head/tail/sort/xargs`,
  `cat ... | tr | grep | wc/head/tail/sort/xargs`,
  `cat ... | xargs echo`, `cat ... | xargs wc -l`, `cat ... | xargs wc -l | sort`,
  `sort ... | xargs echo`,
  `cat ... | uniq`, `cat ... | uniq | wc -l`,
  `cat ... | uniq | head/tail/sort/xargs`,
  `cat ... | uniq | grep | wc/head/tail/sort/xargs`,
  `uniq ... | wc/head/tail/sort/xargs`,
  `uniq ... | grep | wc/head/tail/sort/xargs`,
  `cat ... | sort`, `cat ... | sort | uniq`, `cat ... | sort | uniq | wc -l`,
  `cat ... | sort | grep | wc/head/tail/sort/xargs`, `cat ... | sort | wc -l`,
  `cat ... | sort | head -n`, `cat ... | sort | tail -n`, `cat ... | sort | xargs echo`, `cat ... | sort | xargs wc -l`,
  `cat ... | sort | xargs wc -l | sort`,
  `grep ... | wc -l`, `grep ... | head -n`, `grep ... | tail -n`,
  `grep ... | sort`, `grep ... | sort | uniq`, `grep ... | sort | uniq | wc -l`,
  `grep ... | sort | wc -l`, `grep ... | sort | head -n`,
  `grep ... | sort | tail -n`,
  `grep ... | xargs echo`, `grep ... | xargs wc -l`,
  `grep ... | xargs wc -l | sort`, `grep ... | sort | xargs echo`,
  `grep ... | sort | xargs wc -l`, `grep ... | sort | xargs wc -l | sort | tail`,
  `grep -R ... | head -n`, `grep -R ... | tail -n`, `grep -R ... | sort`,
  `grep -R ... | sort | uniq`, `grep -R ... | sort | uniq | wc -l`,
  `grep -R ... | sort | uniq | head/tail/sort/xargs echo`,
  `grep -R ... | sort | wc -l`, `grep -R ... | sort | head -n`,
  `grep -R ... | sort | tail -n`, `grep -R ... | wc -l`,
  unfiltered `awk '{ print $1 }' ...` producer pipes,
  `awk ... | wc -l`, `awk ... | head -n`, `awk ... | tail -n`,
  `awk ... | sort`, `awk ... | sort | uniq`, `awk ... | sort | uniq | wc -l`,
  `awk ... | sort | uniq | head/tail/sort/xargs echo/xargs wc -l`,
  `awk ... | sort | wc -l`, `awk ... | sort | head -n`,
  `awk ... | sort | tail -n`, `awk ... | xargs echo`,
  `awk ... | xargs wc -l`, `awk ... | xargs wc -l | sort`,
  `awk ... | sort | xargs echo`, `awk ... | sort | xargs wc -l`,
  `awk ... | sort | xargs wc -l | sort | tail`, `awk ... | grep | wc/sort/xargs`,
  `cat ... | awk ... | grep | tail/sort/xargs`, `which ... | wc -l`, `which ... | head -n`,
  `which ... | tail -n`, `which ... | grep | wc/head/tail/sort/xargs`,
  `which -a ... | wc/xargs echo/sort|xargs echo`,
  `command -v ... | wc -l`,
  `command -v ... | head -n`, `command -v ... | tail -n`,
  `command -v ... | grep | wc/head/tail/sort/xargs`,
  `printenv ... | wc -l`, `printenv ... | head -n`,
  `printenv ... | tail -n`, `printenv ... | grep`,
  `printenv ... | grep | wc/head/tail/sort/xargs`, `printenv ... | sort`,
  `true|false | wc/head/tail/sort/xargs`, `true|false | grep | wc/head/tail/sort/xargs`,
  `hostname | wc -l`, `hostname | head -n`, `hostname | tail -n`,
  `hostname | grep`, `hostname | grep | wc/head/tail/sort/xargs`, `hostname | sort`,
  `find ... | xargs wc -l`, `find ... | xargs wc -l | wc/head/tail/sort/sort|uniq`,
  `find ... | xargs echo`, `find ... | xargs`,
  `find ... | grep | wc/head/tail/sort/sort|uniq`,
  `find ... | grep | xargs echo`, `find ... | grep | xargs wc -l`,
  `find ... | grep | sort | xargs echo`, `find ... | grep | sort | xargs wc -l`,
  `find ... | grep | sort | xargs wc -l | sort`,
  `find ... | wc -l`, `find ... | head -n`, `find ... | tail -n`,
  `find ... -maxdepth <positive> | wc/head/sort/grep/xargs`,
  `find ... | sort`, `find ... | sort | uniq`, `find ... | sort | uniq | wc -l`, `find ... | sort | wc -l`,
  `find ... | sort | uniq | head/tail/sort/xargs echo/xargs wc -l/xargs wc -l | sort`,
  `find ... | sort | uniq | grep | wc/head/tail/sort/xargs echo/xargs wc -l`,
  `find ... | sort | xargs echo`, `find ... | sort | xargs wc -l`,
  `find ... | sort | xargs wc -l | sort/head/tail`, and
  `find ... | sort | head -n`, `find ... | sort | tail -n`;
- upstream-error pipeline parity for the fused `cat`, `grep`, and `find`
  shapes where Bash reports the first stage on stderr but returns the last
  stage's exit status;
- missing-path nonzero exit behavior and stderr diagnostics;
- quiet nonzero behavior for recursive `grep` no-match.

This test caught and fixed the `du -sk <missing>` case where cap printed a
synthetic `0<TAB>path` summary while the original command only reported the
error.

## Interpretation

- Dual-win replacements in this baseline: `ls`, `cat`, `uniq`,
  `find`, `du`, `sort`, `sed`, stdin/single-file and recursive `grep`, `wc -l`, narrow `awk`, narrow `xargs`,
  and the listed fused pipe shapes.
- CPU-win replacements in this baseline: narrow streaming `tr`, including exact
  `[:lower:]`/`[:upper:]`/`[:digit:]` class translate/delete forms.
- RSS-fallback replacements in this baseline: none.
- Default takeover rows in this baseline: `true`, `false`, `pwd`, `basename`,
  `dirname`, `echo`, narrow `printf`, narrow `seq`, `whoami`, narrow `id` including default summary and group lists,
  narrow `uname`, `hostname`, narrow `test` and bracket predicates, `head`,
  `tail`, `mkdir`, `touch`, default `wc`, narrow `cut`, `which`, `which -a`, `command -v`, direct `env`/`printenv`,
  lookup pipe shapes, path-lookup grep producer pipe shapes, producer `xargs echo` pipe shapes, producer `xargs wc -l` pipe shapes, producer `xargs wc -l | ...` output downstream shapes, head producer pipe shapes, tail producer pipe shapes, cat-head/tail producer pipe shapes, single-line producer pipe shapes, sed producer pipe shapes, cat-sed producer pipe shapes, cat-awk producer pipe shapes, cut producer pipe shapes, cat-cut producer pipe shapes, cat-tr producer pipe shapes, cat-uniq producer pipe shapes, sort-uniq producer pipe shapes, ls-sort-uniq producer pipe shapes, find-sort-uniq producer pipe shapes, find-maxdepth-positive pipe shapes, printf-sort-uniq producer pipe shapes, printf-grep-sort-uniq producer pipe shapes, seq-sort-uniq producer pipe shapes, seq-grep-sort-uniq producer pipe shapes, awk-sort-uniq producer pipe shapes, awk-grep producer pipe shapes, grep-r-sort-uniq producer pipe shapes, cat-grep-sort-uniq producer pipe shapes, grep-file-sort-uniq producer pipe shapes, `printf|grep|wc`, `printf|grep|head`, `printf|grep|tail`, printf-grep sort/count/uniq pipe shapes, `printf|grep|xargs echo`, `printf|sort`, `printf|sort|uniq`, `printf|sort|uniq|wc`, `printf|sort|wc`, `printf|sort|head`, `printf|sort|tail`, `printf|sort|xargs echo`, `printf|sort|xargs wc -l`, literal printf producer pipe shapes, `seq|sort`, `seq|sort|uniq`, `seq|sort|uniq|wc`, `seq|sort|wc`, `seq|sort|head`, `seq|sort|tail`, `seq|sort|xargs echo`, `seq|grep`, `seq|grep|wc`, `seq|grep|head`, `seq|grep|tail`, seq-grep sort/count/uniq pipe shapes, `seq|grep|xargs echo`, `grep|xargs echo`/`grep|xargs wc -l`, `grep|xargs wc -l|sort`, `grep|sort|xargs echo`, `grep|sort|xargs wc -l`, `grep|sort|xargs wc -l|sort|tail`, `grep|sort|uniq|head`, `grep|sort|uniq|tail`, `grep|sort|uniq|sort|xargs echo`, `awk|wc`, `awk|head`, `awk|tail`, `awk|sort`, `awk|sort|uniq`, `awk|sort|uniq|wc`, `awk|sort|wc`, `awk|sort|head`, `awk|sort|tail`, `awk|grep|wc`, `awk|grep|sort|uniq|wc`, `awk|grep|xargs wc -l`, `awk|grep|xargs wc -l|sort`, `awk|xargs echo`, `awk|xargs wc -l`, `awk|xargs wc -l|sort`, `awk|sort|xargs echo`, `awk|sort|xargs wc -l`, `awk|sort|xargs wc -l|sort|tail`, `ls|grep|xargs echo`, `ls|grep|sort|xargs echo`, `ls|xargs echo`, `find|grep|xargs echo`, `find|grep|xargs wc -l`, `find|grep|sort|xargs echo`, `find|grep|sort|xargs wc -l`, `cat|grep|wc`, `cat|grep|head`, `cat|grep|tail`, `cat|grep|sort`, `cat|grep|sort|uniq`, `cat|grep|sort|uniq|wc`, `cat|grep|sort|wc`, `cat|grep|sort|head`, `cat|grep|sort|tail`, `cat|grep|sort|uniq|head`, `cat|grep|sort|uniq|tail`, `cat|grep|sort|uniq|sort|xargs echo`, `cat|grep|xargs echo`, `cat|grep|xargs wc -l`, `cat|grep|sort|xargs echo`, `cat|grep|sort|xargs wc -l`, `cat|awk|grep|tail`, `cat|awk|grep|sort|xargs wc -l`, `cat|awk|grep|sort|xargs wc -l|sort|tail`, `cat|awk|xargs wc -l|sort`, `cat|xargs echo`, `cat|xargs wc -l`, `cat|sort|xargs echo`, `cat|sort|xargs wc -l`, `sort|xargs echo`, `sort|xargs wc -l`, single-name `printenv` pipe shapes, single-name printenv-grep producer pipe shapes, empty primitive producer pipe shapes, side-effect empty producer pipe shapes, predicate empty producer pipe shapes, wc regular-file producer pipe shapes, wc stdin producer pipe shapes, literal printf producer pipe shapes, du-sk producer pipe shapes, hostname pipe shapes, and
  hostname-grep producer pipe shapes.
- Default takeover also includes grep-file-cut producer pipe shapes for
  `grep <literal> <file> | cut -d <char> -f <field>` and supported
  grep/count/head/tail/sort/xargs downstreams.
- Default takeover also includes grep-file-awk producer pipe shapes for
  `grep <literal> <file> | awk '{ print $1 }'` and supported
  grep/count/head/tail/sort/xargs downstreams.
- Default takeover also includes unfiltered awk fixed-field producer pipe
  shapes for `awk '{ print $<field> }' <file> | ...`, no-file
  `awk '{ print $<field> }' | ...` over stdin, and stdin-style
  `cat <file> | awk '{ print $<field> }' | ...` with supported
  count/head/tail/sort/xargs downstreams.
- Default takeover also includes awk-grep producer pipe shapes for
  `awk '{ print $1 }' <file> | grep <literal> | ...` and stdin-style
  `cat <file> | awk '{ print $1 }' | grep <literal> | ...` with supported
  count/head/tail/sort/xargs downstreams.
- Unsupported candidates in this baseline: option-sensitive `echo`, conversion
  and unsupported `printf` formats, general/floating-point `seq`, unsupported `id` flags,
  unsupported `uname` flags beyond `-s/-n/-r/-v/-m/-p/-a`, hostname flags/arguments, compound `test`/`[` expressions, general `awk`
  programs, xargs option forms outside the listed narrow subsets, general `cut` forms, general `tr` forms beyond exact
  `[:lower:]`/`[:upper:]`/`[:digit:]` translate/delete class tokens,
  `which` flags other than `-a`, general `command` builtin forms, `env` assignments/options,
  `printenv` flags/multiple names, full-environment pipes, cwd-sensitive
  `ls|xargs` consumers such as `ls <dir> | xargs wc -l`, `du` pipe forms outside
  `du -sk <existing-path> | ...`, multi-operand `yes` producers, and
  pipe-shaped shell commands outside the listed fused patterns.
- The public `cap` binary uses a no-startfiles syscall dispatcher on macOS
  arm64 with a sibling `cap-fast` helper for heavier replacements. The policy
  now prefers native takeover for safe shell-free subsets and keeps dual-win,
  CPU-win, and RSS-fallback rows as resource regression gates. The hook forwards all
  non-recursive Bash commands to `cap run '<original>'`; cap then
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
- `true`, `false`, `pwd`, `echo`, narrow `printf`, narrow `seq`, `whoami`,
  narrow `id` including group lists, narrow `uname`, narrow `test`/`[`, `basename`, `dirname`,
  `head`, `tail`, `mkdir`, `touch`, `which`, `which -a`, and `command -v` may still lose small CPU/RSS rows. They remain default native
  takeovers because the safe subset is simple and high-volume consistency is
  more valuable than avoiding a few milliseconds of wrapper overhead.
- Narrow `cut` direct rows are takeover rows: they are CPU-positive over file
  and stdin input, while this run measured RSS at 1.01x. Narrow `tr`, `awk`,
  `xargs echo`, `xargs wc -l`, producer `xargs echo` pipe shapes,
  xargs-grep stdin producer pipe shapes, producer `xargs wc -l` pipe shapes,
  head/tail producer pipe shapes, cat-head/tail
  producer pipe shapes, single-line producer pipe shapes, sed producer pipe
  shapes, cat-sed producer pipe shapes, cat-awk producer pipe shapes, cut
  producer pipe shapes including streaming `cut stdin | wc -l`, head/tail
  stdin producer `wc -l` pipe shapes, cat-cut
  producer pipe shapes, cat-tr producer pipe shapes, cat-uniq producer pipe
  shapes, sort-uniq producer pipe shapes, ls-sort-uniq producer pipe shapes,
  find-sort-uniq producer pipe shapes, printf-sort-uniq producer pipe shapes,
  printf-grep-sort-uniq producer pipe shapes, seq-sort-uniq producer pipe
  shapes, seq-grep-sort-uniq producer pipe shapes, awk-sort-uniq producer pipe
  shapes, grep-r-sort-uniq producer pipe shapes, cat-grep-sort-uniq producer
  pipe shapes, grep-file-sort-uniq producer pipe shapes, printf grep/count/sort/uniq pipe shapes, `printf|grep|xargs echo`, printf sort/count/uniq pipe shapes, `printf|sort|xargs echo`/`printf|sort|xargs wc -l`, literal printf producer pipe shapes, seq sort/count/uniq pipe shapes, `seq|sort|xargs echo`, seq grep/count/sort/uniq pipe shapes, `seq|grep|xargs echo`, `grep|xargs echo`/`grep|xargs wc -l`, `grep|sort|xargs echo`/`grep|sort|xargs wc -l`, `grep|sort|uniq` producer pipe shapes, `ls|grep|xargs echo`/`ls|grep|sort|xargs echo`, find-grep producer pipe shapes, `find|grep|xargs echo`/`find|grep|xargs wc -l`, `find|grep|sort|xargs echo`/`find|grep|sort|xargs wc -l`, `cat|grep|wc/head/tail`, cat-grep sort/count/uniq pipe shapes, `cat|grep|sort|uniq` producer pipe shapes, `cat|grep|xargs echo`/`cat|grep|xargs wc -l`, `cat|grep|sort|xargs echo`/`cat|grep|sort|xargs wc -l`, `awk|wc/head/tail`, awk sort/count/uniq pipe shapes, `awk|sort|uniq` producer pipe shapes, `awk|xargs echo`/`awk|xargs wc -l`, `awk|sort|xargs echo`/`awk|sort|xargs wc -l`, cat-awk producer pipe shapes, sort pipeline shapes, cat/sort `xargs echo`/`xargs wc -l` pipe shapes, and the
  listed pipe shapes win after moving out of `cap-full`/`bash -c` fallback and
  into the C fast path.
- grep-file-cut producer pipe shapes are measured in the same fused pipe family
  and avoid a second shell round trip between grep, cut, and supported
  downstream consumers.
- grep-file-awk producer pipe shapes are measured in the same fused pipe family
  and avoid a second shell round trip between grep, awk fixed-field extraction,
  and supported downstream consumers.
- Unfiltered awk fixed-field producer pipe shapes are measured in the same fused
  pipe family and avoid a second shell round trip between awk fixed-field
  extraction and supported downstream consumers.
- Awk-grep producer pipe shapes are measured in the same fused pipe family and
  avoid a second shell round trip between awk fixed-field extraction, literal
  grep filtering, and supported downstream consumers.
- Finite `echo`/`printf` awk fixed-field producer pipe shapes are measured in
  the same fused pipe family and avoid a second shell round trip between the
  finite producer, awk fixed-field extraction, and supported downstream
  consumers.
- `wc` stdin/regular-file and `du -sk` finite producer pipe shapes are measured
  in the same fused pipe family and avoid a second shell round trip between the
  producer row emission and supported downstream consumers.
- Literal `printf` finite producer pipe shapes are measured in the same fused
  pipe family and avoid a second shell round trip between literal byte emission,
  optional literal grep filtering, and supported downstream consumers.

This means benchmark data must gate future expansion beyond simple shell-free
primitives. For tiny commands, the fixed process footprint dominates memory
size; cap may still own the command when the safe subset is simple and parity is
strong.

## Retired Tiny Command Experiments

Focused experiments on 2026-06-13 show the strict dual-win / strict RSS-win
gap for `true` and `false` is dominated by macOS process floor rather than
command logic. They can reach RSS parity by `execve`ing Apple `/usr/bin/true`
and `/usr/bin/false`, but a stable strict RSS win has not been found. `pwd`
and `dirname` can become tiny RSS wins by `execve`ing Apple `/usr/bin/true`
when stdout is discarded, but the win is too small to matter for the takeover
policy:

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

The conclusion is not to chase tiny-command RSS wins as a promotion gate. These
commands can stay native on parity grounds; future work should focus benchmark
strictness on larger subsets, `awk`/`xargs`, and pipeline fusion.
