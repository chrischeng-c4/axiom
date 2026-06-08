---
change_id: envfile-support
type: gap_spec_knowledge
created_at: 2026-02-10T02:29:25.398086+00:00
updated_at: 2026-02-10T02:29:25.398086+00:00
---

# Gap Analysis: Spec vs Knowledge

## Identified Gaps

1. **Standardization of Env Support**:
    - **Knowledge**: The project seems to be standardizing on `dotenvy` for Rust and `python-dotenv` for Python.
    - **Spec**: `shield-settings-management` documents this well for `cclab-shield`.
    - **Gap**: There is no central "configuration" or "environment" spec that covers how *all* `cclab` tools should handle `.env` files. This change (`envfile-support`) will effectively establish this pattern for `cclab-genesis`.
