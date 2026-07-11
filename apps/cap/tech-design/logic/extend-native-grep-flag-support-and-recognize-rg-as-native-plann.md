---
id: extend-native-grep-flag-support-and-recognize-rg-as-native-plann
summary: Extend cap's native grep matcher (plan_grep_file/GrepFilePlan) to accept a bounded flag vocabulary (-n -i -v -c -l -o -A/-B/-C -e), add an `rg` alias in plan_native's dispatch that normalizes onto the same vocabulary plus a cosmetic no-op allowlist, and translate bare zero-flag `rg` to the literal `grep` token at the pipe-fusion word-list entry point so existing fusion arms keep working — falling back to bash unchanged for any unsupported flag/shape.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: command-planner-grep-flag-rg-alias-native-recognition
    claim: command-planner-grep-flag-rg-alias-native-recognition
    coverage: partial
    rationale: "Extends plan_grep_file/GrepFilePlan with a bounded flag vocabulary and adds an rg alias dispatch arm, covering the large majority of real grep/rg traffic that today falls back to bash, under the same capability the prior cd-prefix native recognition work (#1378) landed under."
---

# TD: cap native grep flag support and rg alias recognition

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-native-grep-flags-and-rg-alias
entry: plan_native_dispatch
nodes:
  plan_native_dispatch: { kind: start, label: "plan_native's match on command[0]: 'grep' -> plan_grep_file(args); 'rg' -> new rg alias arm" }
  rg_normalize: { kind: process, label: "normalize_rg_args: translate rg-specific cosmetic no-ops and feed remaining argv into the same flag parser plan_grep_file uses" }
  zero_flag_stdin: { kind: decision, label: "args == [pattern], no leading '-', is_plain_literal_pattern(pattern)? (EXISTING branch, byte-for-byte unchanged)" }
  zero_flag_file: { kind: decision, label: "args == [pattern, file], no leading '-' on either, is_plain_literal_pattern(pattern), Path::new(file).is_file()? (EXISTING branch, byte-for-byte unchanged)" }
  flag_tokenize: { kind: process, label: "NEW third branch: split args into flag tokens vs non-flag tokens, preserving order" }
  flag_parse: { kind: process, label: "parse each flag token: combinable short flags (-n -i -v -c -l -o, may cluster e.g. -ni); -A/-B/-C accepted only as standalone token+numeric-token or attached-digit form (-A3), never combined with other short flags in the same cluster (-A3n rejected); repeatable -e PATTERN collected into patterns list" }
  flag_reject: { kind: decision, label: "any flag token outside the accepted set (e.g. -r -R --glob -g --type --include -u -p --json -U) or a rejected combined-context form (-A3n)?" }
  cosmetic_rg_allowlist: { kind: decision, label: "(rg path only) token is one of --no-heading / --heading / --color=<any>? consumed harmlessly, does not count as a disqualifying flag" }
  pattern_source: { kind: decision, label: "no -e tokens given AND exactly one non-flag token remains before an optional file? use it as the sole pattern (bare-pattern form)" }
  pattern_validate: { kind: decision, label: "every collected pattern (bare or from -e) passes is_plain_literal_pattern (no regex metachars)?" }
  file_count: { kind: decision, label: "at most one remaining non-flag, non-pattern token (the file), and if present Path::new(file).is_file()?" }
  build_plan: { kind: process, label: "construct GrepFilePlan{ patterns, ignore_case, invert, line_numbers, count, files_with_matches, only_matching, context_before, context_after, file } wrapped in NativeCommand::GrepFile" }
  disqualify: { kind: terminal, label: "None returned; plan_native falls through to plan_shell's unchanged bash -c <original> fallback" }
  native_result: { kind: terminal, label: "Some(NativePlan) returned; reason names the accepted flag set" }
  pipe_words_assembled: { kind: start, label: "plan_shell: split_simple_shell_words(original) produces the pipe-segment word list (existing point words are first assembled)" }
  rg_pipe_translate: { kind: decision, label: "NEW: leading segment's first word is 'rg' AND the rest of that segment is the exact already-accepted zero-flag shape ([pattern] or [pattern, file], no dashes, is_plain_literal_pattern)?" }
  rewrite_to_grep: { kind: process, label: "rewrite that segment's first word from 'rg' to the literal 'grep' token in the word list (no new match arms added)" }
  existing_fusion_arms: { kind: terminal, label: "existing plan_pipe_words / plan_head_grep_producer_mode family (keyed on literal 'grep') dispatch unchanged" }
  flag_bearing_pipe_fallback: { kind: terminal, label: "flag-bearing rg (or grep) segment inside a pipe: no translation attempted here; falls through to whatever the existing pipe-fusion / bash-fallback path already does for that shape (explicit out-of-scope boundary, not a regression)" }
