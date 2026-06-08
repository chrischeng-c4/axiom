# Mamba MVP Validation Inventory

This directory is the staging home for the canonical list of what Mamba must
validate before the MVP can be called ready.

Issues track work progress. The files in this directory track the validation
contracts themselves: what is tested, which profile requires it, where the
fixture or runner lives, what oracle decides pass/fail, and which work item owns
missing or blocked coverage.

The tech design will be backfilled later. Until then, this directory is the
working source of truth for the validation inventory shape and profile grouping.

## Files

- `mvp.toml` defines the top-level MVP profiles and references the current
  project-local manifests.
- Existing manifests outside this directory remain authoritative until they are
  migrated or included here:
  - `../tests/cpython/lib_test_seeds/` — folder-based contract (#3729);
    parent directory IS the pinned outcome (no TOML).
  - `../ecosystem_fixture_manifest.toml`
  - `../baseline.json`

## Item Rules

Each validation item should eventually have:

- `id`: stable dotted id, for example `lang.scope.nonlocal.001`.
- `category`: one of the profile categories in `mvp.toml`.
- `required`: whether the item blocks MVP release.
- `source_of_truth`: CPython 3.12 docs, CPython Lib/test, MVP spec, or local
  mamba contract.
- `oracle`: how pass/fail is decided.
- `fixture`: path to the test fixture or runner entry.
- `worker_issue`: issue number for missing, blocked, or implementation work.
- `status`: `missing`, `implemented`, `passing`, or `blocked`.

Release-required items must not count `skip`, `xfail`, `Stub`, or `ImportPass`
as passing.
