---
id: pulsar-markup
type: proposal
version: 1
created_at: 2026-01-30T07:20:00.000000+00:00
updated_at: 2026-01-30T07:20:00.000000+00:00
author: claude
status: proposed
iteration: 1
summary: "Create cclab-pulsar-markup: Pure Rust markup processing library (lxml replacement)"
history:
  - timestamp: 2026-01-30T07:20:00.000000+00:00
    agent: "claude"
    tool: "manual"
    action: "created"
impact:
  scope: minor
  affected_files: 12
  new_files: 12
affected_specs:
  - id: pulsar-markup-html
    path: specs/pulsar-markup-html.md
    depends: []
  - id: pulsar-markup-xml
    path: specs/pulsar-markup-xml.md
    depends: []
  - id: pulsar-markup-xpath
    path: specs/pulsar-markup-xpath.md
    depends: [pulsar-markup-html, pulsar-markup-xml]
  - id: pulsar-markup-css
    path: specs/pulsar-markup-css.md
    depends: [pulsar-markup-html]
  - id: pulsar-markup-xslt
    path: specs/pulsar-markup-xslt.md
    depends: [pulsar-markup-xml, pulsar-markup-xpath]
---

<proposal>

# Change: pulsar-markup

## Summary

Create cclab-pulsar-markup: Pure Rust markup processing library replacing Python's lxml and BeautifulSoup.

## Why

Data science and web scraping workflows require robust HTML/XML parsing. Python's lxml is a common dependency but requires C libraries. A pure Rust implementation provides:
- No external C dependencies
- Better performance for large documents
- Memory safety guarantees
- Integration with pulsar ecosystem

## What Changes

- Create new crate `crates/cclab-pulsar-markup`
- Implement HTML parser with lenient mode (handles malformed HTML)
- Implement XML parser with namespace support
- Implement XPath query engine
- Implement CSS selector engine
- Implement DOM traversal and manipulation
- Implement basic XSLT transformations

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~12
- Affected specs:
  - `pulsar-markup-html` (no dependencies)
  - `pulsar-markup-xml` (no dependencies)
  - `pulsar-markup-xpath` → depends on: `html`, `xml`
  - `pulsar-markup-css` → depends on: `html`
  - `pulsar-markup-xslt` → depends on: `xml`, `xpath`
- **Breaking Changes**: None (new crate)

</proposal>
