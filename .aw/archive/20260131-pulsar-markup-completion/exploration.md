---
id: pulsar-markup-completion
type: exploration
created_at: 2026-01-31T02:51:19.885085+00:00
needs_clarification: false
---

# Codebase Exploration

The markup module is located in crates/cclab-pulsar/src/markup.
Relevant files for implementation:
- crates/cclab-pulsar/src/markup/xml/mod.rs: Currently a wrapper around HTML parser. Needs a proper XML parser with namespace tracking.
- crates/cclab-pulsar/src/markup/dom/node.rs: Node struct already has namespace and prefix fields, but needs methods for convenient creation and management.
- crates/cclab-pulsar/src/markup/dom/document.rs: Needs namespace-aware lookup (find_by_tag_ns) and serialization.
- crates/cclar-pulsar/src/markup/xslt/mod.rs: Needs implementation of apply-templates, choose, when, otherwise, copy, and copy-of.

Technical Considerations:
- Use a scope stack during XML parsing to resolve prefixes to URIs.
- Implement XSLT priority rules for template matching in apply-templates.
- Ensure serialization correctly handles default namespaces vs prefixed ones.
- Error handling in crates/cclab-pulsar/src/markup/error.rs should be updated for XML/XSLT specific errors if needed.
