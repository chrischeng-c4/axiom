---
id: expand-high-volume-native-command-coverage
summary: Expand cap native command coverage for safe shell-free command shapes with parity tests and benchmark evidence.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: partial
    rationale: "Same-name command replacement changes how cap admits and runs wrapped commands while preserving original fallback behavior for unsupported shapes."
  - id: command-lease-throttling
    role: primary
    gap: memory-and-cpu-pressure-sampling
    claim: memory-and-cpu-pressure-sampling
    coverage: partial
    rationale: "Large-workload benchmark rows remain the regression signal for resource use, while safe shell-free subsets route native at any size."
---

# Expand Native Command Coverage For Safe Shapes

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-native-command-coverage-safe-shape-logic
entry: start
nodes:
  start: { kind: start, label: "cap argv or shell-free cap run command" }
  classify: { kind: decision, label: "matches safe same-name subset?" }
  fallback_shape: { kind: terminal, label: "original path: unsupported or shell-sensitive shape" }
  wc_shape: { kind: decision, label: "wc -l over regular files?" }
  probe_files: { kind: process, label: "verify regular-file safety" }
  native_wc: { kind: process, label: "run cap native line-count aggregate" }
  explain: { kind: process, label: "cap explain reports native_wc_lines vs original fallback" }
  parity: { kind: process, label: "parity tests cover stdout/stderr/exit behavior" }
  bench: { kind: process, label: "benchmark records large-workload CPU/RSS evidence" }
  done: { kind: terminal, label: "native safe subset with shell fallback boundary" }
edges:
  - { from: start, to: classify }
  - { from: classify, to: fallback_shape, label: "no" }
  - { from: classify, to: wc_shape, label: "yes" }
  - { from: wc_shape, to: fallback_shape, label: "no" }
  - { from: wc_shape, to: probe_files, label: "yes" }
  - { from: probe_files, to: native_wc }
  - { from: native_wc, to: explain }
  - { from: explain, to: parity }
  - { from: parity, to: bench }
  - { from: bench, to: done }
safe_subset:
  wc_lines:
    shape: "wc -l/-c/-w [FILE...] with no char/max-line options; zero operands streams stdin, and every explicit operand must be a regular file"
    benchmark_gate: "large rows keep resource-gated CPU/RSS evidence, but small safe rows still use native"
    fallback: "directories, explicit stdin operands, and unsupported flags preserve the original command"
