---
change: jet-dev-server-v2
group: jet-dev-server-v2
date: 2026-03-25
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: For #1092 .jet-store fix: should we implement pnpm-style nested node_modules in store entries, or install packages with optionalDependencies directly into project node_modules?
- **Answer**: pnpm-style nested node_modules in store entries. Create node_modules inside each .jet-store/{pkg}@{version}/ containing symlinks to that package's dependencies. Full Node.js resolution compatibility.

### Q2: General
- **Question**: For #1091 polyfills: should polyfills be served as separate files or bundled inline?
- **Answer**: Separate .jet/polyfill-{name}.mjs files via importmap. Each Node.js builtin polyfill is a standalone ESM file, importmap maps bare specifier (e.g. 'crypto') to .jet/polyfill-crypto.mjs.

