---
number: 924
title: "feat(agent): Add ReviewAgent — CRR review for specs and code"
state: open
labels: [enhancement, crate:agent, P1]
group: "agent-crr-core"
---

# #924 — feat(agent): Add ReviewAgent — CRR review for specs and code

**Parent**: #920

## Summary

Reusable review agent for CRR cycles. Reviews specs against format/quality standards, reviews code against specs.

## Review Types

### Spec Review
- Format compliance (OpenRPC/JSON Schema/Mermaid preferred over prose)
- Diagram correctness (right diagram type for structure)
- Quality (< 10% prose, no real code)
- Completeness (all required sections present)
- Consistency (naming matches across specs)

### Code Review
- Spec compliance (does code implement what spec defines?)
- Security (OWASP top 10)
- Test coverage
- Style consistency

## Output

```rust
pub enum ReviewVerdict {
    Approved,
    NeedsRevision { issues: Vec<ReviewIssue> },
    Rejected { reason: String },
}

pub struct ReviewIssue {
    pub severity: Severity,  // High, Medium, Low
    pub description: String,
    pub suggestion: String,
    pub location: Option<String>,  // file:line or spec section
}
```

## Dependencies
- SpecAgent (used as reviewer in CRR)

## Test Plan
- [ ] Unit: spec review → approved
- [ ] Unit: spec review → needs_revision with issues
- [ ] Unit: code review → issues with severity
