<!-- HANDWRITE-BEGIN gap="missing-generator:contract:266ef16b" tracker="pending-tracker" reason="Beta-only skill contract." -->
---
name: workbench-build-beta
description: Build and launch only the Axiom Workbench Beta macOS app with its isolated beta runtime profile and bundled Rust sidecar.
---

# Build Axiom Workbench Beta

Run the dispatcher without arguments:

```bash
.agents/skills/workbench-build-beta/scripts/build.sh
```

It builds `workbench-core`, builds the Xcode Debug product `Axiom Workbench Beta.app`, stops only a running Beta executable from that exact app bundle, and launches that bundle. It never starts, replaces, or reads the Stable product.

Report the built app path. On failure, report the failing build phase and preserve its output. Do not install, commit, or change user project metadata.

<!-- marker: missing-generator:contract:266ef16b path: .agents/skills/workbench-build-beta/SKILL.md reason: Beta-only skill contract. -->
<!-- HANDWRITE-END -->
