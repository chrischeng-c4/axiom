---
change: pulsar-markup
date: 2026-01-30
---

# Clarifications

## Q1: Feature Scope
- **Question**: What markup features should pulsar-markup include?
- **Answer**: HTML + XML + XSLT - full markup processing capabilities
- **Rationale**: Comprehensive lxml replacement requires all three: HTML for web scraping, XML for data interchange, XSLT for transformations.

## Q2: Parsing Mode
- **Question**: What parsing mode should be supported?
- **Answer**: Lenient parsing (handle malformed HTML like browsers)
- **Rationale**: Real-world HTML is often malformed. Lenient parsing is essential for web scraping and processing user-generated content.