scout_next:
  - "Additional narrow direct uniq shapes now cover no-file stdin and one input file with adjacent byte-line duplicate filtering; flags and multi-file forms still preserve original-command fallback."
  - "Additional narrow direct sort shapes now cover no-file stdin with the same buffered line-span sorter used for one regular file; sort options and multi-file forms still preserve original-command fallback."
  - "Additional narrow direct head/tail shapes now cover no-file stdin for default line windows, `-n N`, `-c N`, and `-N`; explicit file operands keep the existing file path, and unsupported zero-count `head` still preserves original-command fallback."
  - "Additional narrow direct wc shapes now cover stdin for `wc -l`, `wc -c`, and `wc -w`; explicit `-` stdin operands still preserve original-command fallback."
  - "Additional narrow direct wc default-output shapes now cover line/word/byte counts over stdin or explicit regular files; options beyond `-l`/`-c`/`-w` and non-regular operands still preserve original-command fallback."
  - "Additional narrow direct awk shapes now cover no-file stdin for `{ print $<field> }`, `/NEEDLE/ { print $<field> }`, and `/NEEDLE/ { c++ } END { print c }`; general awk language, flags, and unsupported scripts still preserve original-command fallback."
  - "Additional narrow direct xargs shapes now cover no-argument default echo as the same stdin token batching path as `xargs echo`; fixed `-n <positive>` batches are handled separately, while other xargs option forms still preserve original-command fallback."
  - "Additional narrow direct cut shapes now cover no-file stdin for single-field extraction with optional single-byte delimiter; byte/char/range/list/suppress and multi-file forms still preserve original-command fallback."
  - "Additional narrow tr class shapes now cover exact `[:lower:]`, `[:upper:]`, and `[:digit:]` class tokens for translate/delete paths; other character classes and full tr language still preserve original-command fallback."
  - "Additional narrow id group-list shapes now cover direct `id -G` and `id -Gn` plus the same single-line producer pipe downstreams as existing `id -u/-un/-g/-gn` support."
  - "Additional default id summary shapes now cover direct `id` plus single-line producer pipe downstreams, rendering uid/gid/groups once in-process instead of falling back to Bash."
  - "Additional narrow uname processor shapes now cover direct `uname -p` plus single-line producer pipe downstreams, using the platform processor mapping instead of falling back to Bash."
  - "Additional narrow xargs fixed-size batch shapes now cover direct `xargs -n <positive> [echo]` / `xargs -n<positive> [echo]` plus supported finite-producer and stdin xargs pipe downstreams without falling back to Bash."
  - "Additional narrow cut stdin producer shapes now cover `cut -d char -f field | wc -l` as a streaming record count without materializing cut output or re-running a downstream shell round trip."
  - "Additional narrow no-file stdin producer shapes now cover `head -n N | wc -l` and `tail -n N | wc -l` by streaming the current process stdin directly into the fused downstream count path without an intermediate shell round trip."
  - "Additional narrow xargs stdin producer shapes now cover `xargs | wc -l` and `xargs echo | wc -l` by streaming stdin token detection directly into the fused downstream count path without materializing the xargs output line or re-running a downstream shell round trip."
  - "Additional narrow xargs-grep stdin producer shapes now cover `xargs echo | grep literal` plus supported count/head/tail/sort/xargs downstreams, including a streaming `xargs echo | grep literal | wc -l` fast path that avoids materializing the xargs output line or re-running a downstream shell round trip."
  - "Additional narrow grep stdin shapes now cover direct `grep literal` over no-file stdin plus `grep literal | ...` with supported count/head/tail/sort/xargs downstreams, preserving grep no-match status without re-running a separate shell round trip between stages."
  - "Additional narrow path-lookup grep producer shapes now cover which names | grep literal and command -v names | grep literal plus supported count/head/tail/sort/xargs downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional path-lookup producer coverage now routes which names and command -v names directly through xargs echo and sorted count/head/tail/xargs downstreams without a second bash pipe."
  - "Additional narrow which -a path-lookup shapes now cover direct which -a names plus which -a names | wc/head/tail/grep/sort/xargs supported downstreams, scanning all executable PATH matches once and feeding the fused pipeline in-process."
  - "Additional narrow single-name printenv-grep producer shapes now cover printenv NAME | grep literal plus supported count/head/tail/sort/xargs downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional narrow hostname-grep producer shapes now cover hostname | grep literal plus supported count/head/tail/sort/xargs downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional single-line producer coverage now routes printenv NAME and hostname directly through xargs echo and sort-to-xargs echo, preserving missing printenv names as empty downstream input instead of an extra bash pipe."
  - "Additional narrow empty primitive producer shapes now cover true | ... and false | ... with supported count/head/tail/sort/xargs plus grep downstreams, preserving Bash last-stage pipeline exit behavior without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional narrow side-effect empty producer shapes now cover mkdir [-p] paths | ... and touch paths | ... with supported count/head/tail/sort/xargs plus grep downstreams, performing the left-side side effect before feeding an empty stream to the downstream stage."
  - "Additional narrow predicate empty producer shapes now cover test predicates | ... and bracket predicates | ... with supported count/head/tail/sort/xargs plus grep downstreams, evaluating the left-side predicate before feeding an empty stream to the downstream stage."
  - "Additional narrow wc finite producer shapes now cover wc -l/-c/-w regular files | ... with supported count/head/tail/sort/xargs plus literal-grep downstreams, emitting native wc rows once and feeding the rest of the pipeline in-process."
  - "Additional narrow wc stdin finite producer shapes now cover wc -l/-c/-w | ... with supported count/head/tail/sort/xargs plus literal-grep downstreams, reading stdin once, emitting the native wc row once, and feeding the rest of the pipeline in-process."
  - "Additional narrow literal printf finite producer shapes now cover one no-conversion printf format arg using only \\\\, \\n, \\t, or \\r escapes | ... with supported count/head/tail/sort/xargs plus literal-grep downstreams, emitting the literal bytes once and feeding the rest of the pipeline in-process."
  - "Additional narrow du finite producer shapes now cover du -sk existing-path | ... with supported count/head/tail/sort/xargs plus literal-grep downstreams, emitting the native du row once and feeding the rest of the pipeline in-process."
  - "Additional narrow awk stdin producer shapes now cover `awk '{ print $<field> }' | ...` and `awk '/NEEDLE/ { print $<field> }' | ...` with the same supported count/head/tail/sort/xargs downstreams as file-backed awk producers."
  - "Additional finite-producer awk pipe shapes now cover `echo ... | awk '{ print $<field> }'` and `printf '%s\\n' ... | awk '{ print $<field> }'` plus supported count/head/tail/sort/xargs downstreams without falling back to bash or re-running a separate shell round trip between stages."
  - "Additional narrow wc byte/word shapes now cover direct `wc -c` and `wc -w` over regular files plus terminal `wc -c`/`wc -w` downstreams for generated, file, sort, sort|uniq, and finite producer pipe output without adding another shell round trip between pipe stages."
  - "Initial fused shapes now cover echo ... | wc -l, echo ... | head -n N, echo ... | tail -n N, echo ... | tr set1 set2, echo ... | xargs echo, echo ... | xargs wc -l, printf '%s\\n' ... | wc -l, printf '%s\\n' ... | head -n N, printf '%s\\n' ... | tail -n N, printf '%s\\n' ... | grep literal, printf '%s\\n' ... | tr set1 set2, printf '%s\\n' ... | xargs echo, printf '%s\\n' ... | xargs wc -l, seq ... | wc -l, seq ... | head -n N, seq ... | tail -n N, seq ... | xargs echo, yes [word] | head -n N, ls [-1] dir | wc -l, ls [-1] dir | head -n N, ls [-1] dir | tail -n N, ls [-1] dir | sort, ls [-1] dir | sort | uniq, ls [-1] dir | sort | uniq | wc -l, ls [-1] dir | sort | wc -l, ls [-1] dir | sort | head -n N, ls [-1] dir | sort | tail -n N, ls [-1] dir | grep literal, ls [-1] dir | grep literal | wc -l, ls [-1] dir | grep literal | xargs echo, ls [-1] dir | grep literal | sort | xargs echo, ls [-1] dir | xargs echo, sort ... | uniq, sort ... | uniq | wc -l, sort ... | head -n N, sort ... | tail -n N, sort ... | wc -l, sort ... | xargs echo, sort ... | xargs wc -l, cat ... | wc -l, cat ... | head -n N, cat ... | tail -n N, cat ... | grep literal, cat ... | grep literal | wc -l, cat ... | grep literal | head -n N, cat ... | grep literal | tail -n N, cat ... | grep literal | sort, cat ... | grep literal | sort | uniq, cat ... | grep literal | sort | uniq | wc -l, cat ... | grep literal | sort | wc -l, cat ... | grep literal | sort | head -n N, cat ... | grep literal | sort | tail -n N, cat ... | grep literal | xargs echo, cat ... | grep literal | xargs wc -l, cat ... | grep literal | sort | xargs echo, cat ... | grep literal | sort | xargs wc -l, cat ... | cut -d char -f field, cat ... | tr set1 set2, cat ... | xargs echo, cat ... | xargs wc -l, cat ... | uniq, cat ... | uniq | wc -l, cat ... | sort, cat ... | sort | uniq, cat ... | sort | uniq | wc -l, cat ... | sort | wc -l, cat ... | sort | head -n N, cat ... | sort | tail -n N, cat ... | sort | xargs echo, cat ... | sort | xargs wc -l, grep ... | wc -l, grep ... | head -n N, grep ... | tail -n N, grep ... | sort, grep ... | sort | uniq, grep ... | sort | uniq | wc -l, grep ... | sort | wc -l, grep ... | sort | head -n N, grep ... | sort | tail -n N, grep ... | xargs echo, grep ... | xargs wc -l, grep ... | sort | xargs echo, grep ... | sort | xargs wc -l, grep -R ... | head -n N, grep -R ... | tail -n N, grep -R ... | sort, grep -R ... | sort | uniq, grep -R ... | sort | uniq | wc -l, grep -R ... | sort | wc -l, grep -R ... | sort | head -n N, grep -R ... | sort | tail -n N, grep -R ... | wc -l, awk ... | xargs echo, awk ... | xargs wc -l, which ... | wc -l, which ... | head -n N, which ... | tail -n N, command -v ... | wc -l, command -v ... | head -n N, command -v ... | tail -n N, printenv NAME | wc -l, printenv NAME | head -n N, printenv NAME | tail -n N, printenv NAME | grep literal, printenv NAME | sort, hostname | wc -l, hostname | head -n N, hostname | tail -n N, hostname | grep literal, hostname | sort, find ... -type f optional-name <safe-glob> | xargs wc -l, find ... -type f optional-name <safe-glob> | xargs echo, find ... -type f optional-name <safe-glob> | grep literal | xargs echo, find ... -type f optional-name <safe-glob> | grep literal | xargs wc -l, find ... -type f optional-name <safe-glob> | grep literal | sort | xargs echo, find ... -type f optional-name <safe-glob> | grep literal | sort | xargs wc -l, find ... -type f optional-name <safe-glob> | wc -l, find ... -type f optional-name <safe-glob> | head -n N, find ... -type f optional-name <safe-glob> | tail -n N, find ... -type f optional-name <safe-glob> | sort, find ... -type f optional-name <safe-glob> | sort | uniq, find ... -type f optional-name <safe-glob> | sort | uniq | wc -l, find ... -type f optional-name <safe-glob> | sort | wc -l, find ... -type f optional-name <safe-glob> | sort | xargs echo, find ... -type f optional-name <safe-glob> | sort | xargs wc -l, find ... -type f optional-name <safe-glob> | sort | head -n N, and find ... -type f optional-name <safe-glob> | sort | tail -n N; broader pipe support should add new exact shapes with parity and benchmark evidence."
  - "Additional narrow printf producer shapes now cover printf '%s\\n' ... | sort, printf '%s\\n' ... | sort | uniq, printf '%s\\n' ... | sort | uniq | wc -l, printf '%s\\n' ... | sort | uniq | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, printf '%s\\n' ... | sort | wc -l, printf '%s\\n' ... | sort | head -n N, printf '%s\\n' ... | sort | tail -n N, printf '%s\\n' ... | sort | xargs echo, and printf '%s\\n' ... | sort | xargs wc -l."
  - "Additional narrow printf literal-filter shapes now cover printf '%s\\n' ... | grep literal | wc -l, printf '%s\\n' ... | grep literal | head -n N, printf '%s\\n' ... | grep literal | tail -n N, printf '%s\\n' ... | grep literal | sort, printf '%s\\n' ... | grep literal | sort | uniq, printf '%s\\n' ... | grep literal | sort | uniq | wc -l, printf '%s\\n' ... | grep literal | sort | uniq | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, printf '%s\\n' ... | grep literal | sort | wc -l, printf '%s\\n' ... | grep literal | sort | head -n N, printf '%s\\n' ... | grep literal | sort | tail -n N, printf '%s\\n' ... | grep literal | sort | xargs echo, and printf '%s\\n' ... | grep literal | xargs echo."
  - "Additional narrow seq producer shapes now cover seq ... | sort, seq ... | sort | uniq, seq ... | sort | uniq | wc -l, seq ... | sort | uniq | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, seq ... | sort | wc -l, seq ... | sort | head -n N, seq ... | sort | tail -n N, and seq ... | sort | xargs echo."
  - "Additional narrow seq literal-filter shapes now cover seq ... | grep literal, seq ... | grep literal | wc -l, seq ... | grep literal | head -n N, seq ... | grep literal | tail -n N, seq ... | grep literal | sort, seq ... | grep literal | sort | uniq, seq ... | grep literal | sort | uniq | wc -l, seq ... | grep literal | sort | uniq | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, seq ... | grep literal | sort | wc -l, seq ... | grep literal | sort | head -n N, seq ... | grep literal | sort | tail -n N, seq ... | grep literal | sort | xargs echo, and seq ... | grep literal | xargs echo."
  - "Additional narrow recursive grep producer shapes now cover grep -R ... | sort | uniq | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow single-file grep producer shapes now cover grep literal file | sort | uniq | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow grep-file-cut producer shapes now cover grep literal file | cut -d char -f field plus supported grep/count/head/tail/sort/xargs downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional narrow grep-file-awk producer shapes now cover grep literal file | awk '{ print $<field> }' plus supported grep/count/head/tail/sort/xargs downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional narrow unfiltered awk fixed-field producer shapes now cover awk '{ print $<field> }' file and cat file | awk '{ print $<field> }' plus supported count/head/tail/sort/xargs downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional narrow awk-grep producer shapes now cover awk '{ print $<field> }' file | grep literal plus cat file | awk '{ print $<field> }' | grep literal with supported count/head/tail/sort/xargs downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional narrow awk producer shapes now cover awk ... | wc -l, awk ... | head -n N, awk ... | tail -n N, awk ... | sort, awk ... | sort | uniq, awk ... | sort | uniq | wc -l, awk ... | sort | uniq | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, awk ... | sort | wc -l, awk ... | sort | head -n N, awk ... | sort | tail -n N, awk ... | sort | xargs echo, and awk ... | sort | xargs wc -l for file-backed and no-file stdin producers."
  - "Additional narrow cat-awk producer shapes now cover cat file | awk ... direct output plus cat file | awk ... | wc/head/tail/sort/xargs echo/xargs wc -l, cat file | awk ... | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, and cat file | awk ... | sort | xargs echo/xargs wc -l without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow cat-head/tail producer aliases now cover cat file | head/tail default, cat file | head/tail -N, and cat file | head/tail -n N | wc/head/tail/sort/xargs echo/xargs wc -l, sort | uniq/wc/head/tail/xargs echo/xargs wc -l, and grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|xargs echo/xargs wc -l without falling back to bash -c for those bounded stdin-style producer pipe strings."
  - "Additional narrow head producer shapes now cover head -n N [file] | wc -l/head/tail/sort/xargs echo/xargs wc -l, head -n N [file] | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, head -n N [file] | grep literal, and head -n N [file] | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, with streaming fast-frontend paths for non-sort consumers and benchmarked large-input dual-win rows."
  - "Additional narrow tail producer shapes now cover tail -n N [file] | wc -l/head/tail/sort/xargs echo/xargs wc -l, tail -n N [file] | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, tail -n N [file] | grep literal, and tail -n N [file] | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, with fast-frontend backward offset discovery plus forward streaming and benchmarked large-input dual-win rows."
  - "Additional narrow single-line producer shapes now cover pwd/basename/dirname/whoami/id/uname | wc -l/head/tail/sort/xargs echo/xargs wc -l, plus literal grep and the same grep downstream sort/count/head/tail/xargs modes, without falling back to bash -c for those linear pipe strings."
  - "Additional narrow sed producer shapes now cover sed -n range file | wc/head/tail/sort/xargs echo/xargs wc -l, sed -n range file | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, sed -n range file | grep literal, and sed -n range file | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, without falling back to bash -c for those bounded producer pipe strings."
  - "Additional narrow cat-sed producer shapes now cover cat file | sed -n range | wc/head/tail/sort/xargs echo/xargs wc -l, cat file | sed -n range | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, cat file | sed -n range | grep literal, and cat file | sed -n range | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, without falling back to bash -c for those bounded producer pipe strings."
  - "Additional narrow cut producer shapes now cover cut -d char -f field file | wc/head/tail/sort/xargs echo/xargs wc -l, cut -d char -f field file | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, cut -d char -f field file | grep literal, and cut -d char -f field file | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow cat-cut producer shapes now cover cat file | cut -d char -f field | wc/head/tail/sort/xargs echo/xargs wc -l, cat file | cut -d char -f field | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, cat file | cut -d char -f field | grep literal, and cat file | cut -d char -f field | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow cat-tr producer shapes now cover cat file | tr set1 set2 | wc/head/tail/sort/xargs echo/xargs wc -l, cat file | tr set1 set2 | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, cat file | tr set1 set2 | grep literal, and cat file | tr set1 set2 | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow cat-grep-sort-uniq producer shapes now cover cat file | grep literal | sort | uniq | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow cat-uniq producer shapes now cover cat file | uniq | wc/head/tail/sort/xargs echo/xargs wc -l, cat file | uniq | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, cat file | uniq | grep literal, and cat file | uniq | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow direct uniq producer shapes now cover uniq file | wc/head/tail/sort/xargs echo/xargs wc -l, uniq file | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, uniq file | grep literal, and uniq file | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow sort-uniq producer shapes now cover sort file | uniq | wc/head/tail/sort/xargs echo/xargs wc -l, sort file | uniq | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, sort file | uniq | grep literal, sort file | uniq | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, and the same cat file | sort | uniq aliases, without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow sort-grep producer shapes now cover sort file | grep literal plus cat file | sort | grep literal with supported count/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional narrow ls-sort-uniq producer shapes now cover ls [-1] dir | sort | uniq | wc/head/tail/sort/xargs echo, ls [-1] dir | sort | uniq | sort | uniq/wc/head/tail/xargs echo, ls [-1] dir | sort | uniq | grep literal, and ls [-1] dir | sort | uniq | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/sort|xargs echo, without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow find-sort-uniq producer shapes now cover find root -type f optional-name glob | sort | uniq | wc/head/tail/sort/xargs echo/xargs wc -l, find root -type f optional-name glob | sort | uniq | sort | uniq/wc/head/tail/xargs echo/xargs wc -l, find root -type f optional-name glob | sort | uniq | grep literal, and find root -type f optional-name glob | sort | uniq | grep literal | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail/xargs echo/xargs wc -l/sort|xargs echo/sort|xargs wc -l, without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow find xargs-wc output producer shapes now cover find root -type f optional-name glob | xargs wc -l | wc/head/tail/sort/sort|uniq/sort|wc/sort|head/sort|tail, find root -type f optional-name glob | sort or sort | uniq | xargs wc -l | sort/head/tail/sort|wc, and find root -type f optional-name glob | grep literal | sort or sort | uniq | xargs wc -l | sort/tail, without falling back to bash -c for those finite producer pipe strings in the Rust planner or cap-fast frontend."
  - "Additional generalized finite producer xargs-wc output shapes now cover direct cat/sort path-list streams, grep literal file, narrow awk fixed-field file, and cat file | awk fixed-field path-token streams through xargs wc -l | count/head/tail/sort downstreams, including sorted input tokens, without re-running the downstream side through a second shell round trip."
  - "Additional narrow find-grep producer shapes now cover find root -type f optional-name glob | grep literal direct output plus supported wc/head/tail/sort/sort|uniq/xargs echo/xargs wc -l downstreams without falling back to bash -c for those finite producer pipe strings."
  - "Additional narrow default-xargs find pipe shapes now cover find root -type f optional-name glob | xargs as default xargs-echo output without falling back to bash -c in the Rust planner or cap-fast frontend."
  - "Additional narrow find maxdepth-positive pipe shapes now cover find root -maxdepth positive-integer -type f optional-name glob plus supported count/head/tail/sort/xargs and grep downstreams without falling back to bash -c or re-running a separate shell round trip between stages."
  - "Additional narrow ls -a and ls -A pipe shapes now cover all-entry and almost-all directory output plus supported count/head/tail/sort/sort|uniq/xargs echo and grep downstreams, preserving the distinction that -a includes . and .. while -A includes other dot entries but excludes . and ..."
  - "Additional narrow ls-sort-xargs pipe shapes now cover ls [-1|-a|-A] dir | sort | xargs echo without falling back to bash -c, while ls ... | sort | xargs wc -l stays compatibility fallback because ls emits cwd-relative entry names."
  - "Additional narrow ls-grep producer shapes now cover ls [-1] dir | grep literal direct output plus supported wc/head/tail/sort/sort|uniq/xargs echo downstreams without falling back to bash -c, while xargs wc remains compatibility fallback because ls emits cwd-sensitive entry names."
  - "Unsupported pipe syntax and full-environment pipes such as env | wc -l or printenv | sort remain compatibility fallback until a conservative segment-aware planner can prove Bash environment behavior."
