---
change: sdd-codegen-completion
group: frontend-codegen
date: 2026-03-20
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-sdd/generate/codegen-system.md | code-generation | high | SpecIRGenerator trait definition and routing, Pluggable generators for multiple frameworks, Code generation system architecture, Support for wireframe/component/design-token section types |
| cclab-sdd/generate/generator-react.md | code-generation | high | WireframeSpec input schema (name, component_type, props, layout), JSX rendering for node kinds (text, button, input, list, semantic HTML), TypeScript props interface generation, Barrel export file generation, Template fallback mechanism |
| cclab-sdd/generate/spec-ir-schema.md | code-generation | high | SpecIR YAML manifest schema with Kubernetes-style envelope (apiVersion, kind, metadata, spec), Kind enum definition (Api, FlowchartPlus, SequencePlus, ClassPlus, ErdPlus, RequirementPlus), SpecMetadata contract, Strict serialization validation |
| cclab-sdd/generate/spec-ir-contract.md | code-generation | high | SpecIR enum type definition (Api, Wireframe, Component, DesignToken, Deploy variants), SpecMetadata structure for routing, From trait implementations for type conversion |
| cclab-sdd/generate/code-generator-contract.md | code-generation | high | Code generator contract and responsibilities, Spec input to framework-specific output mapping, Inference rules for semantic interpretation, Generator composition patterns |
| cclab-sdd/generate/template-engine.md | code-generation | high | Tera template engine integration, Template directory structure (react/, component/, design-token/), Fallback to inline string generation, Template discovery and routing |
| cclab-sdd/generate/spec-validator.md | code-generation | high | Spec validation framework for all section types, Validator logic and error reporting, Completeness checks for wireframe/component/design-token inputs |
| cclab-sdd/generate/architecture.md | code-generation | medium | Generate subsystem architecture, Generation pipeline and data flow, Code generation framework overview, Section type routing |
| cclab-sdd/README.md | sdd-framework | low | SDD phase flow context, Reference context phase placement in workflow |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| frontend-codegen-main | create | cclab-sdd/generate/frontend-codegen.md | overview, schema, logic, interaction, component, wireframe, design-token, template, test-plan |

