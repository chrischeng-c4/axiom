---
topic: sdd-pipeline-upgrade-arc
date: '20260427'
project: main
branch: main
---

## Status

Track 1 (dogfood + gap fixes) closed end-to-end on main: 4 issues merged (A/B/Z/D). Track 2 (chat verb + SDD pipeline upgrade) split into 2 issues (C/E) with specs authored + reviewed but **NOT merged** — implementations require multi-week arc, not session-scope. Coverage now 227/233 on-target sdd files (97%) with 2 explicitly skip-flagged via `@codegen-skip`. 1778 sdd lib tests pass.

## Findings

- **27%/73% codegen split is real** for IO/parser/formatter-heavy CLI verbs (chat.rs case study: 700 LOC, 190 codegen-able). User's principle is "100% codegen via Mermaid Plus" — not yet realized; gap is in YAML primitive vocabulary + logic generator, not in Mermaid syntax.
- **Mermaid Plus YAML is the source of truth** — Mermaid syntax + Rust code are both DERIVATIVES. Renderer (YAML→Mermaid) works; emitter (YAML→Rust) is incomplete. Architecture is sound; the `primitive:` field on flowchart nodes is the missing bridge.
- **gen-code's "modify existing file" mode keeps failing** on hand-written Rust files (Issues A, B, C, D, E all hit this). gen-code dumb-appends a CODEGEN block at file end with `todo!()` placeholders or wrong-place inserts. The fix is mainthread takeover; the recurring pattern means gen-code needs a "modify-aware" mode (separate enhancement).
- **gen-code on doc-only changes (CLAUDE.md edit)** also fails: writes a CODEGEN block with TODO content instead of applying the prose replace. Hand-apply is the workaround.
- **`required:` lists in spec YAML** must include every Vec/bool/non-Option-with-default field, otherwise auto-Option-wrap produces type mismatches. Fixed at generator level by Issue A's R11 (collection-type bypass) — future dogfood specs no longer need the workaround.
- **`#[serde(default)]` on no-serde-derive structs** (e.g. IssuePatch with only `#[derive(Default)]`) used to fail compile; fixed at generator level by Issue A's R12 (gate explicit_default_bool on struct_has_serde).
- **Two false-positive dogfood candidates** (`crates/sdd/src/tools/analyze/mod.rs`, `projects/score/cli/src/fillback.rs`) — `pub struct/enum` matches inside `r#"..."#` raw-string fixtures. Marked with `//! @codegen-skip: test-fixture-only` annotation per Issue D. Future scans filter these out via `grep -L 'CODEGEN-BEGIN\|@codegen-skip'`.
- **Cross-WT discussion via /tmp/ files works**: validated via `/tmp/score-td-fitness-for-mamba-conformance.md` exchange with mamba team. The pattern (structured ask + tagged verdict reply) is concise + auditable. Issue C formalizes this as `score chat` verb (5 subcommands) but blocks on Issue E.

## Done

Merged into main this session (in order):

- **Issue A** (`enhancement-bundle-gap-a-vec-collection-bypass-option-auto-wra`) — R11 + R12 in `crates/sdd/src/generate/gen/rust/schema.rs`. Vec/HashMap/BTreeMap/HashSet/BTreeSet bypass Option auto-wrap; `explicit_default_bool` gated on `struct_has_serde`. Commit: `faa7682ee` then merged.
- **Issue B** (`refactor-dogfood-crates-sdd-src-models-change-rs-17-types-n`) — change.rs (1011 lines, 17 types) wrapped in CODEGEN block. Verifies `x-serde-skip: "serializing"` (R9 string variant). Commit: `e58344805` then merged.
- **Issue Z** (`enhancement-update-claude-md-file-size-rule-for-codegen-era-ta`) — CLAUDE.md `## Constraints` updated with three-class file-size rule (spec markdown / hand-written source / generated source). Commit: `428ffbaa6` then merged.
- **Issue D** (`enhancement-mark-test-fixture-only-files-with-codegen-skip-ann`) — `//! @codegen-skip` annotation on analyze/mod.rs + fillback.rs + CLAUDE.md scan-convention bullet. Commit: `bcbe22dc7` then merged.

Authored + reviewed but **NOT merged** (worktrees alive):

- **Issue C** (`enhancement-add-score-chat-verb-for-cross-wt-agent-discussion`) — TD spec at `.score/worktrees/td-enhancement-add-score-chat-verb-for-cross-wt-agent-discussion/.score/tech_design/projects/score/specs/score-cli-chat.md` (538 lines). Describes 5 subcommands: post/list/read/agents/listen + auto-detect team identity from `.score/config.toml [team]` block. Implementation BLOCKED on Issue E.
- **Issue E** (`enhancement-extend-yaml-schema-with-primitive-vocabulary-rust`) — TD specs at `.score/worktrees/td-enhancement-extend-yaml-schema-with-primitive-vocabulary-rust/.score/tech_design/projects/score/specs/mermaid-plus-primitive-vocabulary.md` (685 lines, 30 primitives across 7 categories) + `crates/sdd/generate/generators/logic-primitive-emitter.md` (345 lines). Defines vocabulary + emit algorithm.

