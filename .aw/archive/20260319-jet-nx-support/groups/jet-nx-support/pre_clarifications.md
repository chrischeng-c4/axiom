---
change: jet-nx-support
group: jet-nx-support
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: Workspace Detection
- **Answer**: Primary detection should rely on 'nx.json' at the workspace root. Secondary checks include 'node_modules/.bin/nx' and '@nx/workspace' in 'package.json'.

### Q2: Project Graph Integration
- **Answer**: Querying the Nx graph via 'nx graph --json' is the preferred method to ensure consistency with Nx's internal logic.

### Q3: Task Pipeline
- **Answer**: Standard target names like 'build' and 'install' should be used. Users can configure these in their 'project.json' files using a Jet-specific executor.

### Q4: Build/Install Context
- **Answer**: 'jet build' should respect the Nx graph and trigger builds for dependent Jet projects if they are out of date.

### Q5: Configuration
- **Answer**: Custom settings should be supported in 'cclab/config.toml' under an '[nx]' section.