edges:
  - { from: plan_native_dispatch, to: zero_flag_stdin, label: "command[0] == 'grep'" }
  - { from: plan_native_dispatch, to: rg_normalize, label: "command[0] == 'rg'" }
  - { from: rg_normalize, to: cosmetic_rg_allowlist }
  - { from: cosmetic_rg_allowlist, to: zero_flag_stdin, label: "no cosmetic tokens present, remaining argv shape checked same as grep" }
  - { from: cosmetic_rg_allowlist, to: disqualify, label: "unrecognized rg-specific flag present" }
  - { from: zero_flag_stdin, to: native_result, label: "matches (EXISTING reason string unchanged)" }
  - { from: zero_flag_stdin, to: zero_flag_file, label: "no match" }
  - { from: zero_flag_file, to: native_result, label: "matches (EXISTING reason string unchanged)" }
  - { from: zero_flag_file, to: flag_tokenize, label: "no match, but args non-empty" }
  - { from: flag_tokenize, to: flag_parse }
  - { from: flag_parse, to: flag_reject }
  - { from: flag_reject, to: disqualify, label: "yes: unsupported flag or -A3n-style combined-context form" }
  - { from: flag_reject, to: pattern_source, label: "no: all flag tokens accepted" }
  - { from: pattern_source, to: pattern_validate }
  - { from: pattern_validate, to: disqualify, label: "any pattern contains a regex metacharacter (not plain literal)" }
  - { from: pattern_validate, to: file_count, label: "all patterns plain literal" }
  - { from: file_count, to: disqualify, label: "more than one file token, or the one file token is missing/not-a-regular-file (excludes directories, no recursive support)" }
  - { from: file_count, to: build_plan, label: "zero or one valid file" }
  - { from: build_plan, to: native_result }
  - { from: pipe_words_assembled, to: rg_pipe_translate }
  - { from: rg_pipe_translate, to: rewrite_to_grep, label: "yes: bare rg, zero flags, already-accepted shape" }
  - { from: rewrite_to_grep, to: existing_fusion_arms }
  - { from: rg_pipe_translate, to: flag_bearing_pipe_fallback, label: "no: flag-bearing rg/grep segment inside a pipe, or non-grep/rg segment" }
