---
id: lens-markdown-spec
main_spec_ref: cclab-lens/lens-markdown.md
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "This analysis/standardization logic TD supports brownfield semantic coverage and takeover readiness gates."
---

# Lens Markdown Spec

## Overview
<!-- type: doc lang: markdown -->


Add Markdown/MDX language support and API spec format detection to cclab-lens, covering two major areas:

**1. Markdown/MDX Language Support**
- tree-sitter-mdx grammar for MDX (JSX-aware AST), tree-sitter-md for plain Markdown
- Lint rules: broken links (internal + external HTTP HEAD), heading structure (skip levels, duplicates), code block language tags, frontmatter schema validation (Hugo/Jekyll/Docusaurus), line length
- Symbol extraction: headings (h1-h6 hierarchy), links (internal/external), code blocks (with language), frontmatter fields, MDX components
- Language enum extension: `.md`, `.mdx`, `.markdown` → Language::Markdown / Language::Mdx

**2. API Spec Format Detection & Linting**
- OpenAPI 3.x standalone file detection (YAML/JSON content inspection, not just extension)
- AsyncAPI 2.x/3.x standalone file detection and structure validation
- OpenRPC 1.x standalone file detection and method/param validation
- Mermaid (.mmd, .mermaid) via tree-sitter-mermaid grammar — diagram type detection and syntax validation
- Embedded markdown extraction: lint description/summary fields in OpenAPI/AsyncAPI/OpenRPC specs (single level, no recursion)
- Integration with existing YAML dispatcher (yaml_dispatch.rs)

### Key Decisions
- MDX: tree-sitter-mdx grammar (full JSX AST)
- Markdown: tree-sitter-md grammar
- Mermaid: tree-sitter-mermaid grammar (full AST)
- OpenAPI: 3.x only (no Swagger 2.0)
- Frontmatter: schema-based validation (Hugo/Jekyll/Docusaurus schemas)
- Broken links: internal file check + external HTTP HEAD
- Embedded markdown: single level only (description fields, no recursive code block parsing)
## Requirements
<!-- type: doc lang: markdown -->


### R1 - Language Enum Extension

```yaml
id: R1
priority: high
status: draft
```

Add `Markdown`, `Mdx`, and `Mermaid` variants to the `Language` enum.
File extension mapping:
- `.md`, `.markdown` → Markdown
- `.mdx` → Mdx
- `.mmd`, `.mermaid` → Mermaid

Register tree-sitter grammars in MultiParser:
- `tree-sitter-md` for Markdown
- `tree-sitter-mdx` for MDX (if available as crate, else tree-sitter-md + JSX regex detection)
- `tree-sitter-mermaid` for Mermaid

### R2 - Markdown Lint Checker

```yaml
id: R2
priority: high
status: draft
```

