---
change: sdd-codegen-testgen
group: testgen-requirementplus
date: 2026-03-19
written_by: artifact_cli
review_verdict: REVISED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-sdd/generate/code-generator-contract.md | requirement-test-mapping | high | Requirement Plus → Test: Detailed Mapping section. N Requirements → N Test Classes, N Scenarios → N Test Functions via -verifies-> relationship, M Modules → import statements via -satisfies-> relationship, derives → test execution ordering, risk: High → critical test / CI fast path, verifymethod: Test → automated test function vs Inspection → TODO comment. |
| cclab-sdd/generate/requirement-plus-enhancement.md | requirement-plus-schema | high | R2: Risk levels (Low/Medium/High) and verification methods (Analysis/Inspection/Test/Demonstration). R2: verifymethod: Test triggers scaffold generation vs other methods. R3: Relationship types Satisfies/Verifies/Refines/Traces. R3: Verifies relationship is CI coverage gate target (Q3: check every Requirement has at least one Verifies). R3: Satisfies relationship maps to module under test import. R4: Schema validation for all new types. |
| cclab-sdd/generate/spec-model.md | test-output-structure | high | Requirement Plus layer mapped to test files and test functions. Full YAML test output example with given/when/then structure. System archetypes showing Requirement+ → test files, test functions mappings. Test generation integration patterns. |
| cclab-sdd/interfaces/cli/commands.md | cli-registration | high | Q1 subcommand: 'cclab sdd gen test <spec-path>'. CLI command registration via cclab-sdd-cli crate and CliModule trait. Command tree structure and CLI → logic mapping. Standalone CLI invocation and internal use during implement phase. |
| cclab-sdd/generate/codegen-system.md | codegen-architecture | medium | R5: Test generation component in code generation system. TestGenerator component in data flow. Unified internal representation for all spec types including RequirementPlus. Spec validation before generation. Template-based generation engine. Pluggable generators architecture. |
| cclab-sdd/generate/mermaid-plus-format.md | requirement-plus-input | medium | RequirementPlus input format specification. YAML frontmatter schema inside Mermaid code block (Mermaid official format). Frontmatter includes risk levels, verification methods, relationship types. Parsing rules for requirements/elements fields. Structured YAML frontmatter + Mermaid diagram combination. |
| cclab-sdd/generate/test-generation.md | test-integration | medium | Test generation integration with cclab-probe framework. Sequence diagram for test generation flow. RequirementPlus to test scaffold generation pipeline. Q4 deferred: cclab-probe fixture integration and advanced fixture DI patterns. Background context for Q4 follow-up work on probe integration. |
| cclab-sdd/generate/architecture.md | generate-pipeline | medium | Generate diagram generator architecture and generation pipeline overview. Input types: Semantic input, code analysis, spec definition. Generate engine: Parser, AST transformer, output generator. Output formats: Mermaid, OpenAPI 3.1, AsyncAPI 2.6, OpenRPC 1.3, Serverless Workflow. RequirementPlus as supported input type. |
| cclab-sdd/sdd-cli.md | sdd-workflow | low | Q1: CLI can be called internally during implement phase. SDD CLI subcommands and state machine interoperability. CLI tool registration via CliModule trait. Background on how SDD phases and CLI routing work for test generation invocation. |
| cclab-probe/README.md | test-framework | low | cclab-probe test framework overview. Rust CLI with pyo3 bindings for Python test execution. Q4 deferred: full probe integration with fixture DI and advanced testing patterns. Background context for deferred cclab-probe integration work. Test discovery and execution architecture for generated test suites. |

