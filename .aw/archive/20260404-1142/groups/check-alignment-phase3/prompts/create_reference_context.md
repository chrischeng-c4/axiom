# Task: Gather Reference Context for Group 'check-alignment-phase3' (Change '1142')

Issues: #1142_feat-sdd-check-alignment-workflow-integration-phas

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write the artifact. The system verifies
artifacts were written via CLI — direct file writes will be REJECTED and you
will have to redo the work.

## Existing Spec Structure

The following ASCII tree shows existing spec directories for the affected crate(s). Use this to plan spec_plan entries — prefer modifying existing files over creating new ones.

```
cclab-sdd
├── README.md
├── config
│   ├── agents.md
│   └── platform.md
├── generate
│   ├── README.md
│   ├── architecture.md
│   ├── block-plus-spec.md
│   ├── code-generator-contract.md
│   ├── codegen-system
│   ├── codegen-system.md
│   ├── generator-axum.md
│   ├── generator-deploy
│   ├── generator-express.md
│   ├── generator-fastapi.md
│   ├── generator-react
│   ├── generator-react.md
│   ├── json-schema-core.md
│   ├── mermaid-plus-conversion.md
│   ├── mermaid-plus-format.md
│   ├── requirement-plus-enhancement.md
│   ├── spec-ir-contract.md
│   ├── spec-ir-evaluation.md
│   ├── spec-ir-schema
│   ├── spec-ir-schema.md
│   ├── spec-model.md
│   ├── spec-validator
│   ├── spec-validator.md
│   ├── template-claude-md.md
│   ├── template-engine.md
│   ├── template-knowledge-index.md
│   ├── test-generation.md
│   └── ux-pattern-library.md
├── interfaces
│   ├── cli
│   │   ├── commands.md
│   │   └── sdd-cli.md
│   ├── lens
│   │   ├── lens-cli-subcommands.md
│   │   └── lens-pdg-mcp-tools.md
│   └── tools
│       ├── artifact-tools.md
│       ├── utility-tools.md
│       └── workflow-tools.md
├── logic
│   ├── agent-context-builder.md
│   ├── agent-output-format.md
│   ├── analysis-tools.md
│   ├── cclab-lens-spec.md
│   ├── change-merge.md
│   ├── change-spec-logic.md
│   ├── change-spec.md
│   ├── check-alignment.md
│   ├── class-diagram.md
│   ├── code-analysis-service-v2.md
│   ├── executor-resolution.md
│   ├── implement-task.md
│   ├── lens-README.md
│   ├── lens-beyond-ide.md
│   ├── lens-codegen-unification.md
│   ├── lens-comprehensive.md
│   ├── lens-full-upgrade-spec.md
│   ├── lens-index-storage.md
│   ├── lens-lang-support.md
│   ├── lens-markdown.md
│   ├── lens-yaml-codegen.md
│   ├── merge-lens-into-sdd-spec.md
│   ├── post-clarifications.md
│   ├── pre-clarifications.md
│   ├── python-pdg-core.md
│   ├── refactoring-api.md
│   ├── reference-context.md
│   ├── restructure-input.md
│   ├── rust-symbol-analysis.md
│   ├── scope-resolution.md
│   ├── semantic-search-api.md
│   ├── spec-diff-codegen.md
│   ├── spec-structure.md
│   ├── state-machine.md
│   ├── tech-stack-inference.md
│   ├── type-inference-pipeline.md
│   └── usage-examples.md
├── skills
│   ├── agent.md
│   ├── fillback.md
│   ├── merge.md
│   ├── revise-artifact.md
│   └── run-change.md
└── tools
    ├── sdd-codegen-testgen-spec.md
    └── utils
        ├── analyze-code-for-spec.md
        ├── delegate-agent.md
        ├── fetch-issues.md
        ├── list-changed-files.md
        ├── platform-sync.md
        ├── read-artifact.md
        ├── read-implementation-summary.md
        ├── validate-change.md
        ├── validate-spec-completeness.md
        └── write-artifact.md

```

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chrischeng/projects/wt/sdd/cclab/changes/1142/groups/check-alignment-phase3/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, prompt, changes]
Use these as starting point for spec_plan.sections. Adjust based on your analysis.