---
flowchart TB
  plan_native_dispatch["plan_native's match on command[0]: 'grep' -> plan_grep_file(args); 'rg' -> new rg alias arm"] -->|command[0] == 'grep'| zero_flag_stdin{"args == [pattern], no leading '-', is_plain_literal_pattern? (EXISTING, unchanged)"}
  plan_native_dispatch -->|command[0] == 'rg'| rg_normalize["normalize_rg_args: translate rg cosmetic no-ops, feed remaining argv into the same flag parser"]
  rg_normalize --> cosmetic_rg_allowlist{"token is --no-heading / --heading / --color=<any>? consumed harmlessly"}
  cosmetic_rg_allowlist -->|no cosmetic tokens present, remaining argv shape checked same as grep| zero_flag_stdin
  cosmetic_rg_allowlist -->|unrecognized rg-specific flag| disqualify(["None: plan_native falls through to plan_shell's unchanged bash -c fallback"])
  zero_flag_stdin -->|matches, EXISTING reason unchanged| native_result(["Some(NativePlan); reason names accepted flag set"])
  zero_flag_stdin -->|no match| zero_flag_file{"args == [pattern, file], no leading '-', is_plain_literal_pattern, Path::is_file()? (EXISTING, unchanged)"}
  zero_flag_file -->|matches, EXISTING reason unchanged| native_result
  zero_flag_file -->|no match, args non-empty| flag_tokenize["NEW third branch: split args into flag tokens vs non-flag tokens, order preserved"]
  flag_tokenize --> flag_parse["parse: combinable short flags (-n -i -v -c -l -o, e.g. -ni); -A/-B/-C standalone+numeric or attached-digit only, never combined with other shorts (-A3n rejected); repeatable -e PATTERN collected"]
  flag_parse --> flag_reject{"any flag token outside accepted set, or a rejected combined-context form?"}
  flag_reject -->|yes: unsupported flag / -A3n-style form| disqualify
  flag_reject -->|no: all accepted| pattern_source{"no -e tokens given AND exactly one non-flag token before optional file? use as bare pattern"}
  pattern_source --> pattern_validate{"every collected pattern (bare or -e) passes is_plain_literal_pattern?"}
  pattern_validate -->|any pattern has a regex metacharacter| disqualify
  pattern_validate -->|all plain literal| file_count{"at most one remaining non-flag/non-pattern token as file, and Path::is_file() if present?"}
  file_count -->|more than one file, or missing/not-regular-file| disqualify
  file_count -->|zero or one valid file| build_plan["construct GrepFilePlan{patterns, ignore_case, invert, line_numbers, count, files_with_matches, only_matching, context_before, context_after, file} wrapped in NativeCommand::GrepFile"]
  build_plan --> native_result
  pipe_words_assembled["plan_shell: split_simple_shell_words(original) produces the pipe-segment word list"] --> rg_pipe_translate{"leading segment's first word is 'rg' AND rest is the exact already-accepted zero-flag shape?"}
  rg_pipe_translate -->|yes: bare rg, zero flags, accepted shape| rewrite_to_grep["rewrite that segment's first word 'rg' -> literal 'grep' in the word list (no new match arms added)"]
  rewrite_to_grep --> existing_fusion_arms(["existing plan_pipe_words / plan_head_grep_producer_mode family, keyed on literal 'grep', dispatches unchanged"])
  rg_pipe_translate -->|no: flag-bearing rg/grep segment in a pipe, or non-grep/rg segment| flag_bearing_pipe_fallback(["falls through to whatever the existing pipe-fusion/bash-fallback path already does (explicit out-of-scope boundary, not a regression)"])
