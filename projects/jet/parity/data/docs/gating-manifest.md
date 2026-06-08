# Jet parity — CI gating manifest

Issue: [#2144](https://github.com/chrischeng-c4/cclab/issues/2144) (parity / foundation).
Parent epic: [#2133](https://github.com/chrischeng-c4/cclab/issues/2133).

## What this is

`parity-gating.toml` is the single source of truth that drives the
jet parity CI gate. It declares:

- which **channels** are in scope (`pixel`, `ax-tree`, `focus-order`,
  `pointer-hit-map`, `ime-trace`),
- per-channel **tolerances**,
- whether the gate is **blocking** (hard-fail = exit 1) or **soft**
  (advisory = exit 2),
- the **adapter id** under test, and
- whether **waivers** (time-boxed exceptions) are honoured.

A companion `waivers.toml` carries the time-boxed exceptions.

## File layout

```
projects/jet/data/parity/
├── parity-gating.toml      # the manifest (this doc)
├── waivers.toml            # time-boxed waivers
├── docs/
│   └── gating-manifest.md  # this file
└── fixtures/               # parity corpus (#2140)
```

The gate binary lives at `projects/jet/parity-gate/` and ships as the
`jet-parity-gate` crate.

## Manifest schema

```toml
# Five canonical channels — anything else is rejected at parse.
channels = ["pixel", "ax-tree", "focus-order", "pointer-hit-map", "ime-trace"]

[tolerance]
# Max per-pixel L2 delta. Integer 0..=255 (8-bit colour space).
pixel_delta     = 8
# Max fraction of a11y nodes that may differ. Float 0.0..=1.0.
a11y_diff_ratio = 0.05

# false → exit 2 on fail (soft / advisory)
# true  → exit 1 on fail (blocking / hard CI fail)
blocking      = false

# false → waivers.toml is ignored, every Fail counts
# true  → unexpired waivers downgrade Fail → Waived (exit 0 if all
#         remaining rows pass)
allow_waivers = true

[adapter]
# Adapter under test. One adapter per manifest.
id = "mui-react-dom"
```

## Waivers

`waivers.toml` is a list-of-tables. Each row silences exactly one
`(fixture_id, channel)` Fail, and **auto-expires** on `expires_on`
(inclusive). Expired rows are not deleted — they stay for the audit
trail and are simply ignored by the gate.

```toml
[[waivers]]
fixture_id = "mui-button-default"
channel    = "pixel"
expires_on = "2026-06-01"
reason     = "AA drift on macOS Metal; #2199"
```

## Exit codes (the contract)

The gate CLI returns:

| code | meaning                                                      |
|------|--------------------------------------------------------------|
| `0`  | all channels pass, OR every Fail is covered by an unexpired waiver |
| `1`  | blocking fail — `blocking = true` AND at least one un-waived Fail |
| `2`  | soft fail   — `blocking = false` AND at least one un-waived Fail |
| `77` | skipped — no `*.channel-result.json` files in `--results-dir` |

`77` is the autotools convention for "skipped". CI treats it as
green so the gate can land before the oracle is wired into CI
(see `.github/workflows/parity.yml`).

## How `init` scaffolds

```
cargo run -p jet-parity-gate -- init \
    --target-dir projects/jet/parity \
    [--force]
```

`init` writes three files: `parity-gating.toml`, `waivers.toml`,
`docs/gating-manifest.md`. Without `--force` it refuses to overwrite
an existing file. The scaffold matches what is checked in at HEAD —
running `init --force` over a clean tree is a no-op.

## Channel-result schema (R2)

The gate consumes one `*.channel-result.json` per (fixture, channel).
Each file is a single JSON object:

```jsonc
{
  "schema_version": 1,
  "fixture_id":     "mui-button-default",
  "channel":        "pixel",
  "status":         "fail",                // pass | fail | waived | skipped
  "diff_kind":      "pixel-l2",            // see below
  "diff_value":     12.0,                  // numeric magnitude
  "captured_at":    "2026-05-16T03:14:15Z",
  "waived_by":      null                   // populated when status=waived
}
```

`diff_kind` enum:

| value                       | applies to        |
|-----------------------------|-------------------|
| `pixel-l2`                  | pixel             |
| `ax-tree-node-count`        | ax-tree           |
| `focus-order-edit-distance` | focus-order       |
| `hit-map-cell-diff`         | pointer-hit-map   |
| `ime-event-seq-diff`        | ime-trace         |

## End-to-end flow (today)

```
oracle (#2139)         corpus (#2140)         gate (#2144 — this slice)
─────────────  ──────  ─────────────────────  ──────────────────────────
runs adapter       →   reads fixtures   →   reads channel-results
emits per-fixture       drives oracle         applies tolerances + waivers
*.channel-result.json   runs                  returns exit 0/1/2/77
```

Until the oracle is producing artifacts in CI, the workflow at
`.github/workflows/parity.yml` runs the gate against an empty dir
and accepts the resulting `77`.

## Pointers

- Gate crate:           `projects/jet/parity-gate/`
- Manifest:             `projects/jet/data/parity/parity-gating.toml`
- Waivers:              `projects/jet/data/parity/waivers.toml`
- Oracle (R1/R2 schema): `projects/jet/parity-oracle/`
- Corpus:               `projects/jet/parity-corpus/`
- CI workflow:          `.github/workflows/parity.yml`
