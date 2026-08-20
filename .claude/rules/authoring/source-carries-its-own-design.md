# Source carries its own design

## Intent

Keep the authoring surface and the artifact the same file, so a design decision
cannot drift from the code it governs while both still look maintained.

## Rules

- Write the design into the `.rs` file that owns it. A module's `//!` block
  carries the rules that module owns; a type's `///` block carries its own.
- Do not create a `tech-design/` or an `external-contracts/` directory under a
  project the TD/EC retirement emptied. The fifteen are `apps/lumen` and
  `libs/{build-stamp, cli-std, metrics-prometheus, openapi-codegen, peer-tls,
  raft-core, raft-runtime, service-auth, service-backup, service-http,
  service-k8s, service-observability, storage-durable, transport-h2c}`.
- Do not write an `@spec` line in those projects. That marker is the retired
  mechanism's own syntax — it named the design document a file was generated
  from — and the generator behind it is deleted, so a surviving one points at a
  document nothing maintains.
- Prose that mentions a retired tree is not the regression. Several files in
  these projects record that the tree is gone, and saying so is what a `//!`
  block is for.
- This rule is scoped to those fifteen and not to the repository. That is a
  measurement: at the commit that introduced the gate below, 560 design-tree
  files were still tracked across 26 other owners — `apps/tape`, `apps/pgpool`,
  `apps/jet`, `projects/mamba` and the rest. None of them is covered here, and a
  repository-wide assertion would have been red on its first run.

## Verification

- Run `cargo test -p lumen --test design_trees_stay_retired`. Three cases: the
  tree assertion is structural and reads no file contents, the pointer assertion
  reads every text file under the fifteen, and `the_sweep_is_not_vacuous`
  measures the instrument — every project root must resolve, every exemption
  must resolve to a real file, and the walk must reach at least 400 files.
- That command is a strict subset of `cargo test -p lumen`, which
  `apps/lumen/CONTRIBUTING.md:18` declares as the project gate. The gate is what
  refuses a violation; this file only says why.
- The gate deliberately lives in `apps/lumen/e2e/` rather than
  `plugins/aw/verification/`. Nothing in this repository calls
  `plugins/aw/verification/run_all.py` — no CI workflow, no git hook, no phase
  script — so a check registered there is refused by nobody. Before moving this
  gate, find the caller first.
- All three cases were proven able to fail before being accepted: a planted
  `libs/cli-std/tech-design` directory, the three `@spec` lines the retirement
  left in `libs/service-http`, `libs/service-k8s` and `libs/transport-h2c`, and
  a mistyped project root. Each was restored and the test file verified
  byte-identical by sha256 (`b81dedee…1a2a`).
- Nothing regenerates this file. It is hand-maintained, like every other file
  under `.claude/rules/`, so a rule here that has stopped being true stays in
  every session's context until a human deletes it.

## References

- `CLAUDE.md` section “Artifact write order”, which states that
  `external-contracts/` and `tech-design/` are not write roots and that the
  `.rs` file is the authoring surface.
- `CLAUDE.md` section “Test Layout”, which supersedes those trees along with
  `tests/`.
- `apps/lumen/docs/td-ec-retirement.md` — the campaign that emptied the fifteen,
  and the thirty-four hazards it measured on the way. Hazards 29, 30, 31 and 33
  are four instances of one failure: a gate whose scan was narrower than its
  declaration. `the_sweep_is_not_vacuous` exists because of them.
- `.claude/rules/authoring/artifact-layout.md` for the file-shape principle this
  rule composes with.