MarkdownChecker implementing the Checker trait with rules:
- MD001: Heading level skip (h1 → h3 without h2)
- MD002: Duplicate heading text in same file
- MD003: Missing code block language tag
- MD004: Line length exceeds limit (default 120)
- MD005: Broken internal link (relative path doesn't exist)
- MD006: Broken external link (HTTP HEAD returns 4xx/5xx)
- MD007: Invalid frontmatter YAML syntax
- MD008: Frontmatter schema violation (Hugo/Jekyll/Docusaurus)
- MD009: Trailing whitespace
- MD010: Multiple consecutive blank lines

### R3 - MDX Lint Rules

```yaml
id: R3
priority: medium
status: draft
```

MDX-specific rules extending MarkdownChecker:
- MDX001: Unclosed JSX component
- MDX002: Import without usage
- MDX003: Invalid export (non-component export in MDX)

### R4 - Markdown Symbol Builder

```yaml
id: R4
priority: high
status: draft
```

SymbolTableBuilder for Markdown/MDX extracting:
- Headings: h1-h6 with hierarchy (parent-child), line numbers, text
- Links: internal (relative path) and external (URL), anchor text
- Code blocks: language tag, line range
- Frontmatter fields: key-value pairs from YAML frontmatter
- MDX components: imported component names, usage locations

### R5 - OpenAPI Standalone Detection

```yaml
id: R5
priority: high
status: draft
```

Extend YAML dispatcher to detect OpenAPI 3.x files:
- Detection: top-level `openapi: "3.x.x"` key in YAML/JSON
- Validation: required fields (info, paths), path item structure, $ref format hints
- Embedded markdown: extract `description` and `summary` fields, lint with MD001-MD004 subset
- New checker: OpenApiChecker with rules:
  - OA001: Missing required field (info.title, info.version, paths)
  - OA002: Invalid $ref format
  - OA003: Empty path item
  - OA004: Missing operation ID
  - OA005: Description contains invalid markdown

### R6 - AsyncAPI Standalone Detection

```yaml
id: R6
priority: high
status: draft
```

Extend YAML dispatcher to detect AsyncAPI files:
- Detection: top-level `asyncapi: "2.x.x"` or `asyncapi: "3.x.x"` key
- Validation: required fields (info, channels/servers), message structure
- Embedded markdown: extract description fields, lint markdown content
- New checker: AsyncApiChecker with rules:
  - AA001: Missing required field (info, channels)
  - AA002: Invalid channel name pattern
  - AA003: Message without schema
  - AA004: Missing server protocol
  - AA005: Description contains invalid markdown

### R7 - OpenRPC Standalone Detection

```yaml
id: R7
priority: medium
status: draft
```

Detect OpenRPC files in JSON:
- Detection: top-level `openrpc: "1.x.x"` key in JSON
- Validation: methods array, params/result schema structure
- Embedded markdown: extract description fields
- New checker: OpenRpcChecker with rules:
  - OR001: Missing required field (openrpc, info, methods)
  - OR002: Method without params schema
  - OR003: Method without result schema
  - OR004: Duplicate method name
  - OR005: Description contains invalid markdown

### R8 - Mermaid Standalone Support

```yaml
id: R8
priority: medium
status: draft
```

Mermaid file support via tree-sitter-mermaid:
- Parse `.mmd`, `.mermaid` files with tree-sitter grammar
- Symbol extraction: diagram type, node IDs, edge labels
- Lint rules:
  - MM001: Unknown diagram type
  - MM002: Undefined node reference
  - MM003: Duplicate node ID
  - MM004: Empty diagram (no nodes)
  - MM005: Syntax error (tree-sitter ERROR nodes)

### R9 - Frontmatter Schema Registry

```yaml
id: R9
priority: medium
status: draft
```

Schema-based frontmatter validation:
- Detect framework from project config (hugo.toml, _config.yml, docusaurus.config.js)
- Programmatic JSON schemas for common frontmatter fields:
  - Hugo: title, date, draft, tags, categories, weight
  - Jekyll: layout, title, date, permalink, categories, tags
  - Docusaurus: id, title, sidebar_label, sidebar_position, slug
- Fallback: generic schema (title, date, description, tags)
- Integration with SchemaRegistry pattern from lens-beyond-ide

### R10 - YAML Dispatcher Extension

```yaml
id: R10
priority: high
status: draft
```

Extend existing `yaml_dispatch.rs` to route:
- `openapi: 3.x` → OpenApiChecker
- `asyncapi: 2.x/3.x` → AsyncApiChecker
- JSON files with `openrpc: 1.x` → OpenRpcChecker (via new JSON dispatcher)
- Preserve existing routes: K8s → KubernetesChecker, GitLab CI → GitlabCiChecker
## Scenarios
<!-- type: doc lang: markdown -->


### S1 - Markdown File Linting

```
Given a .md file with heading skip (h1 → h3)
When lens check is invoked
Then MD001 diagnostic is emitted with severity Warning
And heading hierarchy is reported in symbols
```

### S2 - MDX Component Detection

```
Given a .mdx file importing <MyComponent>
When lens symbols is invoked
Then imported component appears in symbol table
And JSX usage locations are tracked
```

### S3 - Broken Internal Link

```
Given a markdown file with [link](./nonexistent.md)
When lens check is invoked
Then MD005 diagnostic is emitted with the broken path
```

### S4 - Broken External Link

```
Given a markdown file with [link](https://example.com/404)
When lens check is invoked
Then MD006 diagnostic is emitted with HTTP status
```

### S5 - Frontmatter Schema Validation

```
Given a Hugo project with markdown containing frontmatter missing required 'title'
When lens check is invoked
Then MD008 diagnostic is emitted referencing Hugo schema
```

### S6 - OpenAPI Detection via YAML Content

```
Given a .yaml file containing 'openapi: "3.1.0"' at top level
When the YAML dispatcher processes the file
Then it routes to OpenApiChecker (not KubernetesChecker or GitlabCiChecker)
And OA001-OA005 rules are applied
```

### S7 - AsyncAPI Detection

```
Given a .yaml file containing 'asyncapi: "2.6.0"'
When lens check is invoked
Then AsyncApiChecker validates channels and messages
And embedded description fields are lint-checked for markdown
```

### S8 - OpenRPC JSON Detection

```
Given a .json file containing 'openrpc: "1.3.2"'
When lens check is invoked
Then OpenRpcChecker validates methods and params
```

### S9 - Mermaid File Parsing

```
Given a .mmd file with a flowchart diagram
When lens check is invoked
Then tree-sitter-mermaid parses the AST
And MM001-MM005 rules are applied
And diagram type + node IDs appear in symbols
```

### S10 - Embedded Markdown in OpenAPI

```
Given an OpenAPI spec with description: "# Overview\n\nSome **bold** text"
When lens check is invoked
Then the description content is extracted and lint-checked
And MD001 (heading skip) is applied to embedded content
```
## Diagrams
<!-- type: doc lang: markdown -->

### Sequence Diagram
<!-- TODO -->

### Flowchart
<!-- TODO -->

### Class Diagram
<!-- TODO -->

### State Diagram
<!-- TODO -->

### ERD
<!-- TODO -->

## API Spec
<!-- type: doc lang: markdown -->

### OpenAPI 3.1
<!-- TODO -->

### OpenRPC 1.3
<!-- TODO -->

### AsyncAPI 2.6
<!-- TODO -->

### Serverless Workflow 0.8
<!-- TODO -->

## Test Plan
<!-- type: doc lang: markdown -->

<!-- TODO -->

## Changes
<!-- type: doc lang: markdown -->


### New Files

| File | Est. Lines | Description |
|------|-----------|-------------|
| `src/lint/markdown.rs` | ~300 | MarkdownChecker with MD001-MD010 rules |
| `src/lint/mdx.rs` | ~120 | MDX-specific rules MDX001-MDX003 |
| `src/lint/openapi.rs` | ~250 | OpenApiChecker with OA001-OA005 |
| `src/lint/asyncapi.rs` | ~200 | AsyncApiChecker with AA001-AA005 |
| `src/lint/openrpc.rs` | ~200 | OpenRpcChecker with OR001-OR005 |
| `src/lint/mermaid.rs` | ~180 | MermaidChecker with MM001-MM005 |
| `src/semantic/symbols/markdown.rs` | ~200 | Markdown/MDX symbol builder |
| `src/semantic/symbols/mermaid.rs` | ~150 | Mermaid symbol builder (diagram type, nodes) |
| `src/schemas/frontmatter.rs` | ~200 | Frontmatter schema definitions (Hugo/Jekyll/Docusaurus) |
| `src/lint/embedded_markdown.rs` | ~150 | Embedded markdown extractor for API spec descriptions |

### Modified Files

| File | Changes |
|------|---------|
| `Cargo.toml` | +tree-sitter-md, +tree-sitter-mdx (or feature flag), +tree-sitter-mermaid |
| `src/syntax/parser.rs` | +Language::Markdown, Mdx, Mermaid variants; grammar init; extension mapping |
| `src/lint/mod.rs` | +mod markdown, mdx, openapi, asyncapi, openrpc, mermaid, embedded_markdown; register checkers |
| `src/lint/yaml_dispatch.rs` | +OpenAPI 3.x and AsyncAPI routing |
| `src/semantic/symbols/mod.rs` | +mod markdown, mermaid; register builders |
| `src/schemas/mod.rs` | +mod frontmatter; wire to SchemaRegistry |
| `src/types/incremental.rs` | +Language::Markdown, Mdx, Mermaid match arms |
| `src/lsp/server.rs` | +Language::Markdown, Mdx, Mermaid dispatch for symbols |
| `src/lib.rs` | Ensure new modules exported |
