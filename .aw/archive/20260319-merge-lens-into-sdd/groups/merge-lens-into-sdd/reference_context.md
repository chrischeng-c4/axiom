---
change: merge-lens-into-sdd
group: merge-lens-into-sdd
date: 2026-03-19
written_by: mainthread
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-lens/README.md | architecture | high | module structure, feature inventory |
| cclab-lens/class-diagram.md | architecture | high | class hierarchy, module boundaries |
| cclab-lens/code-analysis-service-v2.md | core-service | high | analysis pipeline, fast-path execution |
| cclab-lens/lens-cli-subcommands.md | cli | high | ensure_daemon_ready, 8 analysis subcommands, CliModule registration |
| cclab-lens/lens-comprehensive.md | features | high | feature completeness inventory |
| cclab-lens/lens-codegen-unification.md | integration | high | CodeGenerator trait, GeneratorRegistry, SpecIR import path |
| cclab-lens/lens-beyond-ide.md | rpc-mcp | medium | JSON schema validation, tree-sitter upgrade, language expansion (MCP excluded per Q2) |
| cclab-lens/refactoring-api.md | core-api | medium | refactoring operations |
| cclab-lens/semantic-search-api.md | core-api | medium | search modes |
| cclab-lens/lens-pdg-mcp-tools.md | analysis | medium | PDG analysis tools |
| cclab-lens/python-pdg-core.md | analysis | medium | PDG construction |
| cclab-lens/lens-lang-support.md | language-support | medium | language analyzers |
| cclab-lens/lens-markdown.md | language-support | medium | markdown analysis |
| cclab-lens/lens-index-storage.md | storage | medium | index storage paths |
| cclab-lens/lens-yaml-codegen.md | integration | medium | SpecIR manifest consumption |
| cclab-lens/rust-symbol-analysis.md | symbol-extraction | medium | Rust symbol extraction |
| cclab-lens/usage-examples.md | documentation | low | usage documentation |
| cclab-lens/analysis-tools.md | utilities | low | — |
| cclab-sdd/README.md | target-context | low | — |
| cclab-sdd/sdd-cli.md | target-context | low | CliModule registration pattern |