---
flowchart TD
    start([cap argv or shell-free cap run command]) --> classify{matches safe same-name subset?}
    classify -- no --> fallback_shape([original path: unsupported or shell-sensitive shape])
    classify -- yes --> wc_shape{wc -l over regular files?}
    wc_shape -- no --> fallback_shape
    wc_shape -- yes --> probe_files[verify regular-file safety]
    probe_files --> native_wc[run cap native line-count aggregate]
    native_wc --> explain[cap explain reports native_wc_lines vs original fallback]
    explain --> parity[stdout/stderr/exit parity tests]
    parity --> bench[CPU/RSS large-workload benchmark]
    bench --> done([native safe subset with shell fallback boundary])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cap-native-command-coverage-safe-shape-tests
requirements:
  wc_small_native:
    id: HV-UT-1
    text: "Planner promotes wc -l small regular-file sets to the native aggregate line-count path."
    kind: functional
    risk: high
    verify: test
  wc_large_native:
    id: HV-UT-2
    text: "Planner promotes wc -l many-file or large-byte regular-file operands to the native aggregate line-count path."
    kind: functional
    risk: high
    verify: test
  wc_parity_success:
    id: HV-UT-3
    text: "Native wc -l preserves stdout and exit status for single-file and multi-file success cases, including total rows."
    kind: functional
    risk: high
    verify: test
  wc_parity_errors:
    id: HV-UT-4
    text: "Missing paths, directories, explicit stdin operands, and unsupported wc flags fail open to the original path."
    kind: functional
    risk: high
    verify: test
  explain_visibility:
    id: HV-UT-5
    text: "cap explain reports native_wc_lines for safe workloads and original fallback for unsupported workloads."
    kind: functional
    risk: medium
    verify: test
  benchmark_evidence:
    id: HV-UT-6
    text: "command_resources includes a large wc -l row comparing cap native aggregate against the original system command with CPU and peak RSS evidence."
    kind: functional
    risk: high
    verify: benchmark
