# Task: Gather Reference Context for Group 'subagent-dispatch' (Change 'sdd-subagent-mode')

Issues: #1046_feat-sdd-subagent-execution-mode-claude-code-agent

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
│   ├── template-mcp-configs.md
│   ├── test-generation.md
│   └── ux-pattern-library.md
├── interfaces
│   ├── cli
│   │   ├── commands.md
│   │   └── sdd-cli.md
│   └── tools
│       ├── artifact-tools.md
│       ├── utility-tools.md
│       └── workflow-tools.md
├── logic
│   ├── change-merge.md
│   ├── change-spec-logic.md
│   ├── change-spec.md
│   ├── executor-resolution.md
│   ├── implement-task.md
│   ├── merge-lens-into-sdd-spec.md
│   ├── post-clarifications.md
│   ├── pre-clarifications.md
│   ├── reference-context.md
│   ├── restructure-input.md
│   ├── scope-resolution.md
│   ├── state-machine.md
│   └── tech-stack-inference.md
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
   `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-subagent-mode/groups/subagent-dispatch/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, prompt, cli, changes]
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
- `read_path:specs/crates/cclab-sdd/generate/template-mcp-configs.md`
- `read_path:specs/crates/cclab-sdd/generate/test-generation.md`
- `read_path:specs/crates/cclab-sdd/generate/ux-pattern-library.md`
- `read_path:specs/crates/cclab-sdd/interfaces/cli/commands.md`
- `read_path:specs/crates/cclab-sdd/interfaces/cli/sdd-cli.md`
- `read_path:specs/crates/cclab-sdd/interfaces/tools/artifact-tools.md`
- `read_path:specs/crates/cclab-sdd/interfaces/tools/utility-tools.md`
- `read_path:specs/crates/cclab-sdd/interfaces/tools/workflow-tools.md`
- `read_path:specs/crates/cclab-sdd/logic/change-merge.md`
- `read_path:specs/crates/cclab-sdd/logic/change-spec-logic.md`
- `read_path:specs/crates/cclab-sdd/logic/change-spec.md`
- `read_path:specs/crates/cclab-sdd/logic/executor-resolution.md`
- `read_path:specs/crates/cclab-sdd/logic/implement-task.md`
- `read_path:specs/crates/cclab-sdd/logic/merge-lens-into-sdd-spec.md`
- `read_path:specs/crates/cclab-sdd/logic/post-clarifications.md`
- `read_path:specs/crates/cclab-sdd/logic/pre-clarifications.md`
- `read_path:specs/crates/cclab-sdd/logic/reference-context.md`
- `read_path:specs/crates/cclab-sdd/logic/restructure-input.md`
- `read_path:specs/crates/cclab-sdd/logic/scope-resolution.md`
- `read_path:specs/crates/cclab-sdd/logic/state-machine.md`
- `read_path:specs/crates/cclab-sdd/logic/tech-stack-inference.md`
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


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-sdd/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: cclab/changes/sdd-subagent-mode/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
cclab sdd artifact create-reference-context sdd-subagent-mode cclab/changes/sdd-subagent-mode/payloads/create-reference-context.json
```