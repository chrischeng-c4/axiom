---
change: jet-remaining
group: production-build-pipeline
date: 2026-03-19
status: clarified
---

# Post-Clarifications

## Questions

### Q1: Implementation Priorities
- **Question**: Should we strictly follow the custom minifier path in variable-mangling.md, or move to OXC as suggested in #765? Also, confirm if aot-build.md (796-spec) is the only synthetic test we need.
- **Answer**: Integrate oxc_minifier. Use [name].[hash].[ext] as default. Validation includes both mini-react (796-spec) and real-world apps (#797).
- **Rationale**: OXC provides better performance and maintenance. Multiple validation targets ensure broad compatibility.

## Contradictions

### C1: jet-variable-mangling-spec vs requirement
- **Spec**: jet-variable-mangling-spec
- **Requirement**: Minifier Implementation Strategy
- **Conflict**: #765 suggests Option C (OXC minifier) is preferred, but variable-mangling.md (jet-variable-mangling-spec) describes a custom AST-based minifier implementation.
- **Resolution**: We will integrate oxc_minifier for production builds as per the preference in #765 and the pre-clarification, while ensuring it integrates with the existing jet-specific tree-shaking and scope-hoisting passes.

### C2: 796-spec vs requirement
- **Spec**: 796-spec
- **Requirement**: AOT Validation Scope
- **Conflict**: aot-build.md (796-spec) focuses exclusively on expanding the mini-react example, while #797 and #765 define a much broader validation and feature set for the production build pipeline.
- **Resolution**: aot-build.md will be treated as the 'synthetic benchmark' part of the validation, while #797 (Real-world apps) will be the primary validation target for the features in #765.

