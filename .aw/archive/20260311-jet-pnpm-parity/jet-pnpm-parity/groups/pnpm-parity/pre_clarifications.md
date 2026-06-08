---
change: jet-pnpm-parity
group: pnpm-parity
date: 2026-03-11
status: answered
---

# Pre-Clarifications

### Q1: npmrc-scope
- **Answer**: All levels: project (.npmrc) → user (~/.npmrc) → global (/etc/npmrc), merged with precedence like pnpm/npm

### Q2: workspace-config
- **Answer**: Support both: package.json workspaces field (npm/yarn compat) + optional jet-workspace.yaml for advanced config (hoisting, catalog)

### Q3: frozen-lockfile-ci
- **Answer**: Auto-enable --frozen-lockfile when CI env vars detected (CI=true, GITHUB_ACTIONS, GITLAB_CI, etc.)

### Q4: dedup-granularity
- **Answer**: File-level content-addressable dedup (pnpm approach): hash each file individually for maximum cross-version dedup

