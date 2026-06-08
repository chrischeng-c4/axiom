---
change: sdd-codegen-completion
group: deploy-section-type
date: 2026-03-20
written_by: artifact_cli
review_verdict: approved
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| generator-deploy | deploy-codegen | high | R1-R5: DeployGenerator implements SpecIRGenerator trait. can_generate() returns true only for SpecIR::Deploy. generate_from_ir() produces exactly two files: deployment.yaml (apps/v1 Deployment) and service.yaml (v1 ClusterIP Service). Deployment includes: metadata.name from spec, spec.replicas, container image and port from DeploySpec, env vars (literal and valueFrom), optional resource limits (CPU/memory). Service includes: type ClusterIP, selector.app matching deployment, port mapping. Template fallback and overwrite policy enforcement. |
| codegen-system | deploy-codegen | high | R1-R7: Code generation system architecture with unified internal representation. R6-R7 specifically define SpecIRGenerator trait for section-type generators (Deploy, React, Component, DesignToken variants) and DeployGenerator as R7 example. Routing flow: SpecIR → SpecValidator → TemplateEngine → SpecIRGenerators (Deploy/React). Critical gap: SectionType enum needs Deploy variant added to router and section_rules registration with keywords (deploy|container|k8s|docker|helm|terraform|infra|migration|rollback). |
| spec-ir-contract | spec-ir-system | high | R1-R5: SpecIR enum type that represents all spec types including Deploy variant. SpecMetadata struct with source file, spec group, spec ID, tags. From<T> implementations for constructing SpecIR from domain types. SpecBundle for multi-spec composition. Serializability for MCP transport. Deploy variant introduced here and propagated through codegen system. |
| template-engine | template-system | high | R1-R4: Tera template engine integration for code generation. TemplateEngine class with render() method. Template context building. Rendering flow: parse template → populate context → generate output. Embedded templates via include_str! macro. Used by DeployGenerator R4 for template fallback when deploy/deployment.yaml.j2 and deploy/service.yaml.j2 are present. Template loading, variable interpolation, filter functions. |
| spec-ir-schema | spec-ir-system | high | R1-R3: SpecIR YAML manifest schema (Kubernetes-style: apiVersion, kind, metadata, spec). Standard envelope for all SpecIR types. Deploy kind serialization format. JSON Schema envelope with strict validation. Language-agnostic interface between SDD producer and Lens consumer. Roundtrip serialization/deserialization support for Deploy specs. |
| code-generator-contract | generator-contract | medium | Contract between specs (input) and generators (output). Generator responsibilities: spec signal → code output mapping (e.g. DeploySpec.replicas → Deployment replicas field, DeploySpec.env → container env block). Inference rules for auto-injecting dependencies from spec semantics. Pluggable generator trait implementation pattern. |
| spec-validator | validation-system | medium | R1-R2: Type validation (required fields, type checking) and reference validation (cross-spec links). Context for k8s validation rules: namespace, metadata.name format, image reference validation, port range validation. Soft warning mode for cross-ref validation per pre-clarification Q3 (deferred, best-effort output acceptable). |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: sdd-codegen-completion

**Verdict**: approved

### Summary

Revised reference context now contains 7 actual specs with high-medium relevance for deploy-section-type. All review feedback addressed:

1. ✅ **Placeholder rows filled**: All rows now contain real spec references with complete key requirements
2. ✅ **High-relevance specs included**: 5 specs (generator-deploy, codegen-system, spec-ir-contract, template-engine, spec-ir-schema) directly implement Deploy section type and SpecIR infrastructure
3. ✅ **Medium-relevance specs included**: 2 specs (code-generator-contract, spec-validator) provide supporting context and cross-cutting patterns
4. ✅ **SectionType enum gap surfaced**: codegen-system key_requirements explicitly mentions "Critical gap: SectionType enum needs Deploy variant added to router and section_rules registration"
5. ✅ **Spec scope validation**: All specs from cclab/specs/cclab-sdd/generate/ directory are documented and verified to exist

### Issues Resolved

All issues from prior review are now closed:
- Reference context is no longer placeholder — 7 real specs with full relevance statements and key requirements
- DeployGenerator, SpecIR, codegen system, template engine, and schema specs directly map to this group's scope
- SectionType enum gap is explicitly called out in codegen-system requirements for immediate action during implementation
- Pre-clarifications scope (k8s Deployment+Service, DSL finalization, soft validation) is fully covered by selected specs
- All specs verified to exist at cclab/specs/cclab-sdd/generate/ paths