elements:
  planner_shape_tests:
    kind: test
    type: "cargo test -p cap command_planner"
  replacement_parity_tests:
    kind: test
    type: "cargo test -p cap behavior_cap_command_replacement_parity"
  explain_tests:
    kind: test
    type: "cargo test -p cap explain"
  resource_benchmark_matrix:
    kind: benchmark
    type: "cargo bench -p cap --bench command_resources"
relations:
  - { from: planner_shape_tests, verifies: wc_small_native }
  - { from: planner_shape_tests, verifies: wc_large_native }
  - { from: replacement_parity_tests, verifies: wc_parity_success }
  - { from: replacement_parity_tests, verifies: wc_parity_errors }
  - { from: explain_tests, verifies: explain_visibility }
  - { from: resource_benchmark_matrix, verifies: benchmark_evidence }
---
requirementDiagram
  requirement wc_small_native {
    id: HV-UT-1
    text: "small wc -l regular-file workloads use native aggregate path"
    risk: high
    verifymethod: test
  }
  requirement wc_large_native {
    id: HV-UT-2
    text: "large wc -l workloads use native aggregate path"
    risk: high
    verifymethod: test
  }
  requirement wc_parity_success {
    id: HV-UT-3
    text: "native wc -l success output matches system wc"
    risk: high
    verifymethod: test
  }
  requirement wc_parity_errors {
    id: HV-UT-4
    text: "unsupported or error cases fail open"
    risk: high
    verifymethod: test
  }
  requirement explain_visibility {
    id: HV-UT-5
    text: "explain shows promoted versus fallback path"
    risk: medium
    verifymethod: test
  }
  requirement benchmark_evidence {
    id: HV-UT-6
    text: "benchmarks record large-workload resource evidence"
    risk: high
    verifymethod: benchmark
  }
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/cap/src/command_planner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Add a shape-gated WcLines native command plan for `wc -l/-c/-w [FILE...]`.
      The planner accepts stdin when no operands are present, accepts only
      regular-file explicit operands with no directory or unsupported flag
      semantics, and promotes safe file sets at any size.
      Missing or unsupported shapes remain External Original so behavior stays
      delegated to the system command.

  - path: apps/cap/src/command_planner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Add the Rust native aggregate line-count runner and `cap explain`
      rendering for promoted `wc -l` workloads. The runner must preserve the
      system `wc -l` success shape, including per-file rows and the multi-file
      total row.

  - path: apps/cap/src/cap_fast_frontend.c
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Add the low-overhead production fast path for `wc -l FILE...`, sharing
      the same shape gate as the Rust planner. The C frontend handles safe
      regular-file shapes at any size and returns unsupported for unsafe shapes
      so the public `cap` launcher continues through `cap-full` and
      original-command fallback.

  - path: apps/cap/src/cap_fast_frontend.c
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Register `wc` as an active same-name candidate after the fast path can
      prove regular-file safety. Do not add arbitrary `wc` option support in
      this slice.

  - path: apps/cap/tests/behavior_cap_command_replacement_parity.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Extend installed-frontend parity coverage with large `wc -l` success,
      `cap run "wc -l ..."` success, and fallback/error cases for missing
      paths or unsupported operands.

  - path: apps/cap/benches/command_resources.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Keep a high-volume `wc -l` benchmark scenario with the dual-win gate,
      comparing the production C frontend against `/usr/bin/wc` using median
      child CPU time and peak RSS.

  - path: apps/cap/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Document `wc -l` as an active safe-shape fast path and describe the
      fused pipe shapes that reuse line-count, token batching, and early-stop
      primitives while unsupported pipelines keep shell fallback.

  - path: apps/cap/BENCHMARKS.md
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Record the measured `wc -l` resource result and the gating decision after
      running the command resource benchmark.
```
