---
change: lens-beyond-ide
group: hcl-grammar-fix
date: 2026-03-13
---

# Requirements

Fix tree-sitter-hcl grammar version incompatibility. Current error: 'Incompatible language version 15. Expected minimum 13, maximum 14'.

1. Investigate options: upgrade tree-sitter to support grammar v15, or pin tree-sitter-hcl to a version that produces v14 grammar.
2. Fix must not break existing Terraform/HCL lint rules or symbol builder.
3. All 116 existing test failures caused by this issue should be resolved.
