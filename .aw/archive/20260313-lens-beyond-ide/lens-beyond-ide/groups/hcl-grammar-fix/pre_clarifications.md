---
change: lens-beyond-ide
group: hcl-grammar-fix
date: 2026-03-13
status: answered
---

# Pre-Clarifications

### Q1: Approach
- **Answer**: Upgrade tree-sitter to 0.25+ to support grammar v15. All other grammar crates (python, typescript, rust, javascript, html, css, yaml) must also be upgraded to matching versions. This is a broader change but resolves the incompatibility at the root.

