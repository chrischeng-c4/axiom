---
name: workbench-build-debug
description: Build and launch the current native macOS Workbench debug app, including its Rust PTY sidecar. Use when the user asks to build, rebuild, run, relaunch, or inspect the local Workbench desktop application.
---

# /workbench:build:debug

Run the dispatcher without arguments:

```bash
.agents/skills/workbench-build-debug/scripts/build.sh
```

The dispatcher builds `workbench-core`, builds the Xcode `Workbench.app` that
macOS launches for `com.cclab.workbench`, embeds the exact sidecar in the app,
and opens that app bundle. Do not substitute a SwiftPM executable: it has the
same sources but is a different launch surface and can drift from the Xcode
application.

Report the app path and the result. On failure, report the failing build phase
and preserve its output. Do not commit, install, or modify user project metadata.
