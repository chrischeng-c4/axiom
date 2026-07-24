# OpenAPI codegen EC evidence

The hand-authored Python oracle writes four result files:

- `identity.json` — Cargo/package/crate/version/publish/sidecar/tag identity.
- `references.json` — bounded active-tree scan with the exact historical allowlist.
- `matrix.json` — the complete named language/profile/determinism matrix.
- `consumers.json` — six application checks plus the transport-policy example.

Every file uses `openapi-codegen.ec-evidence.v1`. The validator rejects missing
or extra check ids, duplicate ids, failed or skipped statuses, zero observation
counts, and an inconsistent total. Evidence is overwritten by the oracle on
every run; these files are not hand-authored success declarations.