```

`plan_native`'s existing match arm `"grep" => plan_grep_file(&command[1..], label, original)`
(~line 2433) is extended, and a new sibling arm `"rg" => ...` is added
alongside it. Both existing branches inside `plan_grep_file` (~line 9370) —
`[pattern]` (stdin, no leading `-`, `is_plain_literal_pattern`) and
`[pattern, file]` (same pattern restriction plus `!file.starts_with('-')`
and `Path::new(file).is_file()`) — are preserved byte-for-byte, including
their exact reason strings (`"plain literal grep over stdin can scan
in-process"` / `"plain literal grep over one regular file can scan
in-process"`); this TD adds a **third** match branch (or a sibling helper
`plan_grep_file` delegates to) that only activates when neither existing
branch matches, so the zero-flag shape and its behavior are provably
unaffected (mirrors #1378's insertion-at-the-tail discipline).

The new branch tokenizes `args` into flag tokens (leading `-`) and
non-flag tokens, preserving order. Accepted short flags are `-n`
(`--line-number`), `-i` (`--ignore-case`), `-v` (`--invert-match`), `-c`
(`--count`), `-l` (`--files-with-matches`), `-o` (`--only-matching`); these
six may combine in one clustered token (e.g. `-ni`, `-cv`) exactly like
real grep/rg. The context flags `-A`, `-B`, `-C` are accepted only in two
shapes — a standalone flag token immediately followed by a numeric token
(`-A 3`), or an attached-digit token (`-A3`) — and are rejected (the whole
line disqualifies, falling back to bash) if fused with any other short
flag in the same cluster (e.g. `-A3n` is NOT accepted; this is a
deliberate, documented scope boundary, not a bug). `-e PATTERN` is
repeatable: each occurrence's `PATTERN` argument is collected into a
`patterns: Vec<String>` list that is OR-combined at match time (a line
matches if any pattern matches). When no `-e` token is present and exactly
one non-flag token remains ahead of an optional file token, that token is
the sole bare pattern (grep's traditional `grep [flags] pattern [file]`
shape generalized to carry flags).

Every collected pattern — whether from repeated `-e` or the bare-pattern
form — must independently pass the existing `is_plain_literal_pattern`
check (~line 16871); this TD explicitly does not expand pattern
recognition into any regex dialect (grep's BRE-ish semantics and rg's
Rust-regex-crate semantics are different enough that mixing them unsafely
would be worse than staying conservative — deferred as a documented
follow-up, not implemented here). Any pattern containing a regex
metacharacter disqualifies the whole line.

At most one non-flag, non-pattern token may remain: the file argument
(stdin is used when it is absent, matching the existing zero-flag stdin
branch). If present, it must satisfy `Path::new(file).is_file()` — the
same check the existing `[pattern, file]` branch already uses, which
naturally excludes directories without a separate directory check (no
recursive support, matching R4). More than one such remaining token
disqualifies the whole line. Any flag or shape outside this accepted
vocabulary — `-r`/`-R`, `--glob`/`-g`, `--type`, `--include`, `-u`, `-p`,
`--json`, `-U`, more than one file, or a rejected combined-context form
like `-A3n` — returns `None`, so `plan_native` yields `None` and the
caller's existing final fallback (`bash -c <original>`, unchanged) runs
the whole original command: never partial in-process execution of a
line this TD's parser only partly understood.

The new `"rg" => ...` dispatch arm normalizes rg's argv onto the identical
accepted vocabulary above before feeding it through the same flag-parsing
path (rg's own documentation states -n -i -v -c -l -o -A -B -C -e mean the
same thing as grep's, so no separate semantic table is needed — only a
thin normalization step). A small allowlist of cosmetic rg-only flags is
consumed harmlessly rather than disqualifying: `--no-heading`, `--heading`,
and `--color=<mode>` (any value) never change match semantics and are
common in real traffic (`--no-heading` appears in 15/182 log-sampled rg
events per the WI's log analysis). Any other rg-specific flag (anything
not in the shared vocabulary or the cosmetic allowlist) disqualifies to
the bash fallback, matching R3/R4/AC3.

Separately, and independently of the flag-parsing path above, `plan_shell`
(~line 2213) assembles the pipe-segment word list via
`split_simple_shell_words(original)` before checking for `"|"` and calling
`plan_pipe_words` — the existing family of `cmd == "grep"` pipe-fusion
match arms (`plan_head_grep_producer_mode` and siblings, ~3378-4600+ and
more) key strictly off the literal string `"grep"`. This TD adds one
normalization step at that same word-list-assembly point: if the leading
segment's first word is `rg` and the remainder of that segment is *exactly*
the already-accepted zero-flag shape (`[pattern]` or `[pattern, file]`,
no leading `-` on either token, `is_plain_literal_pattern(pattern)`), the
first word is rewritten from `rg` to the literal `grep` token in the
assembled word list before `plan_pipe_words` runs — so the entire existing
fusion-arm family works unmodified for bare zero-flag `rg`, with zero
duplicated match arms. Any other shape (a non-grep/rg segment, or a
flag-bearing `rg`/`grep` segment participating in a pipe) is left
untouched and falls through to whatever the existing pipe-fusion/bash
fallback logic already does for it today — flag-bearing rg combined with
pipe-fusion is an explicit, documented out-of-scope boundary for this TD
(R5), not a regression.

`plan_grep_replacement` (~line 9483), the pre-existing, unrelated
`ExternalPlan`-based ripgrep shell-out path used for ranges this native
matcher declines (e.g. recursive/glob/type-filtered grep), is untouched by
this TD; nothing here duplicates its flag-mapping table.
