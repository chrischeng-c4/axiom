---
change: jet-workspace-protocol
group: jet-workspace-protocol
date: 2026-03-24
status: answered
---

# Pre-Clarifications

### Q1: pnpm-workspace.yaml detection priority
- **Answer**: jet-workspace.yaml > package.json.workspaces > pnpm-workspace.yaml. Jet's own format always takes precedence; pnpm-workspace.yaml is the lowest-priority fallback.

### Q2: pnpm-workspace.yaml catalog support
- **Answer**: Parse both packages: glob array AND catalog:/catalogs: fields from pnpm-workspace.yaml. Wire catalog support in this change, not a follow-up.

### Q3: Symlink style: relative vs absolute
- **Answer**: Relative symlinks (like pnpm). Compute relative path from node_modules/<pkg-name> to the workspace package directory.

### Q4: Recursive workspace install
- **Answer**: Recursive install. When jet install runs at workspace root, iterate all discovered workspace packages and install each one's external (non-workspace) deps. One command installs everything.

### Q5: Lockfile representation for workspace packages
- **Answer**: Record workspace packages in jet-lock.yaml with workspace: true flag + relative path + resolved semver version. Full traceability.