## Output: spec_plan array

For each change spec that will be created:
- spec_id: identifier for the new change spec
- action: "modify" (copy existing) or "create" (new skeleton)
- main_spec_ref: target path in cclab/specs/ (REQUIRED — must include a named subfolder,
e.g. `crates/cclab-sdd/logic/foo.md`, not `crates/cclab-sdd/foo.md`)
- source: path of existing spec to copy (only for "modify")
- sections: array of section types this spec needs (see change-spec.md § Section Selection)

**Action preference**: Use `action: modify` for any file visible in the spec directory tree
above. Reserve `action: create` for genuinely new subsystems with no existing spec file.

## File Decomposition Rules

1. **One spec file = one logical unit** (service, module, component). Do NOT bundle unrelated concerns.
2. **No duplicate section types in one file** — if a feature needs two REST APIs (e.g., external + internal), split into two spec files, each with its own `rest-api` section.
3. **Spec path mirrors source path** — `src/api/external.rs` → `specs/interfaces/external-api.md`.
4. **Cross-file references** — related specs link via `refs` frontmatter and `$ref` in content.

## In-Scope Specs

### cclab-sdd
- `read_path:specs/crates/cclab-sdd/README.md`
- `read_path:specs/crates/cclab-sdd/config/agents.md`
- `read_path:specs/crates/cclab-sdd/config/platform.md`
- `read_path:specs/crates/cclab-sdd/generate/README.md`
- `read_path:specs/crates/cclab-sdd/generate/architecture.md`
- `read_path:specs/crates/cclab-sdd/generate/block-plus-spec.md`
- `read_path:specs/crates/cclab-sdd/generate/code-generator-contract.md`
- `read_path:specs/crates/cclab-sdd/generate/codegen-system.md`
- `read_path:specs/crates/cclab-sdd/generate/generator-axum.md`
- `read_path:specs/crates/cclab-sdd/generate/generator-express.md`
- `read_path:specs/crates/cclab-sdd/generate/generator-fastapi.md`
- `read_path:specs/crates/cclab-sdd/generate/generator-react.md`
- `read_path:specs/crates/cclab-sdd/generate/json-schema-core.md`
- `read_path:specs/crates/cclab-sdd/generate/mermaid-plus-conversion.md`
- `read_path:specs/crates/cclab-sdd/generate/mermaid-plus-format.md`
- `read_path:specs/crates/cclab-sdd/generate/requirement-plus-enhancement.md`
- `read_path:specs/crates/cclab-sdd/generate/spec-ir-contract.md`
- `read_path:specs/crates/cclab-sdd/generate/spec-ir-evaluation.md`
- `read_path:specs/crates/cclab-sdd/generate/spec-ir-schema.md`
- `read_path:specs/crates/cclab-sdd/generate/spec-model.md`
- `read_path:specs/crates/cclab-sdd/generate/spec-validator.md`
- `read_path:specs/crates/cclab-sdd/generate/template-claude-md.md`
- `read_path:specs/crates/cclab-sdd/generate/template-engine.md`
- `read_path:specs/crates/cclab-sdd/generate/template-knowledge-index.md`
- `read_path:specs/crates/cclab-sdd/generate/test-generation.md`
- `read_path:specs/crates/cclab-sdd/generate/ux-pattern-library.md`
- `read_path:specs/crates/cclab-sdd/interfaces/cli/commands.md`
- `read_path:specs/crates/cclab-sdd/interfaces/cli/sdd-cli.md`
- `read_path:specs/crates/cclab-sdd/interfaces/lens/lens-cli-subcommands.md`
- `read_path:specs/crates/cclab-sdd/interfaces/lens/lens-pdg-mcp-tools.md`
- `read_path:specs/crates/cclab-sdd/interfaces/tools/artifact-tools.md`
- `read_path:specs/crates/cclab-sdd/interfaces/tools/utility-tools.md`
- `read_path:specs/crates/cclab-sdd/interfaces/tools/workflow-tools.md`
- `read_path:specs/crates/cclab-sdd/logic/agent-context-builder.md`
- `read_path:specs/crates/cclab-sdd/logic/agent-output-format.md`
- `read_path:specs/crates/cclab-sdd/logic/analysis-tools.md`
- `read_path:specs/crates/cclab-sdd/logic/cclab-lens-spec.md`
- `read_path:specs/crates/cclab-sdd/logic/change-merge.md`
- `read_path:specs/crates/cclab-sdd/logic/change-spec-logic.md`
- `read_path:specs/crates/cclab-sdd/logic/change-spec.md`
- `read_path:specs/crates/cclab-sdd/logic/check-alignment.md`
- `read_path:specs/crates/cclab-sdd/logic/class-diagram.md`
- `read_path:specs/crates/cclab-sdd/logic/code-analysis-service-v2.md`
- `read_path:specs/crates/cclab-sdd/logic/executor-resolution.md`
- `read_path:specs/crates/cclab-sdd/logic/implement-task.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-README.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-beyond-ide.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-codegen-unification.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-comprehensive.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-full-upgrade-spec.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-index-storage.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-lang-support.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-markdown.md`
- `read_path:specs/crates/cclab-sdd/logic/lens-yaml-codegen.md`
- `read_path:specs/crates/cclab-sdd/logic/merge-lens-into-sdd-spec.md`
- `read_path:specs/crates/cclab-sdd/logic/post-clarifications.md`
- `read_path:specs/crates/cclab-sdd/logic/pre-clarifications.md`
- `read_path:specs/crates/cclab-sdd/logic/python-pdg-core.md`
- `read_path:specs/crates/cclab-sdd/logic/refactoring-api.md`
- `read_path:specs/crates/cclab-sdd/logic/reference-context.md`
- `read_path:specs/crates/cclab-sdd/logic/restructure-input.md`
- `read_path:specs/crates/cclab-sdd/logic/rust-symbol-analysis.md`
- `read_path:specs/crates/cclab-sdd/logic/scope-resolution.md`
- `read_path:specs/crates/cclab-sdd/logic/semantic-search-api.md`
- `read_path:specs/crates/cclab-sdd/logic/spec-diff-codegen.md`
- `read_path:specs/crates/cclab-sdd/logic/spec-structure.md`
- `read_path:specs/crates/cclab-sdd/logic/state-machine.md`
- `read_path:specs/crates/cclab-sdd/logic/tech-stack-inference.md`
- `read_path:specs/crates/cclab-sdd/logic/type-inference-pipeline.md`
- `read_path:specs/crates/cclab-sdd/logic/usage-examples.md`
- `read_path:specs/crates/cclab-sdd/skills/agent.md`
- `read_path:specs/crates/cclab-sdd/skills/fillback.md`
- `read_path:specs/crates/cclab-sdd/skills/merge.md`
- `read_path:specs/crates/cclab-sdd/skills/revise-artifact.md`
- `read_path:specs/crates/cclab-sdd/skills/run-change.md`
- `read_path:specs/crates/cclab-sdd/tools/sdd-codegen-testgen-spec.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/analyze-code-for-spec.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/delegate-agent.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/fetch-issues.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/list-changed-files.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/platform-sync.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/read-artifact.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/read-implementation-summary.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/validate-change.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/validate-spec-completeness.md`
- `read_path:specs/crates/cclab-sdd/tools/utils/write-artifact.md`


Read these specs using the Read tool (file paths under `/Users/chrischeng/projects/wt/sdd/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: cclab/changes/1142/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
cclab sdd artifact create-reference-context 1142 cclab/changes/1142/payloads/create-reference-context.json
```