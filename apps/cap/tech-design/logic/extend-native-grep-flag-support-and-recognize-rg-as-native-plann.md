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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: extend-native-grep-flag-support-and-recognize-rg-as-native-plann-verification
requirements:
  cd_prefix_suite_unaffected_by_grep_rg_changes:
    id: R6
    text: "The #1378 cd-prefix native-recognition test suite (cd_prefix_grep_replans_native and siblings) remains unaffected by this TD's flag-parsing and rg-alias additions."
    kind: regression
    risk: high
    verify: cargo test -p cap command_planner::tests::cd_prefix_grep_replans_native
  flag_bearing_rg_in_pipe_falls_back_to_bash:
    id: R5
    text: "A flag-bearing rg segment participating in a pipe (e.g. 'rg -n pattern file | wc -l') is not translated or fused by this TD's changes and the whole pipeline falls back to bash unchanged (explicit, documented out-of-scope boundary, not a regression)."
    kind: regression
    risk: medium
    verify: cargo test -p cap command_planner::tests::flag_bearing_rg_in_pipe_falls_back_to_bash
  full_command_planner_suite_unaffected:
    id: R6
    text: "The full pre-existing command_planner test suite passes with no new failures relative to a pre-change baseline, proving the new flag-aware grep/rg native path is purely additive (AC4)."
    kind: regression
    risk: high
    verify: cargo test -p cap command_planner
  grep_combined_short_flags_cluster_recognized:
    id: R1
    text: "'grep -ni <literal> <file>' (combined short-flag cluster: line numbers + ignore-case) is recognized and native-plans in-process, applying both flags together."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_combined_short_flags_cluster_recognized
  grep_context_after_standalone_numeric_token_recognized:
    id: R1
    text: "'grep -A 2 <literal> <file>' (standalone flag token followed by a numeric token) is recognized and native-plans in-process, emitting the matched line plus 2 trailing context lines."
    kind: functional
    risk: high
    verify: cargo test -p cap command_planner::tests::grep_context_after_standalone_numeric_token_recognized
  grep_context_before_attached_digit_form_recognized:
    id: R1
    text: "'grep -B2 <literal> <file>' (attached-digit form) is recognized and native-plans in-process, emitting 2 leading context lines plus the matched line."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_context_before_attached_digit_form_recognized
  grep_context_both_c_flag_recognized:
    id: R1
    text: "'grep -C 1 <literal> <file>' is recognized and native-plans in-process, emitting 1 line of context on each side of the match."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_context_both_c_flag_recognized
  grep_context_flag_combined_with_other_short_flag_falls_back:
    id: R4
    text: "'grep -A3n <literal> <file>' (an -A/-B/-C context flag fused with another short flag in the same cluster) is NOT accepted by the flag parser and the whole line falls back to bash unchanged (documented scope boundary)."
    kind: regression
    risk: high
    verify: cargo test -p cap command_planner::tests::grep_context_flag_combined_with_other_short_flag_falls_back
  grep_count_flag_recognized:
    id: R1
    text: "'grep -c <literal> <file>' is recognized and native-plans in-process, emitting only the match count."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_count_flag_recognized
  grep_directory_as_file_arg_falls_back:
    id: R4
    text: "'grep -n <literal> <directory>' (the file argument is a directory, not a regular file) fails Path::is_file() and falls back to bash unchanged; no recursive support."
    kind: regression
    risk: high
    verify: cargo test -p cap command_planner::tests::grep_directory_as_file_arg_falls_back
  grep_files_with_matches_flag_recognized:
    id: R1
    text: "'grep -l <literal> <file>' is recognized and native-plans in-process, emitting only the file name when it contains a match."
    kind: functional
    risk: low
    verify: cargo test -p cap command_planner::tests::grep_files_with_matches_flag_recognized
  grep_glob_flag_falls_back:
    id: R4
    text: "'grep --glob=*.rs <literal> <dir>' and 'grep -g *.rs <literal> <file>' are outside the accepted vocabulary and fall back to bash unchanged."
    kind: regression
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_glob_flag_falls_back
  grep_ignore_case_flag_recognized:
    id: R1
    text: "'grep -i <literal> <file>' is recognized and native-plans in-process, matching case-insensitively."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_ignore_case_flag_recognized
  grep_invert_match_flag_recognized:
    id: R1
    text: "'grep -v <literal> <file>' is recognized and native-plans in-process, emitting only non-matching lines."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_invert_match_flag_recognized
  grep_line_number_flag_recognized:
    id: R1
    text: "'grep -n <literal> <file>' is recognized and native-plans in-process, and run_grep_file emits grep-compatible '<line>:<text>' output for the matched lines."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_line_number_flag_recognized
  grep_more_than_one_file_argument_falls_back:
    id: R4
    text: "'grep -n <literal> <file1> <file2>' (more than one file argument) falls back to bash unchanged; the parser accepts at most one file token."
    kind: regression
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_more_than_one_file_argument_falls_back
  grep_multiline_mode_flags_fall_back:
    id: R4
    text: "'-u', '-p', '--json', and '-U' are outside the accepted vocabulary for both grep and rg and each falls back to bash unchanged."
    kind: regression
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_multiline_mode_flags_fall_back
  grep_only_matching_flag_recognized:
    id: R1
    text: "'grep -o <literal> <file>' is recognized and native-plans in-process, emitting only the matched substring per line."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_only_matching_flag_recognized
  grep_pcre_only_pattern_falls_back:
    id: R2
    text: "A pattern containing a regex metacharacter such as '+', '(', or '|' fails is_plain_literal_pattern and the whole invocation falls back to bash unchanged (no regex-dialect expansion in this TD)."
    kind: regression
    risk: high
    verify: cargo test -p cap command_planner::tests::grep_pcre_only_pattern_falls_back
  grep_recursive_flags_fall_back:
    id: R4
    text: "'grep -r <literal> <dir>' and 'grep -R <literal> <dir>' are outside the accepted vocabulary and fall back to bash unchanged."
    kind: regression
    risk: high
    verify: cargo test -p cap command_planner::tests::grep_recursive_flags_fall_back
  grep_repeated_e_multi_pattern_or_matches:
    id: R1
    text: "'grep -e pat1 -e pat2 <file>' collects both patterns into GrepFilePlan.patterns and OR-combines them: a line matches if either literal pattern is present."
    kind: functional
    risk: high
    verify: cargo test -p cap command_planner::tests::grep_repeated_e_multi_pattern_or_matches
  grep_type_and_include_flags_fall_back:
    id: R4
    text: "'grep --type rust <literal> <file>' and 'grep --include=*.rs <literal> <file>' are outside the accepted vocabulary and fall back to bash unchanged."
    kind: regression
    risk: medium
    verify: cargo test -p cap command_planner::tests::grep_type_and_include_flags_fall_back
  rg_context_after_numeric_alias_recognized:
    id: R3
    text: "'rg -A 3 <literal> <file>' is recognized via the rg alias and native-plans in-process, matching AC2's rg -A 3 example."
    kind: functional
    risk: high
    verify: cargo test -p cap command_planner::tests::rg_context_after_numeric_alias_recognized
  rg_count_alias_recognized:
    id: R3
    text: "'rg -c <literal> <file>' is recognized via the rg alias and native-plans in-process, matching AC2's rg -c example."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::rg_count_alias_recognized
  rg_line_number_alias_recognized:
    id: R3
    text: "'rg -n <literal> <file>' is recognized via the rg alias dispatch arm, normalized onto the same accepted vocabulary, and native-plans in-process identically to the grep equivalent."
    kind: functional
    risk: high
    verify: cargo test -p cap command_planner::tests::rg_line_number_alias_recognized
  rg_no_heading_cosmetic_flag_consumed_harmlessly:
    id: R3
    text: "'rg --no-heading -n <literal> <file>' is recognized via the rg alias: the cosmetic --no-heading token is consumed harmlessly (does not disqualify) and -n still applies, matching AC2's --no-heading example."
    kind: functional
    risk: medium
    verify: cargo test -p cap command_planner::tests::rg_no_heading_cosmetic_flag_consumed_harmlessly
  rg_repeated_e_multi_pattern_alias_recognized:
    id: R3
    text: "'rg -n -e pat1 -e pat2 <file>' is recognized via the rg alias: both -e patterns are collected and OR-combined, and -n applies, matching AC2's multi-pattern rg example."
    kind: functional
    risk: high
    verify: cargo test -p cap command_planner::tests::rg_repeated_e_multi_pattern_alias_recognized
  rg_unsupported_specific_flag_falls_back:
    id: R3
    text: "An rg-specific flag outside the shared vocabulary and the cosmetic allowlist (e.g. '--json') disqualifies the whole rg invocation and it falls back to bash unchanged."
    kind: regression
    risk: medium
    verify: cargo test -p cap command_planner::tests::rg_unsupported_specific_flag_falls_back
  zero_flag_grep_pipe_fusion_unaffected:
    id: R5
    text: "Existing zero-flag literal grep pipe-fusion behavior (e.g. plan_head_grep_producer_mode and sibling match arms keyed on the literal 'grep' token) is byte-for-byte unaffected by the new flag-parsing branch and the rg dispatch arm."
    kind: regression
    risk: high
    verify: cargo test -p cap command_planner::tests::zero_flag_grep_pipe_fusion_unaffected
  zero_flag_rg_pipe_fusion_translates_to_grep:
    id: R5
    text: "A bare zero-flag 'rg <pattern> [file]' pipeline segment is translated to the literal 'grep' token at the pipe-segment word-list assembly point in plan_shell, so the existing grep-keyed pipe-fusion match-arm family fuses it exactly as it would fuse the equivalent grep invocation."
    kind: functional
    risk: high
    verify: cargo test -p cap command_planner::tests::zero_flag_rg_pipe_fusion_translates_to_grep
