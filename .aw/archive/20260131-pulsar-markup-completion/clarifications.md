---
change: pulsar-markup-completion
date: 2026-01-31
---

# Clarifications

## Q1: XML Scope
- **Question**: What scope of XML namespace support do you need?
- **Answer**: Full namespace support: xmlns, prefixed namespaces, namespace-aware lookup, serialization
- **Rationale**: Full namespace support is required for proper XML processing, enabling compatibility with real-world XML documents that use namespaces extensively

## Q2: XSLT Priority
- **Question**: Which XSLT elements are priority?
- **Answer**: Core elements: apply-templates, choose/when/otherwise, copy, copy-of
- **Rationale**: These core elements cover the most common XSLT transformation patterns and complete the existing partial implementation

## Q3: Change Strategy
- **Question**: How should we handle the existing pulsar-markup change folder?
- **Answer**: Create new change (pulsar-markup-completion) as a separate change
- **Rationale**: Keeps the original pulsar-markup change intact for reference while tracking completion work separately

