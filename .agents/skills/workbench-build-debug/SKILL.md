---
name: workbench-build-debug
description: Compatibility entry point for building and launching only Axiom Workbench Beta. Prefer workbench-build-beta for new requests.
---

# Build Axiom Workbench Beta (compatibility entry point)

Run the dispatcher without arguments:

```bash
.agents/skills/workbench-build-debug/scripts/build.sh
```

This compatibility dispatcher invokes the Beta-only build skill. It does not
launch Stable and never targets the retired `com.cclab.workbench` product.

Report the app path and the result. On failure, report the failing build phase
and preserve its output. Do not commit, install, or modify user project metadata.
