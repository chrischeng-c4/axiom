<!-- HANDWRITE-BEGIN gap="missing-generator:contract:44cc6028" tracker="pending-tracker" reason="Stable-only skill contract." -->
---
name: workbench-build-stable
description: Build and launch only the stable Axiom Workbench macOS app with its isolated stable runtime profile and bundled Rust sidecar.
---

# Build Axiom Workbench Stable

Run the dispatcher without arguments:

```bash
.agents/skills/workbench-build-stable/scripts/build.sh
```

It builds `workbench-core`, builds the Xcode Release product `Axiom Workbench.app`, stops only a running Stable executable from that exact app bundle, and launches that bundle. It never starts, replaces, or reads the Beta product.

Report the built app path. On failure, report the failing build phase and preserve its output. Do not install, commit, or change user project metadata.

<!-- marker: missing-generator:contract:44cc6028 path: .agents/skills/workbench-build-stable/SKILL.md reason: Stable-only skill contract. -->
<!-- HANDWRITE-END -->