Other work this session:
- 3 dogfood batches from the prior 62.12% plateau (frontmatter.rs / issues/types.rs / spec_ir/types.rs) — already merged before this Track 1/2 arc began. Total dogfood coverage: 227 files at session end.
- Memory updates: should add `project_sdd_pipeline_codegen_27_73_split.md` (the 27%/73% finding), `project_mermaid_plus_yaml_source_of_truth.md` (architecture clarification), `feedback_codebase_under_td_principle.md` (user's "everything under TD scope" stance).

## Next

Priority order — resume as a multi-week arc, not a single session:

1. **Implement Issue E's primitive vocabulary + logic emitter** (~1500-2500 LOC of generator code). Substantial. Sub-divide if needed:
   - Sub-issue E1: define `PrimitiveKind` enum + registry struct (codegen-able from spec's schema section)
   - Sub-issue E2: extend Mermaid Plus YAML schema (`cclab-agkit/schemas/`) with `primitive:` field
   - Sub-issue E3: file-IO primitives (5 entries: read_file, write_file, append_file, path_exists, walk_up_to_marker)
   - Sub-issue E4: serde primitives (6 entries: parse/serialize × yaml/json/toml + parse_markdown)
   - Sub-issue E5: format/string primitives (3-4 entries)
   - Sub-issue E6: control/iterator primitives (5 entries: for_each, reduce, find, filter, sort_by)
   - Sub-issue E7: time + IO surface + generic (~10 entries)
   - Sub-issue E8: emit algorithm (topological sort + per-node dispatch + variable binding) in `LogicPrimitiveEmitter`
   - Sub-issue E9: regression test against the 4 already-merged dogfood specs (byte-equivalent gen-code output)

2. **Re-author Issue C's TD spec** with primitive-labeled flowchart YAML for each of the 5 subcommands (post/list/read/agents/listen). Each handler becomes a Mermaid Plus flowchart of primitive nodes. Replace existing logic section with rich per-fn flowcharts.

3. **Run gen-code on Issue C** — should now emit 100% of `chat.rs` from primitive YAML. Verify with `cargo build -p score-cli` + `score chat --help` smoke test.

4. **Merge Issue C** + clean up worktree.

5. **Optional follow-ups discovered this session**:
   - Issue: gen-code "modify existing file" mode produces todo!() skeletons; should detect existing types in the target file and replace rather than append. Affects all generator-on-existing-file dogfood (which is now most of them).
   - Issue: gen-code on doc-only changes should apply the prose replace from `## Changes`'s `old:`/`new:` fields, not write a CODEGEN block.
   - Issue: TD-author subagent picks wrong spec path conventions (e.g. proposes `models/spec_ir_types.md` instead of mirroring source path `crates/sdd/generate/spec_ir/types.md`). Agent prompt fix in `.claude/agents/score-td-author.md`.

Resume command:

```bash
score takeoff --latest --json
```

If continuing Issue E implementation, start with:

```bash
cd /Users/chris.cheng/cclab/main/.score/worktrees/td-enhancement-extend-yaml-schema-with-primitive-vocabulary-rust
git rebase main  # pull in this session's merged work
# then implement per the spec's `## Changes` section
```

## Criteria

- [ ] All 4 Track 1 issues merged: `git log --oneline | grep "Merge tech-design td-enhancement-bundle-gap-a\|td-refactor-dogfood-crates-sdd-src-models-change-rs\|td-enhancement-update-claude-md-file-size-rule\|td-enhancement-mark-test-fixture-only-files-with-codegen-skip-ann" | wc -l` returns 4
- [ ] Tests pass: `cargo test -p sdd --lib`
- [ ] Score CLI still builds: `cargo build -p score-cli`
- [ ] Dogfood scan returns 0 unmarked candidates: `find crates/sdd/src crates/score/cli/src projects/score/cli/src -name "*.rs" 2>/dev/null | xargs grep -L 'CODEGEN-BEGIN\|@codegen-skip' 2>/dev/null | xargs grep -lE "^pub (struct|enum) " 2>/dev/null | wc -l` returns 0
- [ ] CLAUDE.md has three-class file-size rule: `grep -c "Spec markdown.*consider split if >= 500" CLAUDE.md` returns 1
- [ ] CLAUDE.md has @codegen-skip convention: `grep -c "@codegen-skip" CLAUDE.md` returns 1
- [ ] Issue C worktree exists with TD spec authored: `test -d .score/worktrees/td-enhancement-add-score-chat-verb-for-cross-wt-agent-discussion && test -f .score/worktrees/td-enhancement-add-score-chat-verb-for-cross-wt-agent-discussion/.score/tech_design/projects/score/specs/score-cli-chat.md`
- [ ] Issue E worktree exists with both TD specs authored: `test -d .score/worktrees/td-enhancement-extend-yaml-schema-with-primitive-vocabulary-rust && test -f .score/worktrees/td-enhancement-extend-yaml-schema-with-primitive-vocabulary-rust/.score/tech_design/projects/score/specs/mermaid-plus-primitive-vocabulary.md`
- [ ] Manual: principle alignment — every fn in any future hand-written code must have `/// @spec ...` annotation pointing to a TD section, OR be inside a CODEGEN-BEGIN/END block. No bare hand-written code.
