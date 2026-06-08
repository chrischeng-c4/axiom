---
id: sdd-fillback-ast
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AST Analyzer Types

## Overview
<!-- type: overview lang: markdown -->

AST analysis types in `projects/agentic-workflow/src/fillback/ast.rs`. Nine shapes:

- `SupportedLanguage` — 5-variant enum, lowercase, derives include Hash.
- `SymbolKind` — 8-variant enum, lowercase, default Function.
- `StructField` — name/rust_type/is_public.
- `Symbol` — name/kind/optional signature/optional doc/line/is_public/fields/variants/optional logic.
- `Import` — path/items/is_external.
- `ModuleInfo` — name/path/language/symbols/imports.
- `ParseError` — path/reason.
- `AnalysisContext` — modules/skipped_files/language_counts.
- `AstAnalyzer` — parsers HashMap.

Codegen replaces all nine type declarations and the serde import they need.
Companion source specs own the non-serde imports and AST runtime/test
implementation.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SupportedLanguage:
    type: string
    enum: [Rust, Python, JavaScript, TypeScript, Go]
    description: Supported programming languages for AST analysis.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize]
      serde_rename_all: lowercase

  SymbolKind:
    type: string
    enum: [Function, Struct, Enum, Interface, Class, Module, Constant, Type]
    description: Kind of symbol extracted from source code.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Function, is_default: true, doc: "Function symbol (default)." }
        - { name: Struct, doc: "Struct symbol." }
        - { name: Enum, doc: "Enum symbol." }
        - { name: Interface, doc: "Interface symbol." }
        - { name: Class, doc: "Class symbol." }
        - { name: Module, doc: "Module symbol." }
        - { name: Constant, doc: "Constant symbol." }
        - { name: Type, doc: "Type alias symbol." }

  StructField:
    type: object
    required: [name, rust_type, is_public]
    description: A field on a struct symbol.
    properties:
      name:
        type: string
        description: "Field name."
      rust_type:
        type: string
        description: "Rendered type text (verbatim source slice)."
      is_public:
        type: boolean
        description: "Whether the field is pub."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  Symbol:
    type: object
    required: [name, kind, signature, doc, line, is_public, fields, variants, logic]
    description: A symbol extracted from source code.
    properties:
      name:
        type: string
        description: "Symbol name."
      kind:
        type: string
        x-rust-type: "SymbolKind"
        description: "Symbol kind."
      signature:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional signature text."
      doc:
        type: string
        x-rust-type: "Option<String>"
        description: "Optional doc comment."
      line:
        type: integer
        x-rust-type: "usize"
        description: "Line number."
      is_public:
        type: boolean
        description: "Whether the symbol is pub."
      fields:
        type: array
        items: { type: object }
        x-rust-type: "Vec<StructField>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Populated for SymbolKind::Struct."
      variants:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Populated for SymbolKind::Enum."
      logic:
        type: object
        x-rust-type: "Option<LogicContent>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Top-level control flow for functions."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  Import:
    type: object
    required: [path, items, is_external]
    description: An import or dependency relationship.
    properties:
      path:
        type: string
        description: "Import path."
      items:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Imported items."
      is_external:
        type: boolean
        description: "External dep vs internal."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ModuleInfo:
    type: object
    required: [name, path, language, symbols, imports]
    description: Parsed module information.
    properties:
      name:
        type: string
        description: "Module name."
      path:
        type: string
        description: "Module path."
      language:
        type: string
        x-rust-type: "SupportedLanguage"
        description: "Module language."
      symbols:
        type: array
        items: { type: object }
        x-rust-type: "Vec<Symbol>"
        description: "Symbols in this module."
      imports:
        type: array
        items: { type: object }
        x-rust-type: "Vec<Import>"
        description: "Imports."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ParseError:
    type: object
    required: [path, reason]
    description: Parse error information.
    properties:
      path:
        type: string
        description: "File path."
      reason:
        type: string
        description: "Failure reason."
    x-rust-struct:
      derive: [Debug, Clone]

  AnalysisContext:
    type: object
    required: [modules, skipped_files, language_counts]
    description: Result of analyzing a codebase.
    properties:
      modules:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ModuleInfo>"
        description: "Parsed modules."
      skipped_files:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Files that were skipped."
      language_counts:
        type: object
        x-rust-type: "HashMap<String, usize>"
        description: "Language → file-count map."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AstAnalyzer:
    type: object
    required: [parsers]
    description: AST Analyzer using tree-sitter.
    properties:
      parsers:
        type: object
        x-rust-type: "HashMap<SupportedLanguage, Parser>"
        x-rust-visibility: private
        description: "Per-language parsers."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/fillback/ast.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - SupportedLanguage
      - SymbolKind
      - StructField
      - Symbol
      - Import
      - ModuleInfo
      - ParseError
      - AnalysisContext
      - AstAnalyzer
    description: |
      Codegen replaces all nine type declarations and the generated serde import.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] 9 types: 2 enums + 7 structs.
- [schema] All in `required:`; foreign types via x-rust-type; default variant via is_default.
- [changes] All nine in `replaces`.