---
flowchart TD
    r1[R1 grep combined short flags cluster recognized] --> cargo_test_p_cap_command_planner_tests_grep_combined_short_flags_cluster_recognized[cargo test -p cap command_planner::tests::grep_combined_short_flags_cluster_recognized]
    r1[R1 grep context after standalone numeric token recognized] --> cargo_test_p_cap_command_planner_tests_grep_context_after_standalone_numeric_token_recognized[cargo test -p cap command_planner::tests::grep_context_after_standalone_numeric_token_recognized]
    r1[R1 grep context before attached digit form recognized] --> cargo_test_p_cap_command_planner_tests_grep_context_before_attached_digit_form_recognized[cargo test -p cap command_planner::tests::grep_context_before_attached_digit_form_recognized]
    r1[R1 grep context both c flag recognized] --> cargo_test_p_cap_command_planner_tests_grep_context_both_c_flag_recognized[cargo test -p cap command_planner::tests::grep_context_both_c_flag_recognized]
    r1[R1 grep count flag recognized] --> cargo_test_p_cap_command_planner_tests_grep_count_flag_recognized[cargo test -p cap command_planner::tests::grep_count_flag_recognized]
    r1[R1 grep files with matches flag recognized] --> cargo_test_p_cap_command_planner_tests_grep_files_with_matches_flag_recognized[cargo test -p cap command_planner::tests::grep_files_with_matches_flag_recognized]
    r1[R1 grep ignore case flag recognized] --> cargo_test_p_cap_command_planner_tests_grep_ignore_case_flag_recognized[cargo test -p cap command_planner::tests::grep_ignore_case_flag_recognized]
    r1[R1 grep invert match flag recognized] --> cargo_test_p_cap_command_planner_tests_grep_invert_match_flag_recognized[cargo test -p cap command_planner::tests::grep_invert_match_flag_recognized]
    r1[R1 grep line number flag recognized] --> cargo_test_p_cap_command_planner_tests_grep_line_number_flag_recognized[cargo test -p cap command_planner::tests::grep_line_number_flag_recognized]
    r1[R1 grep only matching flag recognized] --> cargo_test_p_cap_command_planner_tests_grep_only_matching_flag_recognized[cargo test -p cap command_planner::tests::grep_only_matching_flag_recognized]
    r1[R1 grep repeated e multi pattern or matches] --> cargo_test_p_cap_command_planner_tests_grep_repeated_e_multi_pattern_or_matches[cargo test -p cap command_planner::tests::grep_repeated_e_multi_pattern_or_matches]
    r2[R2 grep pcre only pattern falls back] --> cargo_test_p_cap_command_planner_tests_grep_pcre_only_pattern_falls_back[cargo test -p cap command_planner::tests::grep_pcre_only_pattern_falls_back]
    r3[R3 rg context after numeric alias recognized] --> cargo_test_p_cap_command_planner_tests_rg_context_after_numeric_alias_recognized[cargo test -p cap command_planner::tests::rg_context_after_numeric_alias_recognized]
    r3[R3 rg count alias recognized] --> cargo_test_p_cap_command_planner_tests_rg_count_alias_recognized[cargo test -p cap command_planner::tests::rg_count_alias_recognized]
    r3[R3 rg line number alias recognized] --> cargo_test_p_cap_command_planner_tests_rg_line_number_alias_recognized[cargo test -p cap command_planner::tests::rg_line_number_alias_recognized]
    r3[R3 rg no heading cosmetic flag consumed harmlessly] --> cargo_test_p_cap_command_planner_tests_rg_no_heading_cosmetic_flag_consumed_harmlessly[cargo test -p cap command_planner::tests::rg_no_heading_cosmetic_flag_consumed_harmlessly]
    r3[R3 rg repeated e multi pattern alias recognized] --> cargo_test_p_cap_command_planner_tests_rg_repeated_e_multi_pattern_alias_recognized[cargo test -p cap command_planner::tests::rg_repeated_e_multi_pattern_alias_recognized]
    r3[R3 rg unsupported specific flag falls back] --> cargo_test_p_cap_command_planner_tests_rg_unsupported_specific_flag_falls_back[cargo test -p cap command_planner::tests::rg_unsupported_specific_flag_falls_back]
    r4[R4 grep context flag combined with other short flag falls back] --> cargo_test_p_cap_command_planner_tests_grep_context_flag_combined_with_other_short_flag_falls_back[cargo test -p cap command_planner::tests::grep_context_flag_combined_with_other_short_flag_falls_back]
    r4[R4 grep directory as file arg falls back] --> cargo_test_p_cap_command_planner_tests_grep_directory_as_file_arg_falls_back[cargo test -p cap command_planner::tests::grep_directory_as_file_arg_falls_back]
    r4[R4 grep glob flag falls back] --> cargo_test_p_cap_command_planner_tests_grep_glob_flag_falls_back[cargo test -p cap command_planner::tests::grep_glob_flag_falls_back]
    r4[R4 grep more than one file argument falls back] --> cargo_test_p_cap_command_planner_tests_grep_more_than_one_file_argument_falls_back[cargo test -p cap command_planner::tests::grep_more_than_one_file_argument_falls_back]
    r4[R4 grep multiline mode flags fall back] --> cargo_test_p_cap_command_planner_tests_grep_multiline_mode_flags_fall_back[cargo test -p cap command_planner::tests::grep_multiline_mode_flags_fall_back]
    r4[R4 grep recursive flags fall back] --> cargo_test_p_cap_command_planner_tests_grep_recursive_flags_fall_back[cargo test -p cap command_planner::tests::grep_recursive_flags_fall_back]
    r4[R4 grep type and include flags fall back] --> cargo_test_p_cap_command_planner_tests_grep_type_and_include_flags_fall_back[cargo test -p cap command_planner::tests::grep_type_and_include_flags_fall_back]
    r5[R5 flag bearing rg in pipe falls back to bash] --> cargo_test_p_cap_command_planner_tests_flag_bearing_rg_in_pipe_falls_back_to_bash[cargo test -p cap command_planner::tests::flag_bearing_rg_in_pipe_falls_back_to_bash]
    r5[R5 zero flag grep pipe fusion unaffected] --> cargo_test_p_cap_command_planner_tests_zero_flag_grep_pipe_fusion_unaffected[cargo test -p cap command_planner::tests::zero_flag_grep_pipe_fusion_unaffected]
    r5[R5 zero flag rg pipe fusion translates to grep] --> cargo_test_p_cap_command_planner_tests_zero_flag_rg_pipe_fusion_translates_to_grep[cargo test -p cap command_planner::tests::zero_flag_rg_pipe_fusion_translates_to_grep]
    r6[R6 cd prefix suite unaffected by grep rg changes] --> cargo_test_p_cap_command_planner_tests_cd_prefix_grep_replans_native[cargo test -p cap command_planner::tests::cd_prefix_grep_replans_native]
    r6[R6 full command planner suite unaffected] --> cargo_test_p_cap_command_planner[cargo test -p cap command_planner]
```
