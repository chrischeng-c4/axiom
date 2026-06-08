---
change: sdd-codegen-and-fixes
group: new-spec-types
date: 2026-03-20
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-sdd/logic/implement-task | workflow-logic | high | Codegen routing mechanism: ImplementSpecWithCodegen sub-state for codegen path, Integration with prism_generate_from_spec, Spec execution order via Kahn's topological sort, Per-spec CRR cycle for implementation validation, Support for new section types: deploy, wireframe, component, design-token |
| cclab-sdd/logic/change-spec | workflow-logic | high | Spec preparation and main_spec_ref assignment, Per-spec lifecycle: Prepared → Fill → Prune → Review, fill_sections from spec_plan.sections, Artifact writing enforcement via artifact CLI, SpecSubState enum for tracking progress, Section Type → Spec Lang Mapping must be extended with deploy type (lang: yaml, for k8s Deployment + Service manifests), section_rules keyword pattern for deploy trigger, fill_order priority for deploy sections |
| cclab-sdd/generate/codegen-system | codegen | high | Unified internal representation based on JSON Schema, Template-based generation engine (Tera/MiniJinja), Pluggable generators for different frameworks/targets, Pre-generation spec validation, Test generation support, Support for new generators: Deploy (k8s), React (wireframe), Tailwind (design tokens), TypeScript (CEM) |
| cclab-sdd/generate/code-generator-contract | codegen | high | Generator trait contract definition, Spec input → code output mapping table (core semantic mappings), Inference rules for automatic code pattern detection, Spec input → code output mapping for new section types:,   - Deploy (k8s yaml) → Kubernetes Deployment + Service manifests,   - Wireframe → React component scaffold,   - Design Tokens (DTCG) → CSS/Tailwind tokens,   - Component + REST-API cross-ref → TypeScript interface + data fetching hooks |
| cclab-sdd/generate/spec-validator | codegen | high | Existing: Type validation (R1), Reference validation (R2), Completeness checking (R3), New capabilities needed: Shared validator registration mechanism (Q5), New capabilities needed: Soft warning mode (not blocking) for deploy section cross-ref validation (Q2), Deploy-specific validation rules: reference validation to db-model and rest-api deferred to later iteration |
| cclab-sdd/generate/spec-ir-schema | codegen | medium | Kubernetes-style resource envelope, Kind registry for different spec types, Strict YAML serialization, Language-agnostic interface, Support for new kinds: Deploy, Wireframe, Component, DesignToken |
| cclab-sdd/generate/template-engine | codegen | medium | TemplateEngine::render() contract, Template directory loading (Tera integration), Filter registration for new case conversions (kebab-case for CSS custom properties), Rendering context support for new generator types (React, Tailwind, TypeScript) |
| cclab-sdd/logic/reference-context | artifact-tools | medium | spec_plan generation with spec entries, spec_id, relevance (high/medium/low) categorization, key_requirements extraction from specs, GroupSubState tracking, Artifact writing enforcement via artifact CLI, spec_group assignment for logical architecture grouping |
| cclab-sdd/tools/utils/write-artifact | artifact-tools | medium | Unified artifact writer with payload-based routing, Schema validation for artifact-specific payloads, STATE.yaml phase transition management, Support for multiple artifact types, JSON output for machine-readable results, Support for revise-reference-context and revise-change-spec operations |
| cclab-sdd/interfaces/cli/commands | interfaces | medium | Core workflow subcommands under cclab sdd, artifact create-reference-context and related commands, artifact revise-reference-context command routing, State machine interoperability with STATE.yaml, JSON file input support via --json-file, CliModule trait and distributed_slice registration |
| cclab-sdd/interfaces/tools/workflow-tools | interfaces | medium | OpenRPC method definitions for workflow operations, Parameter and result schemas, sdd_workflow_create_reference_context definition, sdd_workflow_create_change_spec definition, sdd_workflow_create_change_implementation definition |
| cclab-sdd/generate/README | codegen | low | Code generation and template library organization, Subsystem architecture and module organization, Integration points with change_spec and change_impl, Spec IR system design overview |
| cclab-sdd/README | interfaces | low | Spec format priority and authoring rules, Phase flow from Init through Merge, Separation between interfaces and logic layers, Spec naming and organization conventions |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| change-spec-logic | modify | cclab-sdd/logic/change-spec | — |
| codegen-system-extend | modify | cclab-sdd/generate/codegen-system | — |
| spec-validator-extend | modify | cclab-sdd/generate/spec-validator | — |
| spec-ir-schema-extend | modify | cclab-sdd/generate/spec-ir-schema | — |
| generator-deploy | create | cclab-sdd/generate/generator-deploy | — |
| generator-react | create | cclab-sdd/generate/generator-react | — |

