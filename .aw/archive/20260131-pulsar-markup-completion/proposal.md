---
id: pulsar-markup-completion
type: proposal
version: 1
created_at: 2026-01-31T02:48:24.535980+00:00
updated_at: 2026-01-31T02:48:24.535980+00:00
author: mcp
status: proposed
iteration: 1
summary: "Complete markup module with XML namespace support and core XSLT instructions."
history:
  - timestamp: 2026-01-31T02:48:24.535980+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T02:51:55.887056+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T02:52:09.532870+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
  - timestamp: 2026-01-31T02:54:50.086886+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T02:55:07.909957+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 5
  new_files: 0
affected_specs:
  - id: pulsar-markup-xml-ns
    path: specs/pulsar-markup-xml-ns.md
    depends: []
  - id: pulsar-markup-xslt-core
    path: specs/pulsar-markup-xslt-core.md
    depends: []---

<proposal>

# Change: pulsar-markup-completion

## Summary

Complete markup module with XML namespace support and core XSLT instructions.

## Why

The current markup module lacks essential XML and XSLT features. XML namespaces are not correctly handled, and core XSLT instructions are missing, limiting the module's utility for complex document processing and data integration tasks. Standard compliance and robustness are needed for the Pulsar ecosystem.

## What Changes

- Implement namespace-aware XML parsing and serialization.
- Add namespace-aware lookup methods to Document.
- Implement xsl:apply-templates, xsl:choose, xsl:copy, and xsl:copy-of in XSLT transformer.
- Add comprehensive tests for namespaces and new XSLT instructions.

## Impact

- **Scope**: minor
- **Affected Files**: ~5
- **New Files**: ~0
- Affected specs:
  - `pulsar-markup-xml-ns` (no dependencies)
  - `pulsar-markup-xslt-core` (no dependencies)
- Affected code: `crates/cclab-pulsar/src/markup/xml/mod.rs`, `crates/cclab-pulsar/src/markup/dom/document.rs`, `crates/cclab-pulsar/src/markup/dom/node.rs`, `crates/cclab-pulsar/src/markup/xslt/mod.rs`, `crates/cclab-pulsar/src/markup/error.rs`

</proposal>

<review iteration="1" reviewer="mcp" status="approved">
## Summary
The proposal correctly addresses the user's request to complete the markup module with XML namespace support and core XSLT instructions. It identifies the relevant components and outlines the necessary changes.

## Verdict
Approved

## Next Steps
Proceed to technical design (specs).
</review>
