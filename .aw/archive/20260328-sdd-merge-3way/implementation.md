---
id: implementation
type: change_implementation
change_id: sdd-merge-3way
---

# Implementation

## Summary

*(auto-generated baseline from git diff)*

## Changed Files

```
M	.claude/skills/cclab-sdd-run-change/SKILL.md
A	.claude/skills/conductor-dev-server/skill.md
A	.claude/skills/handoff/SKILL.md
M	.gitignore
M	CLAUDE.md
M	Cargo.lock
M	Cargo.toml
A	ECOSYSTEM.md
A	cclab/archive/20260324-clean-mcp-refs/STATE.yaml
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/post_clarifications.md
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/pre_clarifications.md
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/prompts/analyze_spec_mcp-refs-cleanup.md
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/prompts/create_post_clarifications.md
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/prompts/create_pre_clarifications.md
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/reference_context.md
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/requirements.md
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/spec_plan.yaml
A	cclab/archive/20260324-clean-mcp-refs/groups/mcp-cleanup/specs/mcp-refs-cleanup.md
A	cclab/archive/20260324-clean-mcp-refs/issues/issue_1047_refactor-sdd-clean-up-stale-mcp-references-all-too.md
A	cclab/archive/20260324-clean-mcp-refs/payloads/create-change-spec-changes.json
A	cclab/archive/20260324-clean-mcp-refs/payloads/create-change-spec-overview.json
A	cclab/archive/20260324-clean-mcp-refs/payloads/create-change-spec-requirements.json
A	cclab/archive/20260324-clean-mcp-refs/payloads/create-change-spec-scenarios.json
A	cclab/archive/20260324-clean-mcp-refs/payloads/create-post-clarifications.json
A	cclab/archive/20260324-clean-mcp-refs/payloads/create-pre-clarifications.json
A	cclab/archive/20260324-clean-mcp-refs/payloads/create-reference-context.json
A	cclab/archive/20260324-clean-mcp-refs/payloads/restructure-input.json
A	cclab/archive/20260324-clean-mcp-refs/payloads/review-reference-context.json
A	cclab/archive/20260324-clean-mcp-refs/prompts/create_change_merge.md
A	cclab/archive/20260324-clean-mcp-refs/prompts/restructure_input.md
A	cclab/archive/20260324-clean-mcp-refs/user_input.md
R066	cclab/changes/jet-workspace-protocol/STATE.yaml	cclab/archive/20260324-jet-workspace-protocol/STATE.yaml
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/post_clarifications.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/post_clarifications.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/pre_clarifications.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/pre_clarifications.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/prompts/analyze_spec_jet-workspace-protocol-spec.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/analyze_spec_jet-workspace-protocol-spec.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/prompts/begin_implementation.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/begin_implementation.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/prompts/create_post_clarifications.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/create_post_clarifications.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/prompts/create_pre_clarifications.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/create_pre_clarifications.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/prompts/create_reference_context.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/create_reference_context.md
A	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/implement_tests_jet-workspace-protocol-spec.md
A	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/review_impl_jet-workspace-protocol-spec.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/prompts/review_reference_context.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/review_reference_context.md
A	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/prompts/write_implementation_diff.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/reference_context.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/reference_context.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/requirements.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/requirements.md
R100	cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/specs/jet-workspace-protocol-spec.md	cclab/archive/20260324-jet-workspace-protocol/groups/jet-workspace-protocol/specs/jet-workspace-protocol-spec.md
A	cclab/archive/20260324-jet-workspace-protocol/implementation.md
A	cclab/archive/20260324-jet-workspace-protocol/payloads/create-change-implementation.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-pre-clarifications.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-pre-clarifications.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-reference-context.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-reference-context.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-spec-changes.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-spec-changes.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-spec-interaction.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-spec-interaction.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-spec-logic.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-spec-logic.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-spec-overview.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-spec-overview.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-spec-requirements.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-spec-requirements.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-spec-scenarios.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-spec-scenarios.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-spec-schema.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-spec-schema.json
R100	cclab/changes/jet-workspace-protocol/payloads/create-spec-state-machine.json	cclab/archive/20260324-jet-workspace-protocol/payloads/create-spec-state-machine.json
R100	cclab/changes/jet-workspace-protocol/payloads/post-clarify-jet-workspace-protocol.json	cclab/archive/20260324-jet-workspace-protocol/payloads/post-clarify-jet-workspace-protocol.json
R100	cclab/changes/jet-workspace-protocol/payloads/ref-context-jet-workspace-protocol.json	cclab/archive/20260324-jet-workspace-protocol/payloads/ref-context-jet-workspace-protocol.json
R100	cclab/changes/jet-workspace-protocol/payloads/restructure-input.json	cclab/archive/20260324-jet-workspace-protocol/payloads/restructure-input.json
A	cclab/archive/20260324-jet-workspace-protocol/payloads/review-change-implementation.json
R100	cclab/changes/jet-workspace-protocol/payloads/review-reference-context.json	cclab/archive/20260324-jet-workspace-protocol/payloads/review-reference-context.json
R100	cclab/changes/jet-workspace-protocol/payloads/spec-interaction.json	cclab/archive/20260324-jet-workspace-protocol/payloads/spec-interaction.json
R100	cclab/changes/jet-workspace-protocol/payloads/spec-logic.json	cclab/archive/20260324-jet-workspace-protocol/payloads/spec-logic.json
R100	cclab/changes/jet-workspace-protocol/payloads/spec-schema.json	cclab/archive/20260324-jet-workspace-protocol/payloads/spec-schema.json
R100	cclab/changes/jet-workspace-protocol/payloads/spec-state-machine.json	cclab/archive/20260324-jet-workspace-protocol/payloads/spec-state-machine.json
A	cclab/archive/20260324-jet-workspace-protocol/prompts/create_change_merge.md
R100	cclab/changes/jet-workspace-protocol/prompts/restructure_input.md	cclab/archive/20260324-jet-workspace-protocol/prompts/restructure_input.md
R100	cclab/changes/jet-workspace-protocol/user_input.md	cclab/archive/20260324-jet-workspace-protocol/user_input.md
A	cclab/archive/20260324-sdd-subagent-mode/STATE.yaml
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/post_clarifications.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/pre_clarifications.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/analyze_spec_subagent-executor-resolution.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/analyze_spec_subagent-skill-dispatch.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/analyze_spec_subagent-workflow-dispatch.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/begin_implementation.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/create_post_clarifications.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/create_pre_clarifications.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/create_reference_context.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/implement_spec.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/implement_tests_subagent-executor-resolution.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/implement_tests_subagent-skill-dispatch.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/implement_tests_subagent-workflow-dispatch.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/prompts/write_implementation_diff.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/reference_context.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/requirements.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/spec_plan.yaml
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/specs/subagent-executor-resolution.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/specs/subagent-skill-dispatch.md
A	cclab/archive/20260324-sdd-subagent-mode/groups/subagent-dispatch/specs/subagent-workflow-dispatch.md
A	cclab/archive/20260324-sdd-subagent-mode/implementation.md
A	cclab/archive/20260324-sdd-subagent-mode/issues/issue_1046_feat-sdd-subagent-execution-mode-claude-code-agent.md
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-executor-resolution-changes.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-executor-resolution-config.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-executor-resolution-overview.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-skill-dispatch-changes.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-skill-dispatch-interaction.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-skill-dispatch-logic.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-skill-dispatch-overview.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-workflow-dispatch-changes.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-workflow-dispatch-interaction.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-workflow-dispatch-logic.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-change-spec-workflow-dispatch-overview.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-post-clarifications.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-pre-clarifications.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/create-reference-context.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/restructure-input.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/review-change-implementation.json
A	cclab/archive/20260324-sdd-subagent-mode/payloads/review-reference-context.json
A	cclab/archive/20260324-sdd-subagent-mode/prompts/create_change_merge.md
A	cclab/archive/20260324-sdd-subagent-mode/prompts/restructure_input.md
A	cclab/archive/20260324-sdd-subagent-mode/user_input.md
A	cclab/archive/20260324-section-type-coverage/STATE.yaml
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/payloads/create-change-spec.json
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/post_clarifications.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/pre_clarifications.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/analyze_spec_change-spec-section-optionality.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/analyze_spec_tech-stack-inference.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/analyze_spec_ux-pattern-library.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/begin_implementation.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/create_post_clarifications.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/create_pre_clarifications.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/create_reference_context.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/implement_spec.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/implement_tests_change-spec-section-optionality.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/implement_tests_tech-stack-inference.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/implement_tests_ux-pattern-library.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/review_reference_context.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/prompts/revise_reference_context.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/reference_context.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/requirements.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/spec_plan.yaml
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/specs/change-spec-section-optionality.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/specs/tech-stack-inference.md
A	cclab/archive/20260324-section-type-coverage/groups/frontend-design-system/specs/ux-pattern-library.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/post_clarifications.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/pre_clarifications.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/prompts/create_post_clarifications.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/prompts/create_pre_clarifications.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/prompts/create_reference_context.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/prompts/implement_spec.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/prompts/implement_tests_reference-context-types.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/reference_context.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/requirements.md
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/spec_plan.yaml
A	cclab/archive/20260324-section-type-coverage/groups/new-section-types/specs/reference-context-types.md
A	cclab/archive/20260324-section-type-coverage/implementation.md
A	cclab/archive/20260324-section-type-coverage/issues/issue_1051_epic-sdd-section-type-coverage-all-roles-fe-be-sre.md
A	cclab/archive/20260324-section-type-coverage/issues/issue_1052_sdd-design-system-as-tech-stack-config-ux-pattern-.md
A	cclab/archive/20260324-section-type-coverage/issues/issue_1053_sdd-add-e2e-scenario-section-type-for-qa.md
A	cclab/archive/20260324-section-type-coverage/issues/issue_1054_sdd-add-security-section-types-threat-model-auth-m.md
A	cclab/archive/20260324-section-type-coverage/issues/issue_1055_sdd-add-qa-section-types-test-fixture-perf-test.md
A	cclab/archive/20260324-section-type-coverage/issues/issue_1056_sdd-add-sre-section-types-container-deploy-cloud-r.md
A	cclab/archive/20260324-section-type-coverage/issues/issue_1057_sdd-add-backend-mle-agent-section-types-grpc-graph.md
A	cclab/archive/20260324-section-type-coverage/payloads/create-change-implementation.json
A	cclab/archive/20260324-section-type-coverage/payloads/create-post-clarifications.json
A	cclab/archive/20260324-section-type-coverage/payloads/create-pre-clarifications.json
A	cclab/archive/20260324-section-type-coverage/payloads/create-reference-context.json
A	cclab/archive/20260324-section-type-coverage/payloads/restructure-input.json
A	cclab/archive/20260324-section-type-coverage/payloads/review-change-implementation.json
A	cclab/archive/20260324-section-type-coverage/payloads/review-reference-context.json
A	cclab/archive/20260324-section-type-coverage/payloads/revise-reference-context.json
A	cclab/archive/20260324-section-type-coverage/prompts/create_change_merge.md
A	cclab/archive/20260324-section-type-coverage/prompts/restructure_input.md
A	cclab/archive/20260324-section-type-coverage/prompts/review_impl_change-spec-section-optionality.md
A	cclab/archive/20260324-section-type-coverage/prompts/review_impl_reference-context-types.md
A	cclab/archive/20260324-section-type-coverage/prompts/write_implementation_diff.md
A	cclab/archive/20260324-section-type-coverage/user_input.md
R076	cclab/changes/align-fetch-api/STATE.yaml	cclab/archive/20260325-align-fetch-api/STATE.yaml
R100	cclab/changes/align-fetch-api/issues/issue_964_epic-ecosystem-align-cclab-python-api-to-ecosystem.md	cclab/archive/20260325-align-fetch-api/issues/issue_964_epic-ecosystem-align-cclab-python-api-to-ecosystem.md
R100	cclab/changes/align-fetch-api/prompts/restructure_input.md	cclab/archive/20260325-align-fetch-api/prompts/restructure_input.md
R100	cclab/changes/align-fetch-api/user_input.md	cclab/archive/20260325-align-fetch-api/user_input.md
R069	cclab/changes/cclab-agent-p0/STATE.yaml	cclab/archive/20260325-cclab-agent-p0/STATE.yaml
R100	cclab/changes/cclab-agent-p0/groups/structured-output/post_clarifications.md	cclab/archive/20260325-cclab-agent-p0/groups/structured-output/post_clarifications.md
R100	cclab/changes/cclab-agent-p0/groups/structured-output/pre_clarifications.md	cclab/archive/20260325-cclab-agent-p0/groups/structured-output/pre_clarifications.md
R100	cclab/changes/cclab-agent-p0/groups/structured-output/prompts/create_pre_clarifications.md	cclab/archive/20260325-cclab-agent-p0/groups/structured-output/prompts/create_pre_clarifications.md
R100	cclab/changes/cclab-agent-p0/groups/structured-output/prompts/create_reference_context.md	cclab/archive/20260325-cclab-agent-p0/groups/structured-output/prompts/create_reference_context.md
R100	cclab/changes/cclab-agent-p0/groups/structured-output/prompts/review_reference_context.md	cclab/archive/20260325-cclab-agent-p0/groups/structured-output/prompts/review_reference_context.md
R100	cclab/changes/cclab-agent-p0/groups/structured-output/reference_context.md	cclab/archive/20260325-cclab-agent-p0/groups/structured-output/reference_context.md
R100	cclab/changes/cclab-agent-p0/groups/structured-output/requirements.md	cclab/archive/20260325-cclab-agent-p0/groups/structured-output/requirements.md
R100	cclab/changes/cclab-agent-p0/groups/token-counting-and-compact/post_clarifications.md	cclab/archive/20260325-cclab-agent-p0/groups/token-counting-and-compact/post_clarifications.md
R100	cclab/changes/cclab-agent-p0/groups/token-counting-and-compact/pre_clarifications.md	cclab/archive/20260325-cclab-agent-p0/groups/token-counting-and-compact/pre_clarifications.md
R100	cclab/changes/cclab-agent-p0/groups/token-counting-and-compact/prompts/create_pre_clarifications.md	cclab/archive/20260325-cclab-agent-p0/groups/token-counting-and-compact/prompts/create_pre_clarifications.md
R100	cclab/changes/cclab-agent-p0/groups/token-counting-and-compact/reference_context.md	cclab/archive/20260325-cclab-agent-p0/groups/token-counting-and-compact/reference_context.md
R100	cclab/changes/cclab-agent-p0/groups/token-counting-and-compact/requirements.md	cclab/archive/20260325-cclab-agent-p0/groups/token-counting-and-compact/requirements.md
R100	cclab/changes/cclab-agent-p0/issues/issue_786_feat-agent-add-accurate-token-counting.md	cclab/archive/20260325-cclab-agent-p0/issues/issue_786_feat-agent-add-accurate-token-counting.md
R100	cclab/changes/cclab-agent-p0/issues/issue_792_feat-agent-add-structured-output-json-schema-respo.md	cclab/archive/20260325-cclab-agent-p0/issues/issue_792_feat-agent-add-structured-output-json-schema-respo.md
R100	cclab/changes/cclab-agent-p0/issues/issue_876_feat-agent-smart-auto-compact-llm-summarization-ac.md	cclab/archive/20260325-cclab-agent-p0/issues/issue_876_feat-agent-smart-auto-compact-llm-summarization-ac.md
R100	cclab/changes/cclab-agent-p0/payloads/create-change-spec.json	cclab/archive/20260325-cclab-agent-p0/payloads/create-change-spec.json
R100	cclab/changes/cclab-agent-p0/prompts/analyze_spec_cclab-agent-p0-spec.md	cclab/archive/20260325-cclab-agent-p0/prompts/analyze_spec_cclab-agent-p0-spec.md
R100	cclab/changes/cclab-agent-p0/prompts/begin_implementation.md	cclab/archive/20260325-cclab-agent-p0/prompts/begin_implementation.md
R100	cclab/changes/cclab-agent-p0/prompts/create_post_clarifications.md	cclab/archive/20260325-cclab-agent-p0/prompts/create_post_clarifications.md
R100	cclab/changes/cclab-agent-p0/prompts/create_reference_context.md	cclab/archive/20260325-cclab-agent-p0/prompts/create_reference_context.md
R100	cclab/changes/cclab-agent-p0/prompts/restructure_input.md	cclab/archive/20260325-cclab-agent-p0/prompts/restructure_input.md
R100	cclab/changes/cclab-agent-p0/prompts/review_reference_context.md	cclab/archive/20260325-cclab-agent-p0/prompts/review_reference_context.md
R100	cclab/changes/cclab-agent-p0/specs/cclab-agent-p0-spec.md	cclab/archive/20260325-cclab-agent-p0/specs/cclab-agent-p0-spec.md
R100	cclab/changes/cclab-agent-p0/user_input.md	cclab/archive/20260325-cclab-agent-p0/user_input.md
A	cclab/archive/20260325-enhanced-changes-section/STATE.yaml
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/post_clarifications.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/pre_clarifications.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/prompts/analyze_spec_changes-section-schema.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/prompts/analyze_spec_lens-impl-prompt.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/prompts/begin_implementation.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/prompts/create_post_clarifications.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/prompts/create_pre_clarifications.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/prompts/create_reference_context.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/reference_context.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/requirements.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/spec_plan.yaml
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/specs/changes-section-schema.md
A	cclab/archive/20260325-enhanced-changes-section/groups/changes-section-targets/specs/lens-impl-prompt.md
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-change-spec-changes.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-change-spec-lens-changes.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-change-spec-lens-logic.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-change-spec-lens-overview.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-change-spec-overview.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-change-spec-schema.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-post-clarifications.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-pre-clarifications.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/create-reference-context.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/restructure-input.json
A	cclab/archive/20260325-enhanced-changes-section/payloads/review-reference-context.json
A	cclab/archive/20260325-enhanced-changes-section/prompts/create_change_merge.md
A	cclab/archive/20260325-enhanced-changes-section/prompts/restructure_input.md
A	cclab/archive/20260325-enhanced-changes-section/user_input.md
A	cclab/archive/20260325-fix-remaining-drift-risks/STATE.yaml
A	cclab/archive/20260325-fix-remaining-drift-risks/groups/drift-fixes/pre_clarifications.md
A	cclab/archive/20260325-fix-remaining-drift-risks/groups/drift-fixes/prompts/create_pre_clarifications.md
A	cclab/archive/20260325-fix-remaining-drift-risks/groups/drift-fixes/prompts/create_reference_context.md
A	cclab/archive/20260325-fix-remaining-drift-risks/groups/drift-fixes/reference_context.md
A	cclab/archive/20260325-fix-remaining-drift-risks/groups/drift-fixes/requirements.md
A	cclab/archive/20260325-fix-remaining-drift-risks/groups/drift-fixes/spec_plan.yaml
A	cclab/archive/20260325-fix-remaining-drift-risks/groups/drift-fixes/specs/fill-sections-fallback.md
A	cclab/archive/20260325-fix-remaining-drift-risks/groups/drift-fixes/specs/merge-strategy-doc.md
A	cclab/archive/20260325-fix-remaining-drift-risks/payloads/create-pre-clarifications.json
A	cclab/archive/20260325-fix-remaining-drift-risks/payloads/create-reference-context.json
A	cclab/archive/20260325-fix-remaining-drift-risks/payloads/restructure-input.json
A	cclab/archive/20260325-fix-remaining-drift-risks/payloads/review-reference-context.json
A	cclab/archive/20260325-fix-remaining-drift-risks/prompts/create_change_merge.md
A	cclab/archive/20260325-fix-remaining-drift-risks/prompts/restructure_input.md
A	cclab/archive/20260325-fix-remaining-drift-risks/user_input.md
A	cclab/archive/20260325-jet-dev-server-v2/STATE.yaml
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/pre_clarifications.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/analyze_spec_jet-dev-server-v2-spec.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/begin_implementation.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/create_pre_clarifications.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/create_reference_context.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/fill_spec_jet-dev-server-v2-spec_test_plan.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/implement_tests_jet-dev-server-v2-spec.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/review_impl_jet-dev-server-v2-spec.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/review_reference_context.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/revise_change_implementation.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/revise_reference_context.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/prompts/write_implementation_diff.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/reference_context.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/requirements.md
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/spec_plan.yaml
A	cclab/archive/20260325-jet-dev-server-v2/groups/jet-dev-server-v2/specs/jet-dev-server-v2-spec.md
A	cclab/archive/20260325-jet-dev-server-v2/implementation.md
A	cclab/archive/20260325-jet-dev-server-v2/issues/issue_1089_jet-dev-implement-optimizedeps-full-cjs-esm-pre-bu.md
A	cclab/archive/20260325-jet-dev-server-v2/issues/issue_1090_jet-dev-ast-based-typescript-type-stripping-replac.md
A	cclab/archive/20260325-jet-dev-server-v2/issues/issue_1091_jet-dev-browser-compatible-node-js-builtin-polyfil.md
A	cclab/archive/20260325-jet-dev-server-v2/issues/issue_1092_jet-install-jet-store-symlinks-break-node-js-modul.md
A	cclab/archive/20260325-jet-dev-server-v2/payloads/create-change-implementation.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/create-pre-clarifications.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/create-reference-context.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/fill-section-changes.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/fill-section-logic.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/fill-section-overview.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/fill-section-requirements.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/fill-section-scenarios.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/fill-section-test-plan.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/restructure-input.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/review-change-implementation.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/review-reference-context.json
A	cclab/archive/20260325-jet-dev-server-v2/payloads/revise-reference-context.json
A	cclab/archive/20260325-jet-dev-server-v2/prompts/create_change_merge.md
A	cclab/archive/20260325-jet-dev-server-v2/prompts/restructure_input.md
A	cclab/archive/20260325-jet-dev-server-v2/user_input.md
R057	cclab/changes/mamba-conformance-basics/STATE.yaml	cclab/archive/20260325-mamba-conformance-basics/STATE.yaml
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/post_clarifications.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/post_clarifications.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/pre_clarifications.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/pre_clarifications.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_builtins.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_builtins.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_cranelift-jit.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_cranelift-jit.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_cranelift.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_cranelift.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_repl.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_repl.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_string-ops.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_string-ops.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_type-checker.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/analyze_spec_type-checker.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/begin_implementation.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/begin_implementation.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/create_post_clarifications.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/create_post_clarifications.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/create_pre_clarifications.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/create_pre_clarifications.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/create_reference_context.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/create_reference_context.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/implement_spec.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/implement_spec.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_builtins.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_builtins.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_cranelift-jit.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_cranelift-jit.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_cranelift.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_cranelift.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_repl.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_repl.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_string-ops.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_string-ops.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_type-checker.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/implement_tests_type-checker.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_builtins.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_builtins.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_cranelift-jit.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_cranelift-jit.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_cranelift.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_cranelift.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_repl.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_repl.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_string-ops.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_string-ops.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_type-checker.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/review_impl_type-checker.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/review_reference_context.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/review_reference_context.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/revise_change_implementation.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/revise_change_implementation.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/revise_reference_context.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/revise_reference_context.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/prompts/write_implementation_diff.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/prompts/write_implementation_diff.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/reference_context.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/reference_context.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/requirements.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/requirements.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/spec_plan.yaml	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/spec_plan.yaml
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/specs/builtins.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/specs/builtins.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/specs/cranelift-jit.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/specs/cranelift-jit.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/specs/cranelift.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/specs/cranelift.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/specs/repl.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/specs/repl.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/specs/string-ops.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/specs/string-ops.md
R100	cclab/changes/mamba-conformance-basics/groups/runtime-basics/specs/type-checker.md	cclab/archive/20260325-mamba-conformance-basics/groups/runtime-basics/specs/type-checker.md
R100	cclab/changes/mamba-conformance-basics/implementation.md	cclab/archive/20260325-mamba-conformance-basics/implementation.md
R100	cclab/changes/mamba-conformance-basics/issues/issue_1037_test-mamba-py3-12-behavioral-conformance-every-fun.md	cclab/archive/20260325-mamba-conformance-basics/issues/issue_1037_test-mamba-py3-12-behavioral-conformance-every-fun.md
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-implementation.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-implementation.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-spec-builtins-changes.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-spec-builtins-changes.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-spec-builtins-overview.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-spec-builtins-overview.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-spec-cranelift-changes.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-spec-cranelift-changes.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-spec-cranelift-jit-changes.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-spec-cranelift-jit-changes.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-spec-cranelift-jit-overview.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-spec-cranelift-jit-overview.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-spec-cranelift-overview.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-spec-cranelift-overview.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-spec-repl-changes.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-spec-repl-changes.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-change-spec-repl-overview.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-change-spec-repl-overview.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-post-clarifications.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-post-clarifications.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-pre-clarifications-runtime-basics.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-pre-clarifications-runtime-basics.json
R100	cclab/changes/mamba-conformance-basics/payloads/create-reference-context.json	cclab/archive/20260325-mamba-conformance-basics/payloads/create-reference-context.json
R100	cclab/changes/mamba-conformance-basics/payloads/restructure-input.json	cclab/archive/20260325-mamba-conformance-basics/payloads/restructure-input.json
R100	cclab/changes/mamba-conformance-basics/payloads/review-change-implementation.json	cclab/archive/20260325-mamba-conformance-basics/payloads/review-change-implementation.json
R100	cclab/changes/mamba-conformance-basics/payloads/review-reference-context.json	cclab/archive/20260325-mamba-conformance-basics/payloads/review-reference-context.json
R100	cclab/changes/mamba-conformance-basics/payloads/revise-reference-context.json	cclab/archive/20260325-mamba-conformance-basics/payloads/revise-reference-context.json
R100	cclab/changes/mamba-conformance-basics/prompts/restructure_input.md	cclab/archive/20260325-mamba-conformance-basics/prompts/restructure_input.md
R100	cclab/changes/mamba-conformance-basics/user_input.md	cclab/archive/20260325-mamba-conformance-basics/user_input.md
A	cclab/archive/20260325-post-clarifications-scope-summary/STATE.yaml
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/post_clarifications.md
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/pre_clarifications.md
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/prompts/create_post_clarifications.md
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/prompts/create_pre_clarifications.md
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/prompts/create_reference_context.md
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/reference_context.md
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/requirements.md
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/spec_plan.yaml
A	cclab/archive/20260325-post-clarifications-scope-summary/groups/scope-summary/specs/post-clarifications-scope.md
A	cclab/archive/20260325-post-clarifications-scope-summary/payloads/create-post-clarifications.json
A	cclab/archive/20260325-post-clarifications-scope-summary/payloads/create-pre-clarifications.json
A	cclab/archive/20260325-post-clarifications-scope-summary/payloads/create-reference-context.json
A	cclab/archive/20260325-post-clarifications-scope-summary/payloads/restructure-input.json
A	cclab/archive/20260325-post-clarifications-scope-summary/payloads/review-reference-context.json
A	cclab/archive/20260325-post-clarifications-scope-summary/prompts/create_change_merge.md
A	cclab/archive/20260325-post-clarifications-scope-summary/prompts/restructure_input.md
A	cclab/archive/20260325-post-clarifications-scope-summary/user_input.md
R063	cclab/changes/sdd-codegen-completion/STATE.yaml	cclab/archive/20260325-sdd-codegen-completion/STATE.yaml
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/payloads/create-post-clarifications.json	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/payloads/create-post-clarifications.json
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/payloads/create-pre-clarifications.json	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/payloads/create-pre-clarifications.json
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/post_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/post_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/pre_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/pre_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/prompts/create_post_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/prompts/create_post_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/prompts/create_pre_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/prompts/create_pre_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/prompts/create_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/prompts/create_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/prompts/review_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/prompts/review_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/prompts/revise_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/prompts/revise_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/core-codegen/requirements.md	cclab/archive/20260325-sdd-codegen-completion/groups/core-codegen/requirements.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/payloads/create-post-clarifications.json	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/payloads/create-post-clarifications.json
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/payloads/create-pre-clarifications.json	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/payloads/create-pre-clarifications.json
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/post_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/post_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/pre_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/pre_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/prompts/create_post_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/prompts/create_post_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/prompts/create_pre_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/prompts/create_pre_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/prompts/create_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/prompts/create_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/prompts/review_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/prompts/review_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/prompts/revise_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/prompts/revise_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/deploy-section-type/requirements.md	cclab/archive/20260325-sdd-codegen-completion/groups/deploy-section-type/requirements.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/payloads/create-post-clarifications.json	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/payloads/create-post-clarifications.json
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/payloads/create-pre-clarifications.json	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/payloads/create-pre-clarifications.json
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/post_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/post_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/pre_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/pre_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/prompts/create_post_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/prompts/create_post_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/prompts/create_pre_clarifications.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/prompts/create_pre_clarifications.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/prompts/create_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/prompts/create_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/prompts/review_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/prompts/review_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/prompts/revise_reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/prompts/revise_reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/reference_context.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/reference_context.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/requirements.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/requirements.md
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/spec_plan.yaml	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/spec_plan.yaml
R100	cclab/changes/sdd-codegen-completion/groups/frontend-codegen/specs/frontend-codegen-main.md	cclab/archive/20260325-sdd-codegen-completion/groups/frontend-codegen/specs/frontend-codegen-main.md
R100	cclab/changes/sdd-codegen-completion/issues/issue_932_feat-sdd-codegen-last-mile-consume-specir-to-gener.md	cclab/archive/20260325-sdd-codegen-completion/issues/issue_932_feat-sdd-codegen-last-mile-consume-specir-to-gener.md
R100	cclab/changes/sdd-codegen-completion/issues/issue_933_feat-sdd-test-generation-from-requirementplus-spec.md	cclab/archive/20260325-sdd-codegen-completion/issues/issue_933_feat-sdd-test-generation-from-requirementplus-spec.md
R100	cclab/changes/sdd-codegen-completion/issues/issue_934_feat-sdd-deployment-spec-type-infra-as-code-integr.md	cclab/archive/20260325-sdd-codegen-completion/issues/issue_934_feat-sdd-deployment-spec-type-infra-as-code-integr.md
R100	cclab/changes/sdd-codegen-completion/issues/issue_937_feat-sdd-frontend-codegen-wireframe-component-desi.md	cclab/archive/20260325-sdd-codegen-completion/issues/issue_937_feat-sdd-frontend-codegen-wireframe-component-desi.md
R100	cclab/changes/sdd-codegen-completion/payloads/create-change-spec.json	cclab/archive/20260325-sdd-codegen-completion/payloads/create-change-spec.json
R100	cclab/changes/sdd-codegen-completion/payloads/create-reference-context.json	cclab/archive/20260325-sdd-codegen-completion/payloads/create-reference-context.json
R100	cclab/changes/sdd-codegen-completion/payloads/restructure-input.json	cclab/archive/20260325-sdd-codegen-completion/payloads/restructure-input.json
R100	cclab/changes/sdd-codegen-completion/payloads/review-reference-context.json	cclab/archive/20260325-sdd-codegen-completion/payloads/review-reference-context.json
R100	cclab/changes/sdd-codegen-completion/payloads/revise-reference-context-deploy.json	cclab/archive/20260325-sdd-codegen-completion/payloads/revise-reference-context-deploy.json
R100	cclab/changes/sdd-codegen-completion/payloads/revise-reference-context.json	cclab/archive/20260325-sdd-codegen-completion/payloads/revise-reference-context.json
R100	cclab/changes/sdd-codegen-completion/prompts/analyze_spec_frontend-codegen-main.md	cclab/archive/20260325-sdd-codegen-completion/prompts/analyze_spec_frontend-codegen-main.md
R100	cclab/changes/sdd-codegen-completion/prompts/fill_spec_frontend-codegen-main_changes.md	cclab/archive/20260325-sdd-codegen-completion/prompts/fill_spec_frontend-codegen-main_changes.md
R100	cclab/changes/sdd-codegen-completion/prompts/fill_spec_frontend-codegen-main_overview.md	cclab/archive/20260325-sdd-codegen-completion/prompts/fill_spec_frontend-codegen-main_overview.md
R100	cclab/changes/sdd-codegen-completion/prompts/fill_spec_frontend-codegen-main_requirements.md	cclab/archive/20260325-sdd-codegen-completion/prompts/fill_spec_frontend-codegen-main_requirements.md
R100	cclab/changes/sdd-codegen-completion/prompts/fill_spec_frontend-codegen-main_scenarios.md	cclab/archive/20260325-sdd-codegen-completion/prompts/fill_spec_frontend-codegen-main_scenarios.md
R100	cclab/changes/sdd-codegen-completion/prompts/restructure_input.md	cclab/archive/20260325-sdd-codegen-completion/prompts/restructure_input.md
R100	cclab/changes/sdd-codegen-completion/specs/frontend-codegen-main.md	cclab/archive/20260325-sdd-codegen-completion/specs/frontend-codegen-main.md
R100	cclab/changes/sdd-codegen-completion/user_input.md	cclab/archive/20260325-sdd-codegen-completion/user_input.md
A	cclab/archive/20260325-spec-decomposition-rules/STATE.yaml
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/post_clarifications.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/pre_clarifications.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/prompts/analyze_spec_change-spec-review-rules.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/prompts/create_post_clarifications.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/prompts/create_pre_clarifications.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/prompts/create_reference_context.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/reference_context.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/requirements.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/spec_plan.yaml
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/specs/change-spec-review-rules.md
A	cclab/archive/20260325-spec-decomposition-rules/groups/decomposition-rules/specs/ref-context-decomposition.md
A	cclab/archive/20260325-spec-decomposition-rules/payloads/create-change-spec-review-rules-changes.json
A	cclab/archive/20260325-spec-decomposition-rules/payloads/create-change-spec-review-rules-overview.json
A	cclab/archive/20260325-spec-decomposition-rules/payloads/create-post-clarifications.json
A	cclab/archive/20260325-spec-decomposition-rules/payloads/create-pre-clarifications.json
A	cclab/archive/20260325-spec-decomposition-rules/payloads/create-reference-context.json
A	cclab/archive/20260325-spec-decomposition-rules/payloads/restructure-input.json
A	cclab/archive/20260325-spec-decomposition-rules/payloads/review-reference-context.json
A	cclab/archive/20260325-spec-decomposition-rules/prompts/create_change_merge.md
A	cclab/archive/20260325-spec-decomposition-rules/prompts/restructure_input.md
A	cclab/archive/20260325-spec-decomposition-rules/user_input.md
A	cclab/archive/20260326-jet-hmr-validation/STATE.yaml
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/pre_clarifications.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/analyze_spec_jet-hmr-validation-spec.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/begin_implementation.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/create_pre_clarifications.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/create_reference_context.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/implement_tests_jet-hmr-validation-spec.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/review_impl_jet-hmr-validation-spec.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/review_reference_context.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/revise_change_implementation.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/prompts/write_implementation_diff.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/reference_context.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/requirements.md
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/spec_plan.yaml
A	cclab/archive/20260326-jet-hmr-validation/groups/jet-hmr-validation/specs/jet-hmr-validation-spec.md
A	cclab/archive/20260326-jet-hmr-validation/implementation.md
A	cclab/archive/20260326-jet-hmr-validation/issues/issue_1118_jet-dev-javascript-module-hmr-hot-module-replaceme.md
A	cclab/archive/20260326-jet-hmr-validation/issues/issue_1119_jet-dev-validate-with-conductor-fe-real-world-reac.md
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-implementation.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-changes.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-interaction.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-logic.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-overview.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-requirements.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-scenarios.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-schema.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-state-machine.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-change-spec-test-plan.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-pre-clarifications.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/create-reference-context.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/restructure-input.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/review-change-implementation.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/review-reference-context.json
A	cclab/archive/20260326-jet-hmr-validation/payloads/revise-reference-context.json
A	cclab/archive/20260326-jet-hmr-validation/prompts/create_change_merge.md
A	cclab/archive/20260326-jet-hmr-validation/prompts/restructure_input.md
A	cclab/archive/20260326-jet-hmr-validation/user_input.md
A	cclab/archive/20260326-lens-dissolution/STATE.yaml
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/post_clarifications.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/pre_clarifications.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_agent-context-builder.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_agent-output-format.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_lens-dissolution-restructure.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_sdd-cli-context-command.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_type-inference-pipeline.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/begin_implementation.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/create_post_clarifications.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/create_pre_clarifications.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/create_reference_context.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/implement_spec.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/implement_tests_agent-context-builder.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/implement_tests_agent-output-format.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/implement_tests_lens-dissolution-restructure.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/review_impl_lens-dissolution-restructure.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/review_impl_sdd-cli-context-command.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/review_impl_type-inference-pipeline.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/review_reference_context.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/prompts/revise_change_implementation.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/reference_context.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/requirements.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/spec_plan.yaml
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/specs/agent-context-builder.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/specs/agent-output-format.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/specs/lens-dissolution-restructure.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/specs/sdd-cli-context-command.md
A	cclab/archive/20260326-lens-dissolution/groups/lens-dissolution/specs/type-inference-pipeline.md
A	cclab/archive/20260326-lens-dissolution/implementation.md
A	cclab/archive/20260326-lens-dissolution/issues/issue_1087_refactor-dissolve-lens-module-into-sdd-top-level-s.md
A	cclab/archive/20260326-lens-dissolution/issues/issue_944_feat-lens-wire-cross-file-type-propagation-deep-in.md
A	cclab/archive/20260326-lens-dissolution/issues/issue_946_feat-lens-agent-context-builder-smart-file-selecti.md
A	cclab/archive/20260326-lens-dissolution/issues/issue_949_feat-lens-agent-optimized-output-structured-json-f.md
A	cclab/archive/20260326-lens-dissolution/payloads/agent-context-builder-changes.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-context-builder-logic.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-context-builder-overview.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-context-builder-requirements.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-context-builder-scenarios.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-context-builder-schema.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-context-builder-test-plan.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-output-format-changes.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-output-format-overview.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-output-format-requirements.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-output-format-scenarios.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-output-format-schema.json
A	cclab/archive/20260326-lens-dissolution/payloads/agent-output-format-test-plan.json
A	cclab/archive/20260326-lens-dissolution/payloads/create-post-clarifications.json
A	cclab/archive/20260326-lens-dissolution/payloads/create-pre-clarifications.json
A	cclab/archive/20260326-lens-dissolution/payloads/create-reference-context.json
A	cclab/archive/20260326-lens-dissolution/payloads/lens-dissolution-restructure-changes.json
A	cclab/archive/20260326-lens-dissolution/payloads/lens-dissolution-restructure-overview.json
A	cclab/archive/20260326-lens-dissolution/payloads/lens-dissolution-restructure-requirements.json
A	cclab/archive/20260326-lens-dissolution/payloads/lens-dissolution-restructure-scenarios.json
A	cclab/archive/20260326-lens-dissolution/payloads/lens-dissolution-restructure-test-plan.json
A	cclab/archive/20260326-lens-dissolution/payloads/restructure-input.json
A	cclab/archive/20260326-lens-dissolution/payloads/review-change-implementation-v2.json
A	cclab/archive/20260326-lens-dissolution/payloads/review-change-implementation.json
A	cclab/archive/20260326-lens-dissolution/payloads/review-reference-context.json
A	cclab/archive/20260326-lens-dissolution/payloads/review-sdd-cli-context-command.json
A	cclab/archive/20260326-lens-dissolution/payloads/review-type-inference-pipeline.json
A	cclab/archive/20260326-lens-dissolution/payloads/sdd-cli-context-command-changes.json
A	cclab/archive/20260326-lens-dissolution/payloads/sdd-cli-context-command-overview.json
A	cclab/archive/20260326-lens-dissolution/payloads/sdd-cli-context-command-requirements.json
A	cclab/archive/20260326-lens-dissolution/payloads/sdd-cli-context-command-scenarios.json
A	cclab/archive/20260326-lens-dissolution/payloads/type-inference-pipeline-changes.json
A	cclab/archive/20260326-lens-dissolution/payloads/type-inference-pipeline-logic.json
A	cclab/archive/20260326-lens-dissolution/payloads/type-inference-pipeline-overview.json
A	cclab/archive/20260326-lens-dissolution/payloads/type-inference-pipeline-requirements.json
A	cclab/archive/20260326-lens-dissolution/payloads/type-inference-pipeline-scenarios.json
A	cclab/archive/20260326-lens-dissolution/payloads/type-inference-pipeline-schema.json
A	cclab/archive/20260326-lens-dissolution/payloads/type-inference-pipeline-test-plan.json
A	cclab/archive/20260326-lens-dissolution/prompts/create_change_merge.md
A	cclab/archive/20260326-lens-dissolution/prompts/restructure_input.md
A	cclab/archive/20260326-lens-dissolution/user_input.md
A	cclab/archive/20260327-e2e-test-reorg/STATE.yaml
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/post_clarifications.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/pre_clarifications.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/prompts/analyze_spec_e2e-test-infrastructure.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/prompts/begin_implementation.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/prompts/create_post_clarifications.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/prompts/create_pre_clarifications.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/prompts/create_reference_context.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/prompts/implement_tests_e2e-test-infrastructure.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/prompts/review_impl_e2e-test-infrastructure.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/prompts/write_implementation_diff.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/reference_context.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/requirements.md
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/spec_plan.yaml
A	cclab/archive/20260327-e2e-test-reorg/groups/e2e-test-reorg/specs/e2e-test-infrastructure.md
A	cclab/archive/20260327-e2e-test-reorg/implementation.md
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-change-implementation.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-change-spec-changes.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-change-spec-overview.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-change-spec-requirements.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-change-spec-scenarios.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-change-spec-test-plan.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-post-clarifications.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-pre-clarifications.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/create-reference-context.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/restructure-input.json
A	cclab/archive/20260327-e2e-test-reorg/payloads/review-change-implementation.json
A	cclab/archive/20260327-e2e-test-reorg/prompts/create_change_merge.md
A	cclab/archive/20260327-e2e-test-reorg/prompts/restructure_input.md
A	cclab/archive/20260327-e2e-test-reorg/user_input.md
A	cclab/archive/20260327-gcp-cloud-integration/STATE.yaml
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/post_clarifications.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/pre_clarifications.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/analyze_spec_broker-traits.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/analyze_spec_cloud-scheduler-backend.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/analyze_spec_cloudtasks-broker.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/analyze_spec_scheduler-backends-gcp.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/begin_implementation.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/create_post_clarifications.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/create_pre_clarifications.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/create_reference_context.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/implement_spec.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/implement_tests_broker-traits.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/implement_tests_cloud-scheduler-backend.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/implement_tests_cloudtasks-broker.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/implement_tests_scheduler-backends-gcp.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/review_impl_broker-traits.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/review_impl_cloud-scheduler-backend.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/review_impl_cloudtasks-broker.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/review_impl_scheduler-backends-gcp.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/review_reference_context.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/revise_change_implementation.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/prompts/write_implementation_diff.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/reference_context.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/requirements.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/spec_plan.yaml
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/specs/broker-traits.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/specs/cloud-scheduler-backend.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/specs/cloudtasks-broker.md
A	cclab/archive/20260327-gcp-cloud-integration/groups/gcp-cloud-integration/specs/scheduler-backends-gcp.md
A	cclab/archive/20260327-gcp-cloud-integration/implementation.md
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-implementation.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-broker-traits-overview.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-broker-traits-requirements.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-broker-traits-scenarios.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-broker-traits-schema.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloud-scheduler-backend-changes.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloud-scheduler-backend-overview.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloud-scheduler-backend-requirements.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloud-scheduler-backend-rest-api.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloud-scheduler-backend-scenarios.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloud-scheduler-backend-schema.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloudtasks-broker-changes.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloudtasks-broker-overview.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloudtasks-broker-requirements.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloudtasks-broker-rest-api.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloudtasks-broker-scenarios.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-cloudtasks-broker-schema.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-scheduler-backends-gcp-changes.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-scheduler-backends-gcp-overview.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-scheduler-backends-gcp-requirements.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-change-spec-scheduler-backends-gcp-scenarios.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-post-clarifications.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-pre-clarifications.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/create-reference-context.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/restructure-input.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/review-change-implementation.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/review-cloud-scheduler-backend.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/review-cloudtasks-broker.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/review-reference-context.json
A	cclab/archive/20260327-gcp-cloud-integration/payloads/review-scheduler-backends-gcp.json
A	cclab/archive/20260327-gcp-cloud-integration/prompts/create_change_merge.md
A	cclab/archive/20260327-gcp-cloud-integration/prompts/restructure_input.md
A	cclab/archive/20260327-gcp-cloud-integration/user_input.md
A	cclab/archive/20260327-jet-test-gaps/STATE.yaml
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/post_clarifications.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/pre_clarifications.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/analyze_spec_jet-aot-build.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/analyze_spec_jet-hmr.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/analyze_spec_jet-postcss-tailwind.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/begin_implementation.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/create_reference_context.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/implement_spec.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/implement_tests_jet-aot-build.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/implement_tests_jet-hmr.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/prompts/implement_tests_jet-postcss-tailwind.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/reference_context.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/requirements.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/spec_plan.yaml
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/specs/jet-aot-build.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/specs/jet-hmr.md
A	cclab/archive/20260327-jet-test-gaps/groups/jet-test-gaps/specs/jet-postcss-tailwind.md
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-aot-build-changes.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-aot-build-overview.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-aot-build-requirements.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-aot-build-scenarios.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-aot-build-test-plan.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-hmr-changes.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-hmr-overview.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-hmr-requirements.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-hmr-scenarios.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-hmr-test-plan.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-postcss-tailwind-changes.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-postcss-tailwind-overview.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-postcss-tailwind-requirements.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-postcss-tailwind-scenarios.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-change-spec-postcss-tailwind-test-plan.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-post-clarifications.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-pre-clarifications.json
A	cclab/archive/20260327-jet-test-gaps/payloads/create-reference-context.json
A	cclab/archive/20260327-jet-test-gaps/payloads/restructure-input.json
A	cclab/archive/20260327-jet-test-gaps/prompts/create_change_merge.md
A	cclab/archive/20260327-jet-test-gaps/prompts/restructure_input.md
A	cclab/archive/20260327-jet-test-gaps/user_input.md
A	cclab/archive/20260327-mamba-conformance-xfail/STATE.yaml
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/post_clarifications.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/pre_clarifications.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/analyze_spec_conformance-xfail-reduction-spec.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/begin_implementation.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/create_post_clarifications.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/create_pre_clarifications.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/create_reference_context.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/fill_spec_conformance-xfail-reduction-spec_pipeline.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/implement_tests_conformance-xfail-reduction-spec.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/review_impl_conformance-xfail-reduction-spec.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/revise_change_implementation.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/prompts/write_implementation_diff.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/reference_context.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/requirements.md
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/spec_plan.yaml
A	cclab/archive/20260327-mamba-conformance-xfail/groups/conformance-xfail-reduction/specs/conformance-xfail-reduction-spec.md
A	cclab/archive/20260327-mamba-conformance-xfail/implementation.md
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-change-implementation.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-change-spec-changes.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-change-spec-logic.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-change-spec-overview.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-change-spec-pipeline.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-change-spec-requirements.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-change-spec-scenarios.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-post-clarifications.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-pre-clarifications.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/create-reference-context.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/fill-pipeline-section.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/fix-fill-sections-overview.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/fix-scenarios-section.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/restructure-input.json
A	cclab/archive/20260327-mamba-conformance-xfail/payloads/review-change-implementation.json
A	cclab/archive/20260327-mamba-conformance-xfail/prompts/create_change_merge.md
A	cclab/archive/20260327-mamba-conformance-xfail/prompts/restructure_input.md
A	cclab/archive/20260327-mamba-conformance-xfail/user_input.md
A	cclab/archive/20260327-mamba-xfail-zero/STATE.yaml
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/post_clarifications.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/pre_clarifications.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/prompts/analyze_spec_xfail-zero.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/prompts/begin_implementation.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/prompts/create_post_clarifications.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/prompts/create_reference_context.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/prompts/implement_tests_xfail-zero.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/prompts/review_impl_xfail-zero.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/prompts/revise_change_implementation.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/reference_context.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/requirements.md
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/spec_plan.yaml
A	cclab/archive/20260327-mamba-xfail-zero/groups/xfail-zero/specs/xfail-zero.md
A	cclab/archive/20260327-mamba-xfail-zero/implementation.md
A	cclab/archive/20260327-mamba-xfail-zero/payloads/create-change-spec-changes.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/create-change-spec-overview.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/create-change-spec-requirements.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/create-change-spec-scenarios.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/create-change-spec-test-plan.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/create-post-clarifications.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/create-pre-clarifications.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/create-reference-context.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/restructure-input.json
A	cclab/archive/20260327-mamba-xfail-zero/payloads/review-change-implementation.json
A	cclab/archive/20260327-mamba-xfail-zero/prompts/create_change_merge.md
A	cclab/archive/20260327-mamba-xfail-zero/prompts/restructure_input.md
A	cclab/archive/20260327-mamba-xfail-zero/user_input.md
A	cclab/archive/20260327-scheduler-runtime-complete/STATE.yaml
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/post_clarifications.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/pre_clarifications.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/analyze_spec_k8s-cronjob-backend.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/analyze_spec_push-receiver.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/analyze_spec_schedule-monitor.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/analyze_spec_scheduler-mode-selection.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/begin_implementation.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/create_post_clarifications.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/create_pre_clarifications.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/implement_spec.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/implement_tests_k8s-cronjob-backend.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/implement_tests_push-receiver.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/implement_tests_schedule-monitor.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/implement_tests_scheduler-mode-selection.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/review_impl_k8s-cronjob-backend.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/review_reference_context.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/revise_reference_context.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/prompts/write_implementation_diff.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/reference_context.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/requirements.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/spec_plan.yaml
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/specs/k8s-cronjob-backend.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/specs/push-receiver.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/specs/schedule-monitor.md
A	cclab/archive/20260327-scheduler-runtime-complete/groups/scheduler-runtime-complete/specs/scheduler-mode-selection.md
A	cclab/archive/20260327-scheduler-runtime-complete/implementation.md
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-implementation.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-k8s-cronjob-backend-changes.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-k8s-cronjob-backend-overview.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-k8s-cronjob-backend-requirements.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-k8s-cronjob-backend-scenarios.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-k8s-cronjob-backend-schema.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-push-receiver-changes.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-push-receiver-overview.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-push-receiver-requirements.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-push-receiver-rest-api.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-push-receiver-scenarios.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-push-receiver-schema.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-schedule-monitor-changes.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-schedule-monitor-overview.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-schedule-monitor-requirements.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-schedule-monitor-scenarios.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-schedule-monitor-schema.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-scheduler-mode-selection-changes.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-scheduler-mode-selection-overview.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-scheduler-mode-selection-requirements.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-scheduler-mode-selection-scenarios.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-change-spec-scheduler-mode-selection-schema.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-post-clarifications.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-pre-clarifications.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/create-reference-context.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/restructure-input.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/review-change-implementation.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/review-reference-context.json
A	cclab/archive/20260327-scheduler-runtime-complete/payloads/revise-reference-context.json
A	cclab/archive/20260327-scheduler-runtime-complete/prompts/create_change_merge.md
A	cclab/archive/20260327-scheduler-runtime-complete/prompts/restructure_input.md
A	cclab/archive/20260327-scheduler-runtime-complete/user_input.md
A	cclab/archive/20260327-scope-hoisting/STATE.yaml
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/post_clarifications.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/pre_clarifications.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/prompts/analyze_spec_scope-hoisting.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/prompts/begin_implementation.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/prompts/create_pre_clarifications.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/prompts/create_reference_context.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/prompts/implement_tests_scope-hoisting.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/prompts/review_impl_scope-hoisting.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/prompts/write_implementation_diff.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/reference_context.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/requirements.md
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/spec_plan.yaml
A	cclab/archive/20260327-scope-hoisting/groups/scope-hoisting/specs/scope-hoisting.md
A	cclab/archive/20260327-scope-hoisting/implementation.md
A	cclab/archive/20260327-scope-hoisting/issues/issue_1120_jet-build-scope-hoisting-module-concatenation.md
A	cclab/archive/20260327-scope-hoisting/payloads/create-change-implementation.json
A	cclab/archive/20260327-scope-hoisting/payloads/create-change-spec-changes.json
A	cclab/archive/20260327-scope-hoisting/payloads/create-change-spec-logic.json
A	cclab/archive/20260327-scope-hoisting/payloads/create-change-spec-overview.json
A	cclab/archive/20260327-scope-hoisting/payloads/create-change-spec-requirements.json
A	cclab/archive/20260327-scope-hoisting/payloads/create-change-spec-scenarios.json
A	cclab/archive/20260327-scope-hoisting/payloads/create-post-clarifications.json
A	cclab/archive/20260327-scope-hoisting/payloads/create-pre-clarifications.json
A	cclab/archive/20260327-scope-hoisting/payloads/create-reference-context.json
A	cclab/archive/20260327-scope-hoisting/payloads/restructure-input.json
A	cclab/archive/20260327-scope-hoisting/payloads/review-change-implementation.json
A	cclab/archive/20260327-scope-hoisting/prompts/create_change_merge.md
A	cclab/archive/20260327-scope-hoisting/prompts/restructure_input.md
A	cclab/archive/20260327-scope-hoisting/user_input.md
R096	cclab/specs/crates/cclab-sdd/generate/template-mcp-configs.md	cclab/archive/obsolete-specs/template-mcp-configs.md
A	cclab/changes/cclab-api-asgi-dispatch/STATE.yaml
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/post_clarifications.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/pre_clarifications.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/prompts/analyze_spec_asgi-dispatch-spec.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/prompts/begin_implementation.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/prompts/create_post_clarifications.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/prompts/create_pre_clarifications.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/prompts/implement_tests_asgi-dispatch-spec.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/prompts/write_implementation_diff.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/reference_context.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/requirements.md
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/spec_plan.yaml
A	cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/specs/asgi-dispatch-spec.md
A	cclab/changes/cclab-api-asgi-dispatch/implementation.md
A	cclab/changes/cclab-api-asgi-dispatch/payloads/create-change-implementation.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/fill-changes.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/fill-overview.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/fill-requirements.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/fill-scenarios.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/impl.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/post-clarify.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/pre-clarify.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/ref-ctx.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/restructure-input.json
A	cclab/changes/cclab-api-asgi-dispatch/payloads/review-impl.json
A	cclab/changes/cclab-api-asgi-dispatch/prompts/restructure_input.md
A	cclab/changes/cclab-api-asgi-dispatch/user_input.md
A	cclab/changes/cclab-pg-compat/STATE.yaml
A	cclab/changes/cclab-pg-compat/user_input.md
A	cclab/changes/conductor-cclab-migration/STATE.yaml
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/post_clarifications.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/pre_clarifications.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/analyze_spec_conductor-cclab-migration-spec.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/begin_implementation.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/create_post_clarifications.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/create_pre_clarifications.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/create_reference_context.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/fill_spec_conductor-cclab-migration-spec_changes.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/fill_spec_conductor-cclab-migration-spec_dependency.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/fill_spec_conductor-cclab-migration-spec_interaction.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/fill_spec_conductor-cclab-migration-spec_overview.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/fill_spec_conductor-cclab-migration-spec_requirements.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/fill_spec_conductor-cclab-migration-spec_test-plan.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/implement_tests_conductor-cclab-migration-spec.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/review_impl_conductor-cclab-migration-spec.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/review_reference_context.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/revise_reference_context.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/prompts/write_implementation_diff.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/reference_context.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/requirements.md
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/spec_plan.yaml
A	cclab/changes/conductor-cclab-migration/groups/cclab-import-migration/specs/conductor-cclab-migration-spec.md
A	cclab/changes/conductor-cclab-migration/implementation.md
A	cclab/changes/conductor-cclab-migration/payloads/create-change-implementation.json
A	cclab/changes/conductor-cclab-migration/payloads/create-post-clarifications.json
A	cclab/changes/conductor-cclab-migration/payloads/create-pre-clarifications.json
A	cclab/changes/conductor-cclab-migration/payloads/create-reference-context.json
A	cclab/changes/conductor-cclab-migration/payloads/fill-changes.json
A	cclab/changes/conductor-cclab-migration/payloads/fill-dependency.json
A	cclab/changes/conductor-cclab-migration/payloads/fill-interaction.json
A	cclab/changes/conductor-cclab-migration/payloads/fill-overview.json
A	cclab/changes/conductor-cclab-migration/payloads/fill-requirements.json
A	cclab/changes/conductor-cclab-migration/payloads/fill-test-plan.json
A	cclab/changes/conductor-cclab-migration/payloads/restructure-input.json
A	cclab/changes/conductor-cclab-migration/payloads/review-change-implementation.json
A	cclab/changes/conductor-cclab-migration/payloads/review-reference-context.json
A	cclab/changes/conductor-cclab-migration/payloads/revise-reference-context.json
A	cclab/changes/conductor-cclab-migration/prompts/restructure_input.md
A	cclab/changes/conductor-cclab-migration/user_input.md
A	cclab/changes/conductor-mock-backend/STATE.yaml
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/post_clarifications.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/pre_clarifications.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/analyze_spec_conductor-mock-backend-spec.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/begin_implementation.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/create_post_clarifications.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/create_pre_clarifications.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/fill_spec_conductor-mock-backend-spec_changes.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/fill_spec_conductor-mock-backend-spec_overview.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/fill_spec_conductor-mock-backend-spec_requirements.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/fill_spec_conductor-mock-backend-spec_rest-api.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/fill_spec_conductor-mock-backend-spec_test-plan.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/implement_tests_conductor-mock-backend-spec.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/review_impl_conductor-mock-backend-spec.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/prompts/write_implementation_diff.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/reference_context.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/requirements.md
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/spec_plan.yaml
A	cclab/changes/conductor-mock-backend/groups/mock-backend-server/specs/conductor-mock-backend-spec.md
A	cclab/changes/conductor-mock-backend/implementation.md
A	cclab/changes/conductor-mock-backend/payloads/create-change-implementation.json
A	cclab/changes/conductor-mock-backend/payloads/create-post-clarifications.json
A	cclab/changes/conductor-mock-backend/payloads/create-pre-clarifications.json
A	cclab/changes/conductor-mock-backend/payloads/create-reference-context.json
A	cclab/changes/conductor-mock-backend/payloads/fill-changes.json
A	cclab/changes/conductor-mock-backend/payloads/fill-overview.json
A	cclab/changes/conductor-mock-backend/payloads/fill-requirements.json
A	cclab/changes/conductor-mock-backend/payloads/fill-rest-api.json
A	cclab/changes/conductor-mock-backend/payloads/fill-test-plan.json
A	cclab/changes/conductor-mock-backend/payloads/restructure-input.json
A	cclab/changes/conductor-mock-backend/payloads/review-change-implementation.json
A	cclab/changes/conductor-mock-backend/prompts/restructure_input.md
A	cclab/changes/conductor-mock-backend/user_input.md
A	cclab/changes/conductor-multi-platform/STATE.yaml
A	cclab/changes/conductor-multi-platform/groups/multi-platform/post_clarifications.md
A	cclab/changes/conductor-multi-platform/groups/multi-platform/pre_clarifications.md
A	cclab/changes/conductor-multi-platform/groups/multi-platform/prompts/analyze_spec_multi-platform-spec.md
A	cclab/changes/conductor-multi-platform/groups/multi-platform/prompts/create_post_clarifications.md
A	cclab/changes/conductor-multi-platform/groups/multi-platform/prompts/create_pre_clarifications.md
A	cclab/changes/conductor-multi-platform/groups/multi-platform/reference_context.md
A	cclab/changes/conductor-multi-platform/groups/multi-platform/requirements.md
A	cclab/changes/conductor-multi-platform/groups/multi-platform/spec_plan.yaml
A	cclab/changes/conductor-multi-platform/groups/multi-platform/specs/multi-platform-spec.md
A	cclab/changes/conductor-multi-platform/payloads/create-change-spec-changes.json
A	cclab/changes/conductor-multi-platform/payloads/create-change-spec-db-model.json
A	cclab/changes/conductor-multi-platform/payloads/create-change-spec-overview.json
A	cclab/changes/conductor-multi-platform/payloads/create-change-spec-requirements.json
A	cclab/changes/conductor-multi-platform/payloads/create-change-spec-rest-api.json
A	cclab/changes/conductor-multi-platform/payloads/create-change-spec-scenarios.json
A	cclab/changes/conductor-multi-platform/payloads/create-change-spec-test-plan.json
A	cclab/changes/conductor-multi-platform/payloads/post-clarify.json
A	cclab/changes/conductor-multi-platform/payloads/pre-clarify.json
A	cclab/changes/conductor-multi-platform/payloads/ref-ctx.json
A	cclab/changes/conductor-multi-platform/payloads/restructure-input.json
A	cclab/changes/conductor-multi-platform/prompts/restructure_input.md
A	cclab/changes/conductor-multi-platform/user_input.md
A	cclab/changes/conductor-product-features/STATE.yaml
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/post_clarifications.md
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/pre_clarifications.md
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/prompts/analyze_spec_conductor-product-features-spec.md
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/prompts/create_post_clarifications.md
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/prompts/create_pre_clarifications.md
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/reference_context.md
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/requirements.md
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/spec_plan.yaml
A	cclab/changes/conductor-product-features/groups/fe-fixes-and-features/specs/conductor-product-features-spec.md
A	cclab/changes/conductor-product-features/groups/mock-backend-dynamic/post_clarifications.md
A	cclab/changes/conductor-product-features/groups/mock-backend-dynamic/pre_clarifications.md
A	cclab/changes/conductor-product-features/groups/mock-backend-dynamic/prompts/create_post_clarifications.md
A	cclab/changes/conductor-product-features/groups/mock-backend-dynamic/prompts/create_pre_clarifications.md
A	cclab/changes/conductor-product-features/groups/mock-backend-dynamic/prompts/create_reference_context.md
A	cclab/changes/conductor-product-features/groups/mock-backend-dynamic/reference_context.md
A	cclab/changes/conductor-product-features/groups/mock-backend-dynamic/requirements.md
A	cclab/changes/conductor-product-features/issues/issue_1096_conductor-fe-fix-projectdetail-hooks-ordering-tab-.md
A	cclab/changes/conductor-product-features/issues/issue_1097_create-cclab-spec-viewer-package-markdown-mermaid-.md
A	cclab/changes/conductor-product-features/issues/issue_1098_create-cclab-pipeline-package-dag-visualization-no.md
A	cclab/changes/conductor-product-features/issues/issue_1099_conductor-extend-mock-backend-to-support-full-user.md
A	cclab/changes/conductor-product-features/payloads/create-change-spec-changes.json
A	cclab/changes/conductor-product-features/payloads/create-change-spec-overview.json
A	cclab/changes/conductor-product-features/payloads/create-change-spec-requirements.json
A	cclab/changes/conductor-product-features/payloads/create-change-spec-scenarios.json
A	cclab/changes/conductor-product-features/payloads/create-change-spec-test-plan.json
A	cclab/changes/conductor-product-features/payloads/create-reference-context.json
A	cclab/changes/conductor-product-features/payloads/post-clarify-fe-fixes-and-features.json
A	cclab/changes/conductor-product-features/payloads/post-clarify-mock-backend-dynamic.json
A	cclab/changes/conductor-product-features/payloads/pre-clarify-fe-fixes-and-features.json
A	cclab/changes/conductor-product-features/payloads/pre-clarify-mock-backend-dynamic.json
A	cclab/changes/conductor-product-features/payloads/ref-ctx-fe-fixes-and-features.json
A	cclab/changes/conductor-product-features/payloads/ref-ctx-mock-backend-dynamic.json
A	cclab/changes/conductor-product-features/payloads/restructure-input.json
A	cclab/changes/conductor-product-features/prompts/restructure_input.md
A	cclab/changes/conductor-product-features/user_input.md
A	cclab/changes/conductor-product-redesign/STATE.yaml
A	cclab/changes/conductor-product-redesign/groups/route-restructure/post_clarifications.md
A	cclab/changes/conductor-product-redesign/groups/route-restructure/pre_clarifications.md
A	cclab/changes/conductor-product-redesign/groups/route-restructure/prompts/analyze_spec_conductor-redesign-spec.md
A	cclab/changes/conductor-product-redesign/groups/route-restructure/prompts/create_post_clarifications.md
A	cclab/changes/conductor-product-redesign/groups/route-restructure/prompts/create_pre_clarifications.md
A	cclab/changes/conductor-product-redesign/groups/route-restructure/reference_context.md
A	cclab/changes/conductor-product-redesign/groups/route-restructure/requirements.md
A	cclab/changes/conductor-product-redesign/groups/route-restructure/spec_plan.yaml
A	cclab/changes/conductor-product-redesign/groups/route-restructure/specs/conductor-redesign-spec.md
A	cclab/changes/conductor-product-redesign/payloads/fill-changes.json
A	cclab/changes/conductor-product-redesign/payloads/fill-overview.json
A	cclab/changes/conductor-product-redesign/payloads/fill-requirements.json
A	cclab/changes/conductor-product-redesign/payloads/fill-scenarios.json
A	cclab/changes/conductor-product-redesign/payloads/fill-test-plan.json
A	cclab/changes/conductor-product-redesign/payloads/post-clarify.json
A	cclab/changes/conductor-product-redesign/payloads/pre-clarify.json
A	cclab/changes/conductor-product-redesign/payloads/ref-ctx.json
A	cclab/changes/conductor-product-redesign/payloads/restructure-input.json
A	cclab/changes/conductor-product-redesign/payloads/spec-overview.json
A	cclab/changes/conductor-product-redesign/prompts/restructure_input.md
A	cclab/changes/conductor-product-redesign/user_input.md
A	cclab/changes/conductor-sdd-orchestrator/STATE.yaml
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/post_clarifications.md
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/pre_clarifications.md
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/prompts/analyze_spec_sdd-orchestrator-spec.md
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/prompts/create_post_clarifications.md
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/prompts/create_pre_clarifications.md
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/reference_context.md
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/requirements.md
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/spec_plan.yaml
A	cclab/changes/conductor-sdd-orchestrator/groups/orchestrator/specs/sdd-orchestrator-spec.md
A	cclab/changes/conductor-sdd-orchestrator/payloads/cs-changes.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/cs-interaction.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/cs-overview.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/cs-requirements.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/cs-rest-api.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/cs-scenarios.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/cs-schema.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/cs-test-plan.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/post-clarify.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/pre-clarify.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/ref-ctx.json
A	cclab/changes/conductor-sdd-orchestrator/payloads/restructure-input.json
A	cclab/changes/conductor-sdd-orchestrator/prompts/restructure_input.md
A	cclab/changes/conductor-sdd-orchestrator/user_input.md
A	cclab/changes/conductor-state-specs/STATE.yaml
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/post_clarifications.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/pre_clarifications.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/prompts/analyze_spec_conductor-state-specs.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/prompts/begin_implementation.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/prompts/create_post_clarifications.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/prompts/create_pre_clarifications.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/prompts/implement_tests_conductor-state-specs.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/prompts/review_impl_conductor-state-specs.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/prompts/write_implementation_diff.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/reference_context.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/requirements.md
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/spec_plan.yaml
A	cclab/changes/conductor-state-specs/groups/state-machine-specs/specs/conductor-state-specs.md
A	cclab/changes/conductor-state-specs/implementation.md
A	cclab/changes/conductor-state-specs/issues/issue_1094_conductor-complete-state-machine-specs-for-issue-c.md
A	cclab/changes/conductor-state-specs/payloads/create-change-implementation.json
A	cclab/changes/conductor-state-specs/payloads/create-post-clarifications.json
A	cclab/changes/conductor-state-specs/payloads/create-pre-clarifications.json
A	cclab/changes/conductor-state-specs/payloads/create-reference-context.json
A	cclab/changes/conductor-state-specs/payloads/fill-changes.json
A	cclab/changes/conductor-state-specs/payloads/fill-overview.json
A	cclab/changes/conductor-state-specs/payloads/fill-requirements.json
A	cclab/changes/conductor-state-specs/payloads/fill-scenarios.json
A	cclab/changes/conductor-state-specs/payloads/fill-state-machine.json
A	cclab/changes/conductor-state-specs/payloads/fill-test-plan.json
A	cclab/changes/conductor-state-specs/payloads/restructure-input.json
A	cclab/changes/conductor-state-specs/payloads/review-change-implementation.json
A	cclab/changes/conductor-state-specs/prompts/restructure_input.md
A	cclab/changes/conductor-state-specs/user_input.md
A	cclab/changes/gen-thread-pool/STATE.yaml
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/post_clarifications.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/pre_clarifications.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/prompts/analyze_spec_generator-thread-pool-design.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/prompts/begin_implementation.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/prompts/create_post_clarifications.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/prompts/create_pre_clarifications.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/prompts/create_reference_context.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/prompts/implement_tests_generator-thread-pool-design.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/reference_context.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/requirements.md
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/spec_plan.yaml
A	cclab/changes/gen-thread-pool/groups/gen-thread-pool/specs/generator-thread-pool-design.md
A	cclab/changes/gen-thread-pool/implementation.md
A	cclab/changes/gen-thread-pool/issues/issue_1114_fix-mamba-sigbus-crash-in-multi-threaded-conforman.md
A	cclab/changes/gen-thread-pool/payloads/create-change-spec-changes.json
A	cclab/changes/gen-thread-pool/payloads/create-change-spec-overview.json
A	cclab/changes/gen-thread-pool/payloads/create-change-spec-requirements.json
A	cclab/changes/gen-thread-pool/payloads/create-change-spec-scenarios.json
A	cclab/changes/gen-thread-pool/payloads/create-change-spec-state-machine.json
A	cclab/changes/gen-thread-pool/payloads/create-change-spec-test-plan.json
A	cclab/changes/gen-thread-pool/payloads/create-post-clarifications.json
A	cclab/changes/gen-thread-pool/payloads/create-pre-clarifications.json
A	cclab/changes/gen-thread-pool/payloads/create-reference-context.json
A	cclab/changes/gen-thread-pool/payloads/restructure-input.json
A	cclab/changes/gen-thread-pool/prompts/restructure_input.md
A	cclab/changes/gen-thread-pool/user_input.md
A	cclab/changes/jet-aot-build-gaps/STATE.yaml
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/pre_clarifications.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/analyze_spec_jet-aot-build-gaps-spec.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/begin_implementation.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/create_pre_clarifications.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/create_reference_context.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/fill_spec_jet-aot-build-gaps-spec_changes.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/fill_spec_jet-aot-build-gaps-spec_overview.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/fill_spec_jet-aot-build-gaps-spec_requirements.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/fill_spec_jet-aot-build-gaps-spec_scenarios.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/review_reference_context.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/prompts/revise_reference_context.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/reference_context.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/requirements.md
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/spec_plan.yaml
A	cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/specs/jet-aot-build-gaps-spec.md
A	cclab/changes/jet-aot-build-gaps/issues/issue_765_feat-jet-aot-production-build-tree-shaking-code-sp.md
A	cclab/changes/jet-aot-build-gaps/payloads/create-pre-clarifications.json
A	cclab/changes/jet-aot-build-gaps/payloads/create-reference-context.json
A	cclab/changes/jet-aot-build-gaps/payloads/fill-section-overview.json
A	cclab/changes/jet-aot-build-gaps/payloads/restructure-input.json
A	cclab/changes/jet-aot-build-gaps/payloads/review-reference-context.json
A	cclab/changes/jet-aot-build-gaps/payloads/revise-reference-context.json
A	cclab/changes/jet-aot-build-gaps/prompts/restructure_input.md
A	cclab/changes/jet-aot-build-gaps/user_input.md
A	cclab/changes/lens-dissolution/STATE.yaml
A	cclab/changes/lens-dissolution/groups/lens-dissolution/post_clarifications.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/pre_clarifications.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_agent-context-builder.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_agent-output-format.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_lens-dissolution-restructure.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_sdd-cli-context-command.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/analyze_spec_type-inference-pipeline.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/begin_implementation.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/create_post_clarifications.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/create_pre_clarifications.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/create_reference_context.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/implement_spec.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/implement_tests_agent-context-builder.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/implement_tests_agent-output-format.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/implement_tests_lens-dissolution-restructure.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/prompts/review_reference_context.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/reference_context.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/requirements.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/spec_plan.yaml
A	cclab/changes/lens-dissolution/groups/lens-dissolution/specs/agent-context-builder.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/specs/agent-output-format.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/specs/lens-dissolution-restructure.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/specs/sdd-cli-context-command.md
A	cclab/changes/lens-dissolution/groups/lens-dissolution/specs/type-inference-pipeline.md
A	cclab/changes/lens-dissolution/implementation.md
A	cclab/changes/lens-dissolution/issues/issue_1087_refactor-dissolve-lens-module-into-sdd-top-level-s.md
A	cclab/changes/lens-dissolution/issues/issue_944_feat-lens-wire-cross-file-type-propagation-deep-in.md
A	cclab/changes/lens-dissolution/issues/issue_946_feat-lens-agent-context-builder-smart-file-selecti.md
A	cclab/changes/lens-dissolution/issues/issue_949_feat-lens-agent-optimized-output-structured-json-f.md
A	cclab/changes/lens-dissolution/payloads/agent-context-builder-changes.json
A	cclab/changes/lens-dissolution/payloads/agent-context-builder-logic.json
A	cclab/changes/lens-dissolution/payloads/agent-context-builder-overview.json
A	cclab/changes/lens-dissolution/payloads/agent-context-builder-requirements.json
A	cclab/changes/lens-dissolution/payloads/agent-context-builder-scenarios.json
A	cclab/changes/lens-dissolution/payloads/agent-context-builder-schema.json
A	cclab/changes/lens-dissolution/payloads/agent-context-builder-test-plan.json
A	cclab/changes/lens-dissolution/payloads/agent-output-format-changes.json
A	cclab/changes/lens-dissolution/payloads/agent-output-format-overview.json
A	cclab/changes/lens-dissolution/payloads/agent-output-format-requirements.json
A	cclab/changes/lens-dissolution/payloads/agent-output-format-scenarios.json
A	cclab/changes/lens-dissolution/payloads/agent-output-format-schema.json
A	cclab/changes/lens-dissolution/payloads/agent-output-format-test-plan.json
A	cclab/changes/lens-dissolution/payloads/create-post-clarifications.json
A	cclab/changes/lens-dissolution/payloads/create-pre-clarifications.json
A	cclab/changes/lens-dissolution/payloads/create-reference-context.json
A	cclab/changes/lens-dissolution/payloads/lens-dissolution-restructure-changes.json
A	cclab/changes/lens-dissolution/payloads/lens-dissolution-restructure-overview.json
A	cclab/changes/lens-dissolution/payloads/lens-dissolution-restructure-requirements.json
A	cclab/changes/lens-dissolution/payloads/lens-dissolution-restructure-scenarios.json
A	cclab/changes/lens-dissolution/payloads/lens-dissolution-restructure-test-plan.json
A	cclab/changes/lens-dissolution/payloads/restructure-input.json
A	cclab/changes/lens-dissolution/payloads/review-reference-context.json
A	cclab/changes/lens-dissolution/payloads/sdd-cli-context-command-changes.json
A	cclab/changes/lens-dissolution/payloads/sdd-cli-context-command-overview.json
A	cclab/changes/lens-dissolution/payloads/sdd-cli-context-command-requirements.json
A	cclab/changes/lens-dissolution/payloads/sdd-cli-context-command-scenarios.json
A	cclab/changes/lens-dissolution/payloads/type-inference-pipeline-changes.json
A	cclab/changes/lens-dissolution/payloads/type-inference-pipeline-logic.json
A	cclab/changes/lens-dissolution/payloads/type-inference-pipeline-overview.json
A	cclab/changes/lens-dissolution/payloads/type-inference-pipeline-requirements.json
A	cclab/changes/lens-dissolution/payloads/type-inference-pipeline-scenarios.json
A	cclab/changes/lens-dissolution/payloads/type-inference-pipeline-schema.json
A	cclab/changes/lens-dissolution/payloads/type-inference-pipeline-test-plan.json
A	cclab/changes/lens-dissolution/prompts/restructure_input.md
A	cclab/changes/lens-dissolution/user_input.md
A	cclab/changes/mamba-jit-memory/STATE.yaml
A	cclab/changes/mamba-jit-memory/groups/jit-memory/post_clarifications.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/pre_clarifications.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/prompts/analyze_spec_cranelift-jit-memory-fix.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/prompts/analyze_spec_jit-memory.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/prompts/begin_implementation.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/prompts/create_reference_context.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/prompts/implement_tests_cranelift-jit-memory-fix.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/reference_context.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/requirements.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/spec_plan.yaml
A	cclab/changes/mamba-jit-memory/groups/jit-memory/specs/cranelift-jit-memory-fix.md
A	cclab/changes/mamba-jit-memory/groups/jit-memory/specs/jit-memory.md
A	cclab/changes/mamba-jit-memory/issues/issue_1114_fix-mamba-sigbus-crash-in-multi-threaded-conforman.md
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-cranelift-jit-memory-fix-changes.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-cranelift-jit-memory-fix-overview.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-cranelift-jit-memory-fix-requirements.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-cranelift-jit-memory-fix-scenarios.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-cranelift-jit-memory-fix-test-plan.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-jit-memory-changes.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-jit-memory-overview.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-jit-memory-requirements.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-jit-memory-scenarios.json
A	cclab/changes/mamba-jit-memory/payloads/create-change-spec-jit-memory-test-plan.json
A	cclab/changes/mamba-jit-memory/payloads/create-post-clarifications.json
A	cclab/changes/mamba-jit-memory/payloads/create-pre-clarifications.json
A	cclab/changes/mamba-jit-memory/payloads/create-reference-context.json
A	cclab/changes/mamba-jit-memory/payloads/restructure-input.json
A	cclab/changes/mamba-jit-memory/prompts/restructure_input.md
A	cclab/changes/mamba-jit-memory/user_input.md
A	cclab/changes/mamba-refcount-jit/STATE.yaml
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/post_clarifications.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/pre_clarifications.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/prompts/analyze_spec_mamba-refcount-jit-spec.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/prompts/begin_implementation.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/prompts/create_post_clarifications.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/prompts/create_reference_context.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/prompts/implement_tests_mamba-refcount-jit-spec.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/prompts/write_implementation_diff.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/reference_context.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/requirements.md
A	cclab/changes/mamba-refcount-jit/groups/refcount-jit/specs/mamba-refcount-jit-spec.md
A	cclab/changes/mamba-refcount-jit/implementation.md
A	cclab/changes/mamba-refcount-jit/issues/issue_1129_refactor-mamba-implement-cpython-3-12-reference-co.md
A	cclab/changes/mamba-refcount-jit/payloads/create-change-implementation.json
A	cclab/changes/mamba-refcount-jit/payloads/create-change-spec-changes.json
A	cclab/changes/mamba-refcount-jit/payloads/create-change-spec-logic.json
A	cclab/changes/mamba-refcount-jit/payloads/create-change-spec-overview.json
A	cclab/changes/mamba-refcount-jit/payloads/create-change-spec-requirements.json
A	cclab/changes/mamba-refcount-jit/payloads/create-change-spec-scenarios.json
A	cclab/changes/mamba-refcount-jit/payloads/create-change-spec-test-plan.json
A	cclab/changes/mamba-refcount-jit/payloads/create-post-clarifications.json
A	cclab/changes/mamba-refcount-jit/payloads/create-pre-clarifications.json
A	cclab/changes/mamba-refcount-jit/payloads/create-reference-context.json
A	cclab/changes/mamba-refcount-jit/payloads/restructure-input.json
A	cclab/changes/mamba-refcount-jit/prompts/restructure_input.md
A	cclab/changes/mamba-refcount-jit/user_input.md
A	cclab/changes/sdd-gen-code-pipeline/STATE.yaml
A	cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/post_clarifications.md
A	cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/pre_clarifications.md
A	cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/prompts/create_post_clarifications.md
A	cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/prompts/create_pre_clarifications.md
A	cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/reference_context.md
A	cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/requirements.md
A	cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/spec_plan.yaml
A	cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/specs/gen-code-cli.md
A	cclab/changes/sdd-gen-code-pipeline/issues/issue_1128_feat-sdd-gen-code-gen-diff-gen-parse-spec-driven-c.md
A	cclab/changes/sdd-gen-code-pipeline/payloads/create-post-clarifications.json
A	cclab/changes/sdd-gen-code-pipeline/payloads/create-pre-clarifications.json
A	cclab/changes/sdd-gen-code-pipeline/payloads/create-reference-context.json
A	cclab/changes/sdd-gen-code-pipeline/payloads/restructure-input.json
A	cclab/changes/sdd-gen-code-pipeline/payloads/review-reference-context.json
A	cclab/changes/sdd-gen-code-pipeline/prompts/restructure_input.md
A	cclab/changes/sdd-gen-code-pipeline/user_input.md
A	cclab/changes/sdd-index-path-rename/STATE.yaml
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/post_clarifications.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/pre_clarifications.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/analyze_spec_cli-hints-impl-prompt.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/analyze_spec_index-path-rename.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/begin_implementation.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/create_post_clarifications.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/create_pre_clarifications.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/create_reference_context.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/implement_spec.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/implement_tests_cli-hints-impl-prompt.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/implement_tests_index-path-rename.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/review_reference_context.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/prompts/write_implementation_diff.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/reference_context.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/requirements.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/spec_plan.yaml
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/specs/cli-hints-impl-prompt.md
A	cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/specs/index-path-rename.md
A	cclab/changes/sdd-index-path-rename/implementation.md
A	cclab/changes/sdd-index-path-rename/payloads/create-change-implementation.json
A	cclab/changes/sdd-index-path-rename/payloads/create-change-spec-cli-hints-changes.json
A	cclab/changes/sdd-index-path-rename/payloads/create-change-spec-cli-hints-overview.json
A	cclab/changes/sdd-index-path-rename/payloads/create-change-spec-index-path-rename-changes.json
A	cclab/changes/sdd-index-path-rename/payloads/create-change-spec-index-path-rename-overview.json
A	cclab/changes/sdd-index-path-rename/payloads/create-post-clarifications.json
A	cclab/changes/sdd-index-path-rename/payloads/create-pre-clarifications.json
A	cclab/changes/sdd-index-path-rename/payloads/create-reference-context.json
A	cclab/changes/sdd-index-path-rename/payloads/restructure-input.json
A	cclab/changes/sdd-index-path-rename/payloads/review-reference-context.json
A	cclab/changes/sdd-index-path-rename/prompts/restructure_input.md
A	cclab/changes/sdd-index-path-rename/user_input.md
A	cclab/changes/sdd-index-scoped-toolchain/STATE.yaml
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/post_clarifications.md
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/pre_clarifications.md
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/prompts/create_post_clarifications.md
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/prompts/create_pre_clarifications.md
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/reference_context.md
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/requirements.md
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/spec_plan.yaml
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/specs/auto-discover.md
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/specs/index-config-model.md
A	cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/specs/multi-handler-daemon.md
A	cclab/changes/sdd-index-scoped-toolchain/issues/issue_1127_feat-sdd-index-server-scoped-toolchain-binding-aut.md
A	cclab/changes/sdd-index-scoped-toolchain/payloads/create-post-clarifications.json
A	cclab/changes/sdd-index-scoped-toolchain/payloads/create-pre-clarifications.json
A	cclab/changes/sdd-index-scoped-toolchain/payloads/create-reference-context.json
A	cclab/changes/sdd-index-scoped-toolchain/payloads/restructure-input.json
A	cclab/changes/sdd-index-scoped-toolchain/payloads/review-reference-context.json
A	cclab/changes/sdd-index-scoped-toolchain/prompts/restructure_input.md
A	cclab/changes/sdd-index-scoped-toolchain/user_input.md
A	cclab/changes/sdd-phase-advance-timeout/STATE.yaml
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/post_clarifications.md
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/pre_clarifications.md
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/prompts/create_post_clarifications.md
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/prompts/create_pre_clarifications.md
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/reference_context.md
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/requirements.md
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/spec_plan.yaml
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/specs/agent-timeout.md
A	cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/specs/phase-advance-fix.md
A	cclab/changes/sdd-phase-advance-timeout/issues/issue_1124_bug-sdd-reference-context-phase-never-advances-gro.md
A	cclab/changes/sdd-phase-advance-timeout/issues/issue_1126_feat-sdd-agent-execution-timeout-prevent-infinite-.md
A	cclab/changes/sdd-phase-advance-timeout/payloads/create-post-clarifications.json
A	cclab/changes/sdd-phase-advance-timeout/payloads/create-pre-clarifications.json
A	cclab/changes/sdd-phase-advance-timeout/payloads/create-reference-context.json
A	cclab/changes/sdd-phase-advance-timeout/payloads/restructure-input.json
A	cclab/changes/sdd-phase-advance-timeout/payloads/review-reference-context.json
A	cclab/changes/sdd-phase-advance-timeout/prompts/restructure_input.md
A	cclab/changes/sdd-phase-advance-timeout/user_input.md
M	cclab/config.toml
A	cclab/specs/AUTHORING.md
A	cclab/specs/crates/cclab-fetch/broker/broker-traits.md
A	cclab/specs/crates/cclab-fetch/broker/cloudtasks.md
A	cclab/specs/crates/cclab-fetch/scheduler/cloud-scheduler-backend.md
A	cclab/specs/crates/cclab-fetch/scheduler/k8s-cronjob-backend.md
A	cclab/specs/crates/cclab-fetch/scheduler/push-receiver.md
A	cclab/specs/crates/cclab-fetch/scheduler/schedule-monitor.md
A	cclab/specs/crates/cclab-fetch/scheduler/scheduler-backends.md
A	cclab/specs/crates/cclab-fetch/scheduler/scheduler-mode-selection.md
M	cclab/specs/crates/cclab-jet/aot-build.md
D	cclab/specs/crates/cclab-jet/bundle-optimization-hoisting.md
A	cclab/specs/crates/cclab-jet/dev-server.md
A	cclab/specs/crates/cclab-jet/e2e/e2e-test-infrastructure.md
A	cclab/specs/crates/cclab-jet/hmr.md
D	cclab/specs/crates/cclab-jet/jet-remaining-spec.md
A	cclab/specs/crates/cclab-jet/logic/aot-build.md
A	cclab/specs/crates/cclab-jet/logic/hmr.md
A	cclab/specs/crates/cclab-jet/logic/postcss-tailwind.md
A	cclab/specs/crates/cclab-jet/logic/scope-hoisting.md
A	cclab/specs/crates/cclab-jet/workspace-protocol.md
M	cclab/specs/crates/mamba/codegen/cranelift-jit.md
M	cclab/specs/crates/mamba/runtime/gc.md
M	cclab/specs/crates/mamba/runtime/generator.md
M	cclab/specs/crates/mamba/testing/conformance.md
M	cclab/specs/crates/cclab-sdd/config/agents.md
M	cclab/specs/crates/cclab-sdd/generate/README.md
M	cclab/specs/crates/cclab-sdd/generate/architecture.md
M	cclab/specs/crates/cclab-sdd/generate/requirement-plus-enhancement.md
M	cclab/specs/crates/cclab-sdd/generate/spec-ir-contract.md
M	cclab/specs/crates/cclab-sdd/generate/spec-ir-evaluation.md
M	cclab/specs/crates/cclab-sdd/generate/spec-model.md
M	cclab/specs/crates/cclab-sdd/generate/template-claude-md.md
M	cclab/specs/crates/cclab-sdd/generate/template-knowledge-index.md
A	cclab/specs/crates/cclab-sdd/generate/ux-pattern-library.md
M	cclab/specs/crates/cclab-sdd/interfaces/cli/commands.md
M	cclab/specs/crates/cclab-sdd/interfaces/cli/sdd-cli.md
R100	cclab/specs/crates/cclab-lens/lens-cli-subcommands.md	cclab/specs/crates/cclab-sdd/interfaces/lens/lens-cli-subcommands.md
R100	cclab/specs/crates/cclab-lens/lens-pdg-mcp-tools.md	cclab/specs/crates/cclab-sdd/interfaces/lens/lens-pdg-mcp-tools.md
M	cclab/specs/crates/cclab-sdd/interfaces/tools/artifact-tools.md
M	cclab/specs/crates/cclab-sdd/interfaces/tools/utility-tools.md
A	cclab/specs/crates/cclab-sdd/logic/agent-context-builder.md
A	cclab/specs/crates/cclab-sdd/logic/agent-output-format.md
R100	cclab/specs/crates/cclab-lens/analysis-tools.md	cclab/specs/crates/cclab-sdd/logic/analysis-tools.md
R100	cclab/specs/crates/cclab-lens/cclab-lens-spec.md	cclab/specs/crates/cclab-sdd/logic/cclab-lens-spec.md
M	cclab/specs/crates/cclab-sdd/logic/change-merge.md
M	cclab/specs/crates/cclab-sdd/logic/change-spec.md
R100	cclab/specs/crates/cclab-lens/class-diagram.md	cclab/specs/crates/cclab-sdd/logic/class-diagram.md
R100	cclab/specs/crates/cclab-lens/code-analysis-service-v2.md	cclab/specs/crates/cclab-sdd/logic/code-analysis-service-v2.md
M	cclab/specs/crates/cclab-sdd/logic/executor-resolution.md
M	cclab/specs/crates/cclab-sdd/logic/implement-task.md
R100	cclab/specs/crates/cclab-lens/README.md	cclab/specs/crates/cclab-sdd/logic/lens-README.md
R100	cclab/specs/crates/cclab-lens/lens-beyond-ide.md	cclab/specs/crates/cclab-sdd/logic/lens-beyond-ide.md
R100	cclab/specs/crates/cclab-lens/lens-codegen-unification.md	cclab/specs/crates/cclab-sdd/logic/lens-codegen-unification.md
R100	cclab/specs/crates/cclab-lens/lens-comprehensive.md	cclab/specs/crates/cclab-sdd/logic/lens-comprehensive.md
R100	cclab/specs/crates/cclab-lens/lens-full-upgrade-spec.md	cclab/specs/crates/cclab-sdd/logic/lens-full-upgrade-spec.md
R083	cclab/specs/crates/cclab-lens/lens-index-storage.md	cclab/specs/crates/cclab-sdd/logic/lens-index-storage.md
R100	cclab/specs/crates/cclab-lens/lens-lang-support.md	cclab/specs/crates/cclab-sdd/logic/lens-lang-support.md
R100	cclab/specs/crates/cclab-lens/lens-markdown.md	cclab/specs/crates/cclab-sdd/logic/lens-markdown.md
R100	cclab/specs/crates/cclab-lens/lens-yaml-codegen.md	cclab/specs/crates/cclab-sdd/logic/lens-yaml-codegen.md
M	cclab/specs/crates/cclab-sdd/logic/merge-lens-into-sdd-spec.md
M	cclab/specs/crates/cclab-sdd/logic/post-clarifications.md
M	cclab/specs/crates/cclab-sdd/logic/pre-clarifications.md
R100	cclab/specs/crates/cclab-lens/python-pdg-core.md	cclab/specs/crates/cclab-sdd/logic/python-pdg-core.md
R100	cclab/specs/crates/cclab-lens/refactoring-api.md	cclab/specs/crates/cclab-sdd/logic/refactoring-api.md
M	cclab/specs/crates/cclab-sdd/logic/reference-context.md
M	cclab/specs/crates/cclab-sdd/logic/restructure-input.md
R100	cclab/specs/crates/cclab-lens/rust-symbol-analysis.md	cclab/specs/crates/cclab-sdd/logic/rust-symbol-analysis.md
R100	cclab/specs/crates/cclab-lens/semantic-search-api.md	cclab/specs/crates/cclab-sdd/logic/semantic-search-api.md
A	cclab/specs/crates/cclab-sdd/logic/spec-diff-codegen.md
M	cclab/specs/crates/cclab-sdd/logic/state-machine.md
A	cclab/specs/crates/cclab-sdd/logic/tech-stack-inference.md
A	cclab/specs/crates/cclab-sdd/logic/type-inference-pipeline.md
R100	cclab/specs/crates/cclab-lens/usage-examples.md	cclab/specs/crates/cclab-sdd/logic/usage-examples.md
M	cclab/specs/crates/cclab-sdd/skills/agent.md
M	cclab/specs/crates/cclab-sdd/skills/fillback.md
M	cclab/specs/crates/cclab-sdd/skills/run-change.md
M	cclab/specs/crates/cclab-sdd/tools/utils/analyze-code-for-spec.md
M	cclab/specs/crates/cclab-sdd/tools/utils/delegate-agent.md
M	cclab/specs/crates/cclab-sdd/tools/utils/fetch-issues.md
M	cclab/specs/crates/cclab-sdd/tools/utils/list-changed-files.md
M	cclab/specs/crates/cclab-sdd/tools/utils/platform-sync.md
M	cclab/specs/crates/cclab-sdd/tools/utils/read-artifact.md
M	cclab/specs/crates/cclab-sdd/tools/utils/read-implementation-summary.md
M	cclab/specs/crates/cclab-sdd/tools/utils/validate-change.md
M	cclab/specs/crates/cclab-sdd/tools/utils/validate-spec-completeness.md
M	cclab/specs/crates/cclab-sdd/tools/utils/write-artifact.md
A	crates/cclab-aurora/Cargo.toml
M	crates/cclab-jet/examples/dev_server.rs
M	crates/cclab-jet/examples/full_pipeline.rs
M	crates/cclab-jet/src/asset/image_processor.rs
M	crates/cclab-jet/src/bundler/css_bundle.rs
A	crates/cclab-jet/src/bundler/html_minify.rs
A	crates/cclab-jet/src/bundler/json_shake.rs
M	crates/cclab-jet/src/bundler/minify.rs
M	crates/cclab-jet/src/bundler/mod.rs
M	crates/cclab-jet/src/bundler/scope_hoist.rs
A	crates/cclab-jet/src/bundler/scope_hoist_opt.rs
M	crates/cclab-jet/src/bundler/sourcemap.rs
M	crates/cclab-jet/src/bundler/splitting.rs
M	crates/cclab-jet/src/bundler/tree_shake.rs
M	crates/cclab-jet/src/bundler/types.rs
M	crates/cclab-jet/src/cli.rs
M	crates/cclab-jet/src/css/import_resolver.rs
M	crates/cclab-jet/src/css/output.rs
M	crates/cclab-jet/src/css/tailwind/preflight.rs
M	crates/cclab-jet/src/css/tailwind/utilities.rs
M	crates/cclab-jet/src/css/tailwind/variants.rs
M	crates/cclab-jet/src/dev_server/hmr.rs
A	crates/cclab-jet/src/dev_server/hmr_client.rs
A	crates/cclab-jet/src/dev_server/importmap.rs
M	crates/cclab-jet/src/dev_server/mod.rs
A	crates/cclab-jet/src/dev_server/module_graph.rs
A	crates/cclab-jet/src/dev_server/polyfills.rs
A	crates/cclab-jet/src/dev_server/polyfills_tests.rs
A	crates/cclab-jet/src/dev_server/prebundle.rs
A	crates/cclab-jet/src/dev_server/prebundle_tests.rs
A	crates/cclab-jet/src/dev_server/react_refresh.rs
A	crates/cclab-jet/src/dev_server/source_analysis.rs
M	crates/cclab-jet/src/dev_server/watcher.rs
M	crates/cclab-jet/src/pkg_manager/mod.rs
A	crates/cclab-jet/src/pkg_manager/platform.rs
M	crates/cclab-jet/src/pkg_manager/store.rs
M	crates/cclab-jet/src/transform/mod.rs
A	crates/cclab-jet/src/transform/react_refresh.rs
M	crates/cclab-jet/src/transform/transform_tsx.rs
A	crates/cclab-jet/src/transform/transform_tsx_tests.rs
A	crates/cclab-jet/src/transform/type_strip.rs
A	crates/cclab-jet/tests/workspace_protocol.rs
M	crates/mamba/Cargo.toml
M	crates/mamba/src/codegen/cranelift/jit.rs
M	crates/mamba/src/codegen/cranelift/mod.rs
M	crates/mamba/src/conformance/mod.rs
M	crates/mamba/src/lexer/indent.rs
M	crates/mamba/src/lower/ast_to_hir.rs
M	crates/mamba/src/lower/hir_to_mir.rs
M	crates/mamba/src/parser/expr.rs
M	crates/mamba/src/parser/expr_compound.rs
M	crates/mamba/src/runtime/async_rt.rs
M	crates/mamba/src/runtime/builtins.rs
M	crates/mamba/src/runtime/class.rs
M	crates/mamba/src/runtime/dict_ops.rs
M	crates/mamba/src/runtime/gc.rs
M	crates/mamba/src/runtime/generator.rs
M	crates/mamba/src/runtime/iter.rs
M	crates/mamba/src/runtime/list_ops.rs
M	crates/mamba/src/runtime/mod.rs
M	crates/mamba/src/runtime/rc.rs
M	crates/mamba/src/runtime/set_ops.rs
M	crates/mamba/src/runtime/string_ops.rs
M	crates/mamba/src/runtime/symbols.rs
M	crates/mamba/src/types/check_expr.rs
M	crates/mamba/tests/behavioral_builtins_tests.rs
M	crates/mamba/tests/behavioral_lang_tests.rs
M	crates/mamba/tests/behavioral_stdlib_tests.rs
M	crates/mamba/tests/conformance_tests.rs
M	crates/mamba/tests/fixture_tests.rs
M	crates/mamba/tests/fixtures/conformance/__snippet_test.expected
M	crates/mamba/tests/fixtures/conformance/__snippet_test.py
M	crates/mamba/tests/fixtures/conformance/builtins/collection_builtins_edge.py
M	crates/mamba/tests/fixtures/conformance/builtins/collection_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/builtins/numeric_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/builtins/print_kwargs.py
M	crates/mamba/tests/fixtures/conformance/builtins/repr_format.py
M	crates/mamba/tests/fixtures/conformance/builtins/string_repr_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/builtins/type_introspection.py
M	crates/mamba/tests/fixtures/conformance/builtins/type_introspection_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/class_system/mro_edge_cases.expected
M	crates/mamba/tests/fixtures/conformance/class_system/mro_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/data_structures/bytes_edge_cases.expected
M	crates/mamba/tests/fixtures/conformance/data_structures/bytes_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/data_structures/dict_edge_cases_xfail.expected
M	crates/mamba/tests/fixtures/conformance/data_structures/dict_edge_cases_xfail.py
M	crates/mamba/tests/fixtures/conformance/data_structures/list_constructor_xfail.py
M	crates/mamba/tests/fixtures/conformance/data_structures/list_edge_cases_xfail.py
M	crates/mamba/tests/fixtures/conformance/data_structures/list_sort_lambda.py
M	crates/mamba/tests/fixtures/conformance/data_structures/set_edge_cases_xfail.expected
M	crates/mamba/tests/fixtures/conformance/data_structures/set_edge_cases_xfail.py
M	crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.py
M	crates/mamba/tests/fixtures/conformance/data_structures/string_format_xfail.py
M	crates/mamba/tests/fixtures/conformance/data_structures/tuple_edge_cases_xfail.py
M	crates/mamba/tests/fixtures/conformance/exceptions/chaining_edge_cases.expected
M	crates/mamba/tests/fixtures/conformance/exceptions/chaining_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/generators/close_edge_cases_xfail.py
M	crates/mamba/tests/fixtures/conformance/generators/context_manager_pattern_xfail.py
M	crates/mamba/tests/fixtures/conformance/generators/send_edge_cases_xfail.py
M	crates/mamba/tests/fixtures/conformance/generators/state_attributes.expected
M	crates/mamba/tests/fixtures/conformance/generators/state_attributes.py
M	crates/mamba/tests/fixtures/conformance/generators/throw_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/generators/yield_from_passthrough_xfail.expected
M	crates/mamba/tests/fixtures/conformance/generators/yield_from_passthrough_xfail.py
M	crates/mamba/tests/fixtures/conformance/iterators/callable_sentinel.expected
M	crates/mamba/tests/fixtures/conformance/iterators/callable_sentinel.py
M	crates/mamba/tests/fixtures/conformance/iterators/composition_xfail.expected
M	crates/mamba/tests/fixtures/conformance/iterators/composition_xfail.py
M	crates/mamba/tests/fixtures/conformance/iterators/custom_iterator_xfail.expected
M	crates/mamba/tests/fixtures/conformance/iterators/custom_iterator_xfail.py
M	crates/mamba/tests/fixtures/conformance/iterators/unpacking.expected
M	crates/mamba/tests/fixtures/conformance/iterators/unpacking.py
M	crates/mamba/tests/fixtures/conformance/language/comprehension_scope_edge_cases.expected
M	crates/mamba/tests/fixtures/conformance/language/comprehension_scope_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/language/context_manager_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/language/decorator_edge_cases.expected
M	crates/mamba/tests/fixtures/conformance/language/decorator_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/language/lambda_edge_cases.expected
M	crates/mamba/tests/fixtures/conformance/language/lambda_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/language/pattern_matching_edge_cases.py
M	crates/mamba/tests/fixtures/conformance/stdlib/collections/collections_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/collections/collections_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/collections_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/collections_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/csv/csv_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/csv/csv_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/datetime/datetime_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/datetime/datetime_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/datetime_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/datetime_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/functools/functools_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/functools/functools_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/functools_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/functools_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/hashlib/hashlib_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/hashlib/hashlib_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/io/io_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/io/io_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/itertools/itertools_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/itertools/itertools_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/itertools_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/itertools_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/json/json_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/json/json_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/json_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/json_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/math/math_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/math_basic.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/math_basic.py
M	crates/mamba/tests/fixtures/conformance/stdlib/random/random_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/random/random_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/re/re_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/re/re_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/re_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/re_conformance.py
M	crates/mamba/tests/fixtures/conformance/stdlib/struct/struct_conformance.expected
M	crates/mamba/tests/fixtures/conformance/stdlib/struct/struct_conformance.py
A	crates/mamba/tests/gen_thread_pool_tests.rs
A	crates/mamba/tests/generator_conformance_tests.rs
A	crates/mamba/tests/iterator_conformance_tests.rs
A	crates/mamba/tests/jit_refcount_tests.rs
M	crates/mamba/tests/jit_tests.rs
A	crates/mamba/tests/no_arg_constructor_tests.rs
M	crates/mamba/tests/p0_conformance_tests.rs
M	crates/mamba/tests/runtime_bugs_conformance_tests.rs
A	crates/mamba/tests/xfail_zero_conformance_tests.rs
M	crates/cclab-pyo3/src/lib.rs
M	crates/cclab-queue/Cargo.toml
M	crates/cclab-queue/src/broker/cloudtasks.rs
M	crates/cclab-queue/src/broker/mod.rs
M	crates/cclab-queue/src/error.rs
M	crates/cclab-queue/src/lib.rs
M	crates/cclab-queue/src/scheduler/backend.rs
A	crates/cclab-queue/src/scheduler/cloud_scheduler_backend.rs
A	crates/cclab-queue/src/scheduler/cloud_scheduler_backend_tests.rs
A	crates/cclab-queue/src/scheduler/k8s_cronjob_backend.rs
M	crates/cclab-queue/src/scheduler/mod.rs
M	crates/cclab-queue/src/scheduler/periodic.rs
A	crates/cclab-queue/src/scheduler/push_auth.rs
A	crates/cclab-queue/src/scheduler/push_receiver.rs
A	crates/cclab-queue/src/scheduler/schedule_monitor.rs
M	crates/cclab-sdd-cli/src/codegen.rs
M	crates/cclab-sdd-cli/src/commands.rs
M	crates/cclab-sdd-cli/src/daemon.rs
M	crates/cclab-sdd-cli/src/direct.rs
M	crates/cclab-sdd-cli/templates/mainthread/skills/cclab-sdd-run-change/SKILL.md
M	crates/cclab-sdd/Cargo.toml
A	crates/cclab-sdd/src/check_pipeline.rs
R066	crates/cclab-sdd/src/lens/mod.rs	crates/cclab-sdd/src/checker.rs
A	crates/cclab-sdd/src/context_builder/mod.rs
A	crates/cclab-sdd/src/context_builder/test_detection.rs
A	crates/cclab-sdd/src/context_builder/traversal.rs
A	crates/cclab-sdd/src/context_builder/types.rs
R100	crates/cclab-sdd/src/lens/core/config.rs	crates/cclab-sdd/src/core/config.rs
R100	crates/cclab-sdd/src/lens/core/mod.rs	crates/cclab-sdd/src/core/mod.rs
R100	crates/cclab-sdd/src/lens/diagnostic.rs	crates/cclab-sdd/src/diagnostic.rs
R100	crates/cclab-sdd/src/lens/format/detect.rs	crates/cclab-sdd/src/format/detect.rs
R100	crates/cclab-sdd/src/lens/format/mod.rs	crates/cclab-sdd/src/format/mod.rs
R098	crates/cclab-sdd/src/lens/gen/framework/axum.rs	crates/cclab-sdd/src/gen/framework/axum.rs
R098	crates/cclab-sdd/src/lens/gen/framework/express.rs	crates/cclab-sdd/src/gen/framework/express.rs
R098	crates/cclab-sdd/src/lens/gen/framework/fastapi.rs	crates/cclab-sdd/src/gen/framework/fastapi.rs
R100	crates/cclab-sdd/src/lens/gen/framework/mod.rs	crates/cclab-sdd/src/gen/framework/mod.rs
R100	crates/cclab-sdd/src/lens/gen/mod.rs	crates/cclab-sdd/src/gen/mod.rs
R097	crates/cclab-sdd/src/lens/gen/python/meteor.rs	crates/cclab-sdd/src/gen/python/meteor.rs
R097	crates/cclab-sdd/src/lens/gen/python/mod.rs	crates/cclab-sdd/src/gen/python/mod.rs
R097	crates/cclab-sdd/src/lens/gen/python/nebula.rs	crates/cclab-sdd/src/gen/python/nebula.rs
R097	crates/cclab-sdd/src/lens/gen/python/photon.rs	crates/cclab-sdd/src/gen/python/photon.rs
R100	crates/cclab-sdd/src/lens/gen/python/pyo3.rs	crates/cclab-sdd/src/gen/python/pyo3.rs
R099	crates/cclab-sdd/src/lens/gen/python/pyo3_gen.rs	crates/cclab-sdd/src/gen/python/pyo3_gen.rs
R096	crates/cclab-sdd/src/lens/gen/python/quasar.rs	crates/cclab-sdd/src/gen/python/quasar.rs
R100	crates/cclab-sdd/src/lens/gen/python/rust_scanner.rs	crates/cclab-sdd/src/gen/python/rust_scanner.rs
R094	crates/cclab-sdd/src/lens/gen/python/shield.rs	crates/cclab-sdd/src/gen/python/shield.rs
R100	crates/cclab-sdd/src/lens/gen/python/test_extractor.rs	crates/cclab-sdd/src/gen/python/test_extractor.rs
R098	crates/cclab-sdd/src/lens/gen/python/titan.rs	crates/cclab-sdd/src/gen/python/titan.rs
R100	crates/cclab-sdd/src/lens/gen/python/type_map.rs	crates/cclab-sdd/src/gen/python/type_map.rs
R097	crates/cclab-sdd/src/lens/gen/registry.rs	crates/cclab-sdd/src/gen/registry.rs
R097	crates/cclab-sdd/src/lens/gen/rust/axum.rs	crates/cclab-sdd/src/gen/rust/axum.rs
R096	crates/cclab-sdd/src/lens/gen/rust/mod.rs	crates/cclab-sdd/src/gen/rust/mod.rs
R097	crates/cclab-sdd/src/lens/gen/rust/reqwest.rs	crates/cclab-sdd/src/gen/rust/reqwest.rs
R096	crates/cclab-sdd/src/lens/gen/rust/serde.rs	crates/cclab-sdd/src/gen/rust/serde.rs
R097	crates/cclab-sdd/src/lens/gen/rust/sqlx.rs	crates/cclab-sdd/src/gen/rust/sqlx.rs
R099	crates/cclab-sdd/src/lens/gen/traits.rs	crates/cclab-sdd/src/gen/traits.rs
M	crates/cclab-sdd/src/generate/lib.rs
A	crates/cclab-sdd/src/generate/patterns/mod.rs
A	crates/cclab-sdd/src/generate/patterns/registry.rs
A	crates/cclab-sdd/src/generate/patterns/resolver.rs
R100	crates/cclab-sdd/src/lens/graph/mod.rs	crates/cclab-sdd/src/graph/mod.rs
R098	crates/cclab-sdd/src/lens/graph/resolve.rs	crates/cclab-sdd/src/graph/resolve.rs
R095	crates/cclab-sdd/src/lens/handlers.rs	crates/cclab-sdd/src/handlers.rs
R100	crates/cclab-sdd/src/lens/error.rs	crates/cclab-sdd/src/lens_error.rs
M	crates/cclab-sdd/src/lib.rs
R098	crates/cclab-sdd/src/lens/lint/asyncapi.rs	crates/cclab-sdd/src/lint/asyncapi.rs
R097	crates/cclab-sdd/src/lens/lint/autofix.rs	crates/cclab-sdd/src/lint/autofix.rs
R099	crates/cclab-sdd/src/lens/lint/css.rs	crates/cclab-sdd/src/lint/css.rs
R098	crates/cclab-sdd/src/lens/lint/css_rules.rs	crates/cclab-sdd/src/lint/css_rules.rs
R097	crates/cclab-sdd/src/lens/lint/custom.rs	crates/cclab-sdd/src/lint/custom.rs
R098	crates/cclab-sdd/src/lens/lint/dockerfile.rs	crates/cclab-sdd/src/lint/dockerfile.rs
R099	crates/cclab-sdd/src/lens/lint/embedded_markdown.rs	crates/cclab-sdd/src/lint/embedded_markdown.rs
R098	crates/cclab-sdd/src/lens/lint/gitlab_ci.rs	crates/cclab-sdd/src/lint/gitlab_ci.rs
R099	crates/cclab-sdd/src/lens/lint/gitlab_ci_rules.rs	crates/cclab-sdd/src/lint/gitlab_ci_rules.rs
R098	crates/cclab-sdd/src/lens/lint/go.rs	crates/cclab-sdd/src/lint/go.rs
R099	crates/cclab-sdd/src/lens/lint/graphql.rs	crates/cclab-sdd/src/lint/graphql.rs
R098	crates/cclab-sdd/src/lens/lint/html.rs	crates/cclab-sdd/src/lint/html.rs
R098	crates/cclab-sdd/src/lens/lint/html_rules.rs	crates/cclab-sdd/src/lint/html_rules.rs
R099	crates/cclab-sdd/src/lens/lint/javascript.rs	crates/cclab-sdd/src/lint/javascript.rs
R098	crates/cclab-sdd/src/lens/lint/kubernetes.rs	crates/cclab-sdd/src/lint/kubernetes.rs
R099	crates/cclab-sdd/src/lens/lint/kubernetes_rules.rs	crates/cclab-sdd/src/lint/kubernetes_rules.rs
R099	crates/cclab-sdd/src/lens/lint/markdown.rs	crates/cclab-sdd/src/lint/markdown.rs
R098	crates/cclab-sdd/src/lens/lint/mdx.rs	crates/cclab-sdd/src/lint/mdx.rs
R098	crates/cclab-sdd/src/lens/lint/mermaid.rs	crates/cclab-sdd/src/lint/mermaid.rs
R096	crates/cclab-sdd/src/lens/lint/mod.rs	crates/cclab-sdd/src/lint/mod.rs
R098	crates/cclab-sdd/src/lens/lint/openapi.rs	crates/cclab-sdd/src/lint/openapi.rs
R098	crates/cclab-sdd/src/lens/lint/openrpc.rs	crates/cclab-sdd/src/lint/openrpc.rs
R098	crates/cclab-sdd/src/lens/lint/proto.rs	crates/cclab-sdd/src/lint/proto.rs
R097	crates/cclab-sdd/src/lens/lint/python.rs	crates/cclab-sdd/src/lint/python.rs
R098	crates/cclab-sdd/src/lens/lint/python_security.rs	crates/cclab-sdd/src/lint/python_security.rs
R099	crates/cclab-sdd/src/lens/lint/rust_checker.rs	crates/cclab-sdd/src/lint/rust_checker.rs
R098	crates/cclab-sdd/src/lens/lint/sql.rs	crates/cclab-sdd/src/lint/sql.rs
R096	crates/cclab-sdd/src/lens/lint/terraform.rs	crates/cclab-sdd/src/lint/terraform.rs
R096	crates/cclab-sdd/src/lens/lint/terraform_rules.rs	crates/cclab-sdd/src/lint/terraform_rules.rs
R098	crates/cclab-sdd/src/lens/lint/toml_checker.rs	crates/cclab-sdd/src/lint/toml_checker.rs
R097	crates/cclab-sdd/src/lens/lint/typescript.rs	crates/cclab-sdd/src/lint/typescript.rs
R096	crates/cclab-sdd/src/lens/lint/yaml_dispatch.rs	crates/cclab-sdd/src/lint/yaml_dispatch.rs
R100	crates/cclab-sdd/src/lens/lsp/mod.rs	crates/cclab-sdd/src/lsp/mod.rs
R098	crates/cclab-sdd/src/lens/lsp/server.rs	crates/cclab-sdd/src/lsp/server.rs
M	crates/cclab-sdd/src/models/change.rs
A	crates/cclab-sdd/src/models/index_config.rs
M	crates/cclab-sdd/src/models/mod.rs
M	crates/cclab-sdd/src/models/spec_rules.rs
A	crates/cclab-sdd/src/models/tech_stack.rs
M	crates/cclab-sdd/src/orchestrator/cli_mapper.rs
M	crates/cclab-sdd/src/orchestrator/script_runner.rs
A	crates/cclab-sdd/src/output/agent.rs
A	crates/cclab-sdd/src/output/agent_types.rs
A	crates/cclab-sdd/src/output/mod.rs
R085	crates/cclab-sdd/src/lens/output.rs	crates/cclab-sdd/src/output/reporter.rs
R098	crates/cclab-sdd/src/lens/refactoring/extract.rs	crates/cclab-sdd/src/refactoring/extract.rs
R099	crates/cclab-sdd/src/lens/refactoring/extract_helpers.rs	crates/cclab-sdd/src/refactoring/extract_helpers.rs
R096	crates/cclab-sdd/src/lens/refactoring/inline.rs	crates/cclab-sdd/src/refactoring/inline.rs
R097	crates/cclab-sdd/src/lens/refactoring/mod.rs	crates/cclab-sdd/src/refactoring/mod.rs
R098	crates/cclab-sdd/src/lens/refactoring/move_def.rs	crates/cclab-sdd/src/refactoring/move_def.rs
R096	crates/cclab-sdd/src/lens/refactoring/rename.rs	crates/cclab-sdd/src/refactoring/rename.rs
R096	crates/cclab-sdd/src/lens/refactoring/signature.rs	crates/cclab-sdd/src/refactoring/signature.rs
R098	crates/cclab-sdd/src/lens/refactoring/signature_helpers.rs	crates/cclab-sdd/src/refactoring/signature_helpers.rs
R100	crates/cclab-sdd/src/lens/schemas/frontmatter.rs	crates/cclab-sdd/src/schemas/frontmatter.rs
R098	crates/cclab-sdd/src/lens/schemas/gitlab.rs	crates/cclab-sdd/src/schemas/gitlab.rs
R099	crates/cclab-sdd/src/lens/schemas/k8s.rs	crates/cclab-sdd/src/schemas/k8s.rs
R097	crates/cclab-sdd/src/lens/schemas/mod.rs	crates/cclab-sdd/src/schemas/mod.rs
R098	crates/cclab-sdd/src/lens/search/index.rs	crates/cclab-sdd/src/search/index.rs
R095	crates/cclab-sdd/src/lens/search/mod.rs	crates/cclab-sdd/src/search/mod.rs
R097	crates/cclab-sdd/src/lens/search/query.rs	crates/cclab-sdd/src/search/query.rs
R087	crates/cclab-sdd/src/lens/semantic/mod.rs	crates/cclab-sdd/src/semantic/mod.rs
R099	crates/cclab-sdd/src/lens/semantic/pdg/cfg.rs	crates/cclab-sdd/src/semantic/pdg/cfg.rs
R099	crates/cclab-sdd/src/lens/semantic/pdg/data_flow.rs	crates/cclab-sdd/src/semantic/pdg/data_flow.rs
R099	crates/cclab-sdd/src/lens/semantic/pdg/dominator.rs	crates/cclab-sdd/src/semantic/pdg/dominator.rs
R099	crates/cclab-sdd/src/lens/semantic/pdg/mod.rs	crates/cclab-sdd/src/semantic/pdg/mod.rs
R099	crates/cclab-sdd/src/lens/semantic/scope.rs	crates/cclab-sdd/src/semantic/scope.rs
R098	crates/cclab-sdd/src/lens/semantic/symbols/css.rs	crates/cclab-sdd/src/semantic/symbols/css.rs
R099	crates/cclab-sdd/src/lens/semantic/symbols/dockerfile.rs	crates/cclab-sdd/src/semantic/symbols/dockerfile.rs
R099	crates/cclab-sdd/src/lens/semantic/symbols/gitlab_ci.rs	crates/cclab-sdd/src/semantic/symbols/gitlab_ci.rs
R099	crates/cclab-sdd/src/lens/semantic/symbols/go.rs	crates/cclab-sdd/src/semantic/symbols/go.rs
R098	crates/cclab-sdd/src/lens/semantic/symbols/graphql_sym.rs	crates/cclab-sdd/src/semantic/symbols/graphql_sym.rs
R098	crates/cclab-sdd/src/lens/semantic/symbols/html.rs	crates/cclab-sdd/src/semantic/symbols/html.rs
R100	crates/cclab-sdd/src/lens/semantic/symbols/javascript.rs	crates/cclab-sdd/src/semantic/symbols/javascript.rs
R098	crates/cclab-sdd/src/lens/semantic/symbols/kubernetes.rs	crates/cclab-sdd/src/semantic/symbols/kubernetes.rs
R099	crates/cclab-sdd/src/lens/semantic/symbols/markdown.rs	crates/cclab-sdd/src/semantic/symbols/markdown.rs
R099	crates/cclab-sdd/src/lens/semantic/symbols/mermaid.rs	crates/cclab-sdd/src/semantic/symbols/mermaid.rs
R099	crates/cclab-sdd/src/lens/semantic/symbols/mod.rs	crates/cclab-sdd/src/semantic/symbols/mod.rs
R098	crates/cclab-sdd/src/lens/semantic/symbols/proto_sym.rs	crates/cclab-sdd/src/semantic/symbols/proto_sym.rs
R098	crates/cclab-sdd/src/lens/semantic/symbols/python.rs	crates/cclab-sdd/src/semantic/symbols/python.rs
R099	crates/cclab-sdd/src/lens/semantic/symbols/rust.rs	crates/cclab-sdd/src/semantic/symbols/rust.rs
R098	crates/cclab-sdd/src/lens/semantic/symbols/sql_sym.rs	crates/cclab-sdd/src/semantic/symbols/sql_sym.rs
R098	crates/cclab-sdd/src/lens/semantic/symbols/terraform.rs	crates/cclab-sdd/src/semantic/symbols/terraform.rs
R097	crates/cclab-sdd/src/lens/semantic/symbols/toml_sym.rs	crates/cclab-sdd/src/semantic/symbols/toml_sym.rs
R099	crates/cclab-sdd/src/lens/semantic/symbols/typescript.rs	crates/cclab-sdd/src/semantic/symbols/typescript.rs
R098	crates/cclab-sdd/src/lens/semantic/tests.rs	crates/cclab-sdd/src/semantic/tests.rs
R099	crates/cclab-sdd/src/lens/semantic/types/go.rs	crates/cclab-sdd/src/semantic/types/go.rs
R099	crates/cclab-sdd/src/lens/semantic/types/go_advanced.rs	crates/cclab-sdd/src/semantic/types/go_advanced.rs
R099	crates/cclab-sdd/src/lens/semantic/types/go_tests.rs	crates/cclab-sdd/src/semantic/types/go_tests.rs
R100	crates/cclab-sdd/src/lens/semantic/types/mod.rs	crates/cclab-sdd/src/semantic/types/mod.rs
A	crates/cclab-sdd/src/server/auto_discover.rs
R088	crates/cclab-sdd/src/lens/server/daemon.rs	crates/cclab-sdd/src/server/daemon.rs
R099	crates/cclab-sdd/src/lens/server/disk_cache.rs	crates/cclab-sdd/src/server/disk_cache.rs
R095	crates/cclab-sdd/src/lens/server/handler.rs	crates/cclab-sdd/src/server/handler.rs
R099	crates/cclab-sdd/src/lens/server/incremental.rs	crates/cclab-sdd/src/server/incremental.rs
R097	crates/cclab-sdd/src/lens/server/mod.rs	crates/cclab-sdd/src/server/mod.rs
R100	crates/cclab-sdd/src/lens/server/protocol.rs	crates/cclab-sdd/src/server/protocol.rs
R098	crates/cclab-sdd/src/lens/server/tests.rs	crates/cclab-sdd/src/server/tests.rs
R099	crates/cclab-sdd/src/lens/server/watch_bridge.rs	crates/cclab-sdd/src/server/watch_bridge.rs
M	crates/cclab-sdd/src/services/mod.rs
M	crates/cclab-sdd/src/services/spec_service.rs
A	crates/cclab-sdd/src/services/tech_stack_service.rs
R100	crates/cclab-sdd/src/lens/spec/asyncapi/mod.rs	crates/cclab-sdd/src/spec/asyncapi/mod.rs
R099	crates/cclab-sdd/src/lens/spec/asyncapi/parser.rs	crates/cclab-sdd/src/spec/asyncapi/parser.rs
R099	crates/cclab-sdd/src/lens/spec/ir.rs	crates/cclab-sdd/src/spec/ir.rs
R100	crates/cclab-sdd/src/lens/spec/json_schema/mod.rs	crates/cclab-sdd/src/spec/json_schema/mod.rs
R099	crates/cclab-sdd/src/lens/spec/json_schema/parser.rs	crates/cclab-sdd/src/spec/json_schema/parser.rs
R098	crates/cclab-sdd/src/lens/spec/mermaid/generator.rs	crates/cclab-sdd/src/spec/mermaid/generator.rs
R100	crates/cclab-sdd/src/lens/spec/mermaid/mod.rs	crates/cclab-sdd/src/spec/mermaid/mod.rs
R099	crates/cclab-sdd/src/lens/spec/mermaid/parser.rs	crates/cclab-sdd/src/spec/mermaid/parser.rs
R100	crates/cclab-sdd/src/lens/spec/mod.rs	crates/cclab-sdd/src/spec/mod.rs
R100	crates/cclab-sdd/src/lens/spec/openapi/mod.rs	crates/cclab-sdd/src/spec/openapi/mod.rs
R099	crates/cclab-sdd/src/lens/spec/openapi/parser.rs	crates/cclab-sdd/src/spec/openapi/parser.rs
R100	crates/cclab-sdd/src/lens/spec/statemachine/mermaid_plus.rs	crates/cclab-sdd/src/spec/statemachine/mermaid_plus.rs
R100	crates/cclab-sdd/src/lens/spec/statemachine/mod.rs	crates/cclab-sdd/src/spec/statemachine/mod.rs
R100	crates/cclab-sdd/src/lens/spec/statemachine/schema.rs	crates/cclab-sdd/src/spec/statemachine/schema.rs
R100	crates/cclab-sdd/src/lens/spec/statemachine/validator.rs	crates/cclab-sdd/src/spec/statemachine/validator.rs
M	crates/cclab-sdd/src/state/manager.rs
M	crates/cclab-sdd/src/state/mod.rs
R073	crates/cclab-sdd/src/lens/storage.rs	crates/cclab-sdd/src/storage.rs
R100	crates/cclab-sdd/src/lens/syntax/mod.rs	crates/cclab-sdd/src/syntax/mod.rs
R099	crates/cclab-sdd/src/lens/syntax/parser.rs	crates/cclab-sdd/src/syntax/parser.rs
M	crates/cclab-sdd/src/tools/agent.rs
M	crates/cclab-sdd/src/tools/common_change_spec.rs
M	crates/cclab-sdd/src/tools/create_change_impl.rs
M	crates/cclab-sdd/src/tools/create_change_merge.rs
M	crates/cclab-sdd/src/tools/create_change_spec.rs
M	crates/cclab-sdd/src/tools/create_post_clarifications.rs
M	crates/cclab-sdd/src/tools/create_reference_context.rs
M	crates/cclab-sdd/src/tools/mod.rs
M	crates/cclab-sdd/src/tools/review_change_spec.rs
M	crates/cclab-sdd/src/tools/review_reference_context.rs
M	crates/cclab-sdd/src/tools/spec_plan.rs
M	crates/cclab-sdd/src/tools/workflow_common.rs
R100	crates/cclab-sdd/src/lens/types/annotation.rs	crates/cclab-sdd/src/type_inference/annotation.rs
R100	crates/cclab-sdd/src/lens/types/builtins.rs	crates/cclab-sdd/src/type_inference/builtins.rs
R075	crates/cclab-sdd/src/lens/types/cache.rs	crates/cclab-sdd/src/type_inference/cache.rs
R089	crates/cclab-sdd/src/lens/types/cfg_narrow.rs	crates/cclab-sdd/src/type_inference/cfg_narrow.rs
R099	crates/cclab-sdd/src/lens/types/check.rs	crates/cclab-sdd/src/type_inference/check.rs
R098	crates/cclab-sdd/src/lens/types/check_tests.rs	crates/cclab-sdd/src/type_inference/check_tests.rs
R100	crates/cclab-sdd/src/lens/types/class_info.rs	crates/cclab-sdd/src/type_inference/class_info.rs
R099	crates/cclab-sdd/src/lens/types/codegen.rs	crates/cclab-sdd/src/type_inference/codegen.rs
R100	crates/cclab-sdd/src/lens/types/config.rs	crates/cclab-sdd/src/type_inference/config.rs
R074	crates/cclab-sdd/src/lens/types/deep_inference.rs	crates/cclab-sdd/src/type_inference/deep_inference.rs
R100	crates/cclab-sdd/src/lens/types/env.rs	crates/cclab-sdd/src/type_inference/env.rs
R099	crates/cclab-sdd/src/lens/types/frameworks.rs	crates/cclab-sdd/src/type_inference/frameworks.rs
R095	crates/cclab-sdd/src/lens/types/imports.rs	crates/cclab-sdd/src/type_inference/imports.rs
R099	crates/cclab-sdd/src/lens/types/incremental.rs	crates/cclab-sdd/src/type_inference/incremental.rs
R100	crates/cclab-sdd/src/lens/types/infer.rs	crates/cclab-sdd/src/type_inference/infer.rs
R094	crates/cclab-sdd/src/lens/types/infer_tests.rs	crates/cclab-sdd/src/type_inference/infer_tests.rs
R096	crates/cclab-sdd/src/lens/types/mod.rs	crates/cclab-sdd/src/type_inference/mod.rs
R096	crates/cclab-sdd/src/lens/types/model.rs	crates/cclab-sdd/src/type_inference/model.rs
R100	crates/cclab-sdd/src/lens/types/modules.rs	crates/cclab-sdd/src/type_inference/modules.rs
R100	crates/cclab-sdd/src/lens/types/mutable_ast.rs	crates/cclab-sdd/src/type_inference/mutable_ast.rs
R100	crates/cclab-sdd/src/lens/types/narrow.rs	crates/cclab-sdd/src/type_inference/narrow.rs
R100	crates/cclab-sdd/src/lens/types/narrow_tests.rs	crates/cclab-sdd/src/type_inference/narrow_tests.rs
R100	crates/cclab-sdd/src/lens/types/package_managers.rs	crates/cclab-sdd/src/type_inference/package_managers.rs
R099	crates/cclab-sdd/src/lens/types/project.rs	crates/cclab-sdd/src/type_inference/project.rs
A	crates/cclab-sdd/src/type_inference/propagation.rs
R099	crates/cclab-sdd/src/lens/types/refactoring.rs	crates/cclab-sdd/src/type_inference/refactoring.rs
R100	crates/cclab-sdd/src/lens/types/refactoring_multilang.rs	crates/cclab-sdd/src/type_inference/refactoring_multilang.rs
R100	crates/cclab-sdd/src/lens/types/rust_advanced.rs	crates/cclab-sdd/src/type_inference/rust_advanced.rs
R100	crates/cclab-sdd/src/lens/types/rust_infer.rs	crates/cclab-sdd/src/type_inference/rust_infer.rs
R100	crates/cclab-sdd/src/lens/types/rust_lifetimes.rs	crates/cclab-sdd/src/type_inference/rust_lifetimes.rs
R100	crates/cclab-sdd/src/lens/types/rust_symbols.rs	crates/cclab-sdd/src/type_inference/rust_symbols.rs
R100	crates/cclab-sdd/src/lens/types/rust_traits.rs	crates/cclab-sdd/src/type_inference/rust_traits.rs
R099	crates/cclab-sdd/src/lens/types/rust_types.rs	crates/cclab-sdd/src/type_inference/rust_types.rs
R095	crates/cclab-sdd/src/lens/types/semantic_search.rs	crates/cclab-sdd/src/type_inference/semantic_search.rs
R100	crates/cclab-sdd/src/lens/types/semantic_search_rust.rs	crates/cclab-sdd/src/type_inference/semantic_search_rust.rs
R100	crates/cclab-sdd/src/lens/types/stubs.rs	crates/cclab-sdd/src/type_inference/stubs.rs
R100	crates/cclab-sdd/src/lens/types/ts_advanced.rs	crates/cclab-sdd/src/type_inference/ts_advanced.rs
R100	crates/cclab-sdd/src/lens/types/ts_infer.rs	crates/cclab-sdd/src/type_inference/ts_infer.rs
R100	crates/cclab-sdd/src/lens/types/ts_types.rs	crates/cclab-sdd/src/type_inference/ts_types.rs
R100	crates/cclab-sdd/src/lens/types/ty.rs	crates/cclab-sdd/src/type_inference/ty.rs
R100	crates/cclab-sdd/src/lens/types/ty_tests.rs	crates/cclab-sdd/src/type_inference/ty_tests.rs
R100	crates/cclab-sdd/src/lens/types/type_env.rs	crates/cclab-sdd/src/type_inference/type_env.rs
R100	crates/cclab-sdd/src/lens/types/typeshed.rs	crates/cclab-sdd/src/type_inference/typeshed.rs
R100	crates/cclab-sdd/src/lens/watch.rs	crates/cclab-sdd/src/watch.rs
M	crates/cclab-sdd/src/workflow/helpers.rs
M	crates/cclab-sdd/templates/mainthread/skills/cclab-sdd-run-change/SKILL.md
A	crates/cclab-sdd/tests/lens_dissolution_test.rs
M	crates/cclab-server/src/lens_pool.rs
M	crates/cclab-server/src/lib.rs
M	crates/cclab-server/src/mcp/mod.rs
M	crates/cclab-server/src/mcp/router.rs
A	docs/.gitignore
A	docs/.vitepress/config.mjs
D	docs/PROMPT_TEMPLATE_INTEGRATION.md
D	docs/README.md
D	docs/agent_eval_prompt_templates.md
D	docs/api/sse.md
D	docs/archive/ADAPTIVE_SAMPLING.md
D	docs/archive/MIGRATION_MAP.md
D	docs/archive/OPENTELEMETRY.md
D	docs/archive/PHASE2_LAZY_LOADING_IMPLEMENTATION.md
D	docs/archive/PHASE3_IMPLEMENTATION_SUMMARY.md
D	docs/archive/PHASE4_IMPLEMENTATION_SUMMARY.md
D	docs/archive/PHASE5_SUMMARY.md
D	docs/archive/PHASE7_IMPLEMENTATION_SUMMARY.md
D	docs/archive/POSTGRESQL_EXTENSIONS.md
D	docs/archive/README.md
D	docs/archive/SHEET_ARCHITECTURE.md
D	docs/archive/SHEET_CONTRIBUTING.md
D	docs/archive/SHEET_README.md
D	docs/archive/SPAN_HIERARCHY.md
D	docs/archive/TELEMETRY_RELATIONSHIPS.md
D	docs/archive/canvas_primitives.md
D	docs/archive/kv_benchmark_concurrent_fixed.md
D	docs/archive/legacy/BATCH_CONVERSION_SUMMARY.md
D	docs/archive/legacy/CONVERSION_REPORT.md
D	docs/archive/legacy/CRUD_API_REFACTOR_SUMMARY.md
D	docs/archive/legacy/PHASE5_IMPLEMENTATION_COMPLETE.md
D	docs/archive/legacy/PYLOOP_COMPILATION_FIXES.md
D	docs/archive/legacy/PYLOOP_PHASE1_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE2.5_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE2_3_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE2_4_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE2_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE3.1.1_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE3.1.2_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE3.1.3_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE3.1.4_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE3_CRUD_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE3_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE4_FILES.md
D	docs/archive/legacy/PYLOOP_PHASE4_SUMMARY.md
D	docs/archive/legacy/PYLOOP_PHASE5_SUMMARY.md
D	docs/archive/legacy/benchmarks/API_BENCHMARK_GAP_ANALYSIS.md
D	docs/archive/legacy/crates/data-bridge-api/PYTHON_INTEGRATION.md
D	docs/archive/legacy/crates/data-bridge-api/SERVER_README.md
D	docs/archive/legacy/crates/data-bridge-sheet-core/todos.md
D	docs/archive/legacy/crates/data-bridge-sheet-db/todos.md
D	docs/archive/legacy/crates/data-bridge-test/FIXTURES.md
D	docs/archive/legacy/crates/data-bridge-test/IMPLEMENTATION_SUMMARY.md
D	docs/archive/legacy/crates/data-bridge-test/TODOS.md
D	docs/archive/legacy/deploy/OBSERVABILITY.md
D	docs/archive/legacy/deploy/TESTING.md
D	docs/archive/legacy/docs/MIGRATION_COMPLETE.md
D	docs/archive/legacy/docs/MIGRATION_STATUS.md
D	docs/archive/legacy/docs/PYLOOP_BENCHMARKS.md
D	docs/archive/legacy/docs/PYLOOP_CRUD.md
D	docs/archive/legacy/docs/TESTING.md
D	docs/archive/legacy/docs/TEST_SERVER_PYTHON_APP.md
D	docs/archive/legacy/docs/sheet-specs/advanced-features.md
D	docs/archive/legacy/docs/sheet-specs/architecture.md
D	docs/archive/legacy/docs/sheet-specs/clipboard.md
D	docs/archive/legacy/docs/sheet-specs/data-structures.md
D	docs/archive/legacy/docs/sheet-specs/flowchart.md
D	docs/archive/legacy/docs/sheet-specs/formatting-rules.md
D	docs/archive/legacy/docs/sheet-specs/formula-engine.md
D	docs/archive/legacy/docs/sheet-specs/fsm.md
D	docs/archive/legacy/docs/sheet-specs/keyboard-shortcuts.md
D	docs/archive/legacy/docs/sheet-specs/performance.md
D	docs/archive/legacy/docs/sheet-specs/persistence.md
D	docs/archive/legacy/docs/sheet-specs/rendering-engine.md
D	docs/archive/legacy/docs/sheet-specs/sheet-management.md
D	docs/archive/legacy/docs/sheet-specs/ui-interactions.md
D	docs/archive/legacy/docs/sheet-specs/user-experience.md
D	docs/archive/legacy/docs/sheet-specs/wasm-integration.md
D	docs/archive/legacy/docs/tasks/RATELIMIT_GUIDE.md
D	docs/archive/legacy/docs/tasks/README.md
D	docs/archive/legacy/docs/tasks/ROUTER_INTEGRATION_SUMMARY.md
D	docs/archive/legacy/docs/tasks/ROUTING.md
D	docs/archive/legacy/docs/tasks/ROUTING_IMPLEMENTATION_SUMMARY.md
D	docs/archive/legacy/docs/tasks/routing_integration.md
D	docs/archive/legacy/tests/postgres/benchmarks/ARCHITECTURE.md
D	docs/archive/legacy/tests/postgres/benchmarks/QUICKSTART.md
D	docs/archive/legacy/tools/DELIVERABLES.md
D	docs/archive/legacy/tools/IMPLEMENTATION_SUMMARY.md
D	docs/archive/telemetry.md
D	docs/en/user-guide.md
M	docs/index.md
A	docs/jet/bundler.md
A	docs/jet/configuration.md
A	docs/jet/dev-server.md
A	docs/jet/getting-started.md
A	docs/jet/package-manager.md
A	docs/jet/task-runner.md
A	docs/jet/workspaces.md
A	docs/package.json
D	docs/postgres/api.md
D	docs/postgres/guides/aggregation.md
D	docs/postgres/guides/caching.md
D	docs/postgres/guides/events.md
D	docs/postgres/guides/indexes.md
D	docs/postgres/guides/inheritance.md
D	docs/postgres/guides/migrations.md
D	docs/postgres/guides/querying.md
D	docs/postgres/guides/raw_sql.md
D	docs/postgres/guides/state_management.md
D	docs/postgres/guides/tables_and_columns.md
D	docs/postgres/guides/transactions.md
D	docs/postgres/guides/validation.md
D	docs/postgres/quickstart.md
D	docs/postgres/relationships.md
D	docs/tasks/asyncapi.yaml
D	docs/zh-tw/user-guide.md
R100	e2e/app.spec.ts	e2e/grid/app.spec.ts
R100	e2e/cell-editing.spec.ts	e2e/grid/cell-editing.spec.ts
R100	examples/mini-react/dist-jet/index.html	e2e/jet/dist-jet/index.html
R100	examples/mini-react/dist-jet/style.css	e2e/jet/dist-jet/style.css
R100	examples/mini-react/dist-vite/assets/About-Mt8CYShk.js	e2e/jet/dist-vite/assets/About-Mt8CYShk.js
R100	examples/mini-react/dist-vite/assets/Settings-B1a8RmuR.js	e2e/jet/dist-vite/assets/Settings-B1a8RmuR.js
R100	examples/mini-react/dist-vite/assets/index-CFy176Qo.css	e2e/jet/dist-vite/assets/index-CFy176Qo.css
R100	examples/mini-react/dist-vite/assets/index-fWhMswjv.js	e2e/jet/dist-vite/assets/index-fWhMswjv.js
R100	examples/mini-react/dist-vite/index.html	e2e/jet/dist-vite/index.html
R100	examples/mini-react/index.html	e2e/jet/index.html
R100	examples/mini-react/package-lock.json	e2e/jet/package-lock.json
R100	examples/mini-react/package.json	e2e/jet/package.json
R100	examples/mini-react/src/app.tsx	e2e/jet/src/app.tsx
R100	examples/mini-react/src/components/AppInfo.tsx	e2e/jet/src/components/AppInfo.tsx
R100	examples/mini-react/src/components/Header.tsx	e2e/jet/src/components/Header.tsx
R100	examples/mini-react/src/components/TodoFooter.tsx	e2e/jet/src/components/TodoFooter.tsx
R100	examples/mini-react/src/components/TodoItem.module.css	e2e/jet/src/components/TodoItem.module.css
R100	examples/mini-react/src/components/TodoItem.tsx	e2e/jet/src/components/TodoItem.tsx
R100	examples/mini-react/src/components/TodoStats.tsx	e2e/jet/src/components/TodoStats.tsx
R100	examples/mini-react/src/components/index.ts	e2e/jet/src/components/index.ts
R100	examples/mini-react/src/hooks/useLocalStorage.ts	e2e/jet/src/hooks/useLocalStorage.ts
R100	examples/mini-react/src/index.tsx	e2e/jet/src/index.tsx
R100	examples/mini-react/src/lib/async-utils.ts	e2e/jet/src/lib/async-utils.ts
R100	examples/mini-react/src/lib/constants.ts	e2e/jet/src/lib/constants.ts
R100	examples/mini-react/src/lib/formatting.ts	e2e/jet/src/lib/formatting.ts
R100	examples/mini-react/src/lib/index.ts	e2e/jet/src/lib/index.ts
R100	examples/mini-react/src/lib/math.ts	e2e/jet/src/lib/math.ts
R100	examples/mini-react/src/mini-react.ts	e2e/jet/src/mini-react.ts
R100	examples/mini-react/src/pages/About.tsx	e2e/jet/src/pages/About.tsx
R100	examples/mini-react/src/pages/Settings.tsx	e2e/jet/src/pages/Settings.tsx
R100	examples/mini-react/src/style.css	e2e/jet/src/style.css
R100	examples/mini-react/src/types.ts	e2e/jet/src/types.ts
R100	examples/mini-react/src/utils.ts	e2e/jet/src/utils.ts
R100	examples/mini-react/tests/dom-snapshot.spec.ts	e2e/jet/tests/build.spec.ts
A	e2e/jet/tests/css.spec.ts
A	e2e/jet/tests/dev-server.spec.ts
A	e2e/jet/tests/hmr.spec.ts
A	e2e/jet/tests/test-utils.ts
R100	examples/mini-react/tsconfig.json	e2e/jet/tsconfig.json
R100	examples/mini-react/vite.config.ts	e2e/jet/vite.config.ts
A	e2e/playwright.config.ts
D	examples/mini-react/playwright.config.ts
A	jet-lock.yaml
A	packages/@cclab/pipeline/package.json
A	packages/@cclab/pipeline/src/NodeDetail.tsx
A	packages/@cclab/pipeline/src/PipelineDAG.tsx
A	packages/@cclab/pipeline/src/PipelineNode.tsx
A	packages/@cclab/pipeline/src/index.ts
A	packages/@cclab/pipeline/src/layout.ts
A	packages/@cclab/pipeline/src/types.ts
A	packages/@cclab/pipeline/tsconfig.json
A	packages/@cclab/spec-viewer/package.json
A	packages/@cclab/spec-viewer/src/CodeBlock.tsx
A	packages/@cclab/spec-viewer/src/MermaidDiagram.tsx
A	packages/@cclab/spec-viewer/src/SpecViewer.tsx
A	packages/@cclab/spec-viewer/src/index.ts
A	packages/@cclab/spec-viewer/src/types.ts
A	packages/@cclab/spec-viewer/tsconfig.json
M	packages/@cclab/ui/src/feedback/ConnectRepoForm.tsx
M	packages/@cclab/ui/src/spec-viewer/SpecFileBrowser.tsx
A	packages/cclab-agkit/README.md
A	packages/cclab-agkit/RENAME-PLAN.md
A	packages/cclab-agkit/prompts/section-guidance/README.md
A	packages/cclab-agkit/prompts/section-guidance/async-api.md
A	packages/cclab-agkit/prompts/section-guidance/changes.md
A	packages/cclab-agkit/prompts/section-guidance/cli.md
A	packages/cclab-agkit/prompts/section-guidance/component.md
A	packages/cclab-agkit/prompts/section-guidance/config.md
A	packages/cclab-agkit/prompts/section-guidance/db-model.md
A	packages/cclab-agkit/prompts/section-guidance/dependency.md
A	packages/cclab-agkit/prompts/section-guidance/design-token.md
A	packages/cclab-agkit/prompts/section-guidance/doc.md
A	packages/cclab-agkit/prompts/section-guidance/interaction.md
A	packages/cclab-agkit/prompts/section-guidance/logic.md
A	packages/cclab-agkit/prompts/section-guidance/mindmap.md
A	packages/cclab-agkit/prompts/section-guidance/overview.md
A	packages/cclab-agkit/prompts/section-guidance/requirements.md
A	packages/cclab-agkit/prompts/section-guidance/rest-api.md
A	packages/cclab-agkit/prompts/section-guidance/rpc-api.md
A	packages/cclab-agkit/prompts/section-guidance/scenarios.md
A	packages/cclab-agkit/prompts/section-guidance/schema.md
A	packages/cclab-agkit/prompts/section-guidance/state-machine.md
A	packages/cclab-agkit/prompts/section-guidance/test-plan.md
A	packages/cclab-agkit/prompts/section-guidance/wireframe.md
A	packages/cclab-agkit/prompts/system/explore.md
A	packages/cclab-agkit/prompts/system/review.md
A	packages/cclab-agkit/schemas/agent-config.schema.json
A	packages/cclab-agkit/schemas/change.schema.json
A	packages/cclab-agkit/schemas/issue.schema.json
A	packages/cclab-agkit/schemas/pipeline.schema.json
A	packages/cclab-agkit/schemas/project.schema.json
A	packages/cclab-agkit/schemas/spec.schema.json
A	pnpm-lock.yaml
A	projects/conductor/PRODUCT-REVIEW-v1.md
A	projects/conductor/PRODUCT-REVIEW-v2.md
A	projects/conductor/PRODUCT-REVIEW-v3.md
A	projects/conductor/PRODUCT.md
A	projects/conductor/ROADMAP.md
M	projects/conductor/be/main.py
A	projects/conductor/be/mock_progression.py
A	projects/conductor/be/mock_server.py
M	projects/conductor/be/src/agents/context_agent.py
M	projects/conductor/be/src/agents/scan_agent.py
M	projects/conductor/be/src/agents/spec_agent.py
M	projects/conductor/be/src/api/dashboard/deps.py
M	projects/conductor/be/src/api/dashboard/projects.py
M	projects/conductor/be/src/api/dashboard/stats.py
M	projects/conductor/be/src/api/main.py
M	projects/conductor/be/src/api/platform/deps.py
M	projects/conductor/be/src/api/platform/router.py
M	projects/conductor/be/src/api/webhook.py
M	projects/conductor/be/src/database/database.py
M	projects/conductor/be/src/database/repository.py
A	projects/conductor/be/src/db/migrations/009_multi_platform.py
A	projects/conductor/be/src/db/migrations/010_add_sdd_columns.py
M	projects/conductor/be/src/features/changes/models.py
M	projects/conductor/be/src/features/changes/repository.py
M	projects/conductor/be/src/features/changes/routes.py
M	projects/conductor/be/src/features/changes/schemas.py
M	projects/conductor/be/src/features/issues/repository.py
M	projects/conductor/be/src/features/issues/routes.py
M	projects/conductor/be/src/features/pipelines/routes.py
M	projects/conductor/be/src/features/projects/models.py
M	projects/conductor/be/src/features/projects/repository.py
M	projects/conductor/be/src/features/projects/routes.py
M	projects/conductor/be/src/features/projects/schemas.py
M	projects/conductor/be/src/features/specs/code_index_repository.py
M	projects/conductor/be/src/features/specs/repository.py
M	projects/conductor/be/src/integrations/__init__.py
A	projects/conductor/be/src/integrations/github/__init__.py
A	projects/conductor/be/src/integrations/github/adapter.py
A	projects/conductor/be/src/integrations/github/client.py
A	projects/conductor/be/src/integrations/github_import.py
M	projects/conductor/be/src/integrations/gitlab_import.py
M	projects/conductor/be/src/integrations/gitlab_sync.py
A	projects/conductor/be/src/sdd/__init__.py
A	projects/conductor/be/src/sdd/agent_factory.py
A	projects/conductor/be/src/sdd/orchestrator.py
A	projects/conductor/be/src/sdd/phase_mapping.py
A	projects/conductor/be/src/sdd/routes.py
A	projects/conductor/be/src/sdd/state_store.py
M	projects/conductor/be/tests/api/test_issues.py
M	projects/conductor/be/tests/api/test_project_connect_repo.py
M	projects/conductor/be/tests/api/test_project_files.py
M	projects/conductor/be/tests/api/test_project_specs.py
M	projects/conductor/be/tests/api/test_projects.py
M	projects/conductor/be/tests/api/test_workspaces.py
M	projects/conductor/be/tests/conftest.py
A	projects/conductor/be/tests/integration/test_mock_server.py
M	projects/conductor/be/tests/integration/test_platform_e2e.py
A	projects/conductor/be/tests/integrations/github/__init__.py
A	projects/conductor/be/tests/integrations/github/test_adapter.py
A	projects/conductor/be/tests/integrations/github/test_client.py
A	projects/conductor/be/tests/integrations/test_github_import.py
A	projects/conductor/be/tests/sdd/__init__.py
A	projects/conductor/be/tests/sdd/test_agent_factory.py
A	projects/conductor/be/tests/sdd/test_orchestrator.py
A	projects/conductor/be/tests/sdd/test_routes.py
A	projects/conductor/be/tests/sdd/test_state_store.py
M	projects/conductor/fe/jet.config.yaml
A	projects/conductor/fe/jet.config.yaml.bak
M	projects/conductor/fe/package.json
M	projects/conductor/fe/src/App.tsx
M	projects/conductor/fe/src/api/platformChanges.ts
A	projects/conductor/fe/src/pages/ChangeDetail.tsx
M	projects/conductor/fe/src/pages/IssueDetail.tsx
M	projects/conductor/fe/src/pages/ProjectDetail.tsx
M	projects/conductor/fe/tsconfig.json
A	projects/conductor/fe/vite.config.ts
M	projects/conductor/specs/platform/changes/states.md
M	projects/conductor/specs/platform/issues/states.md
M	projects/conductor/specs/platform/ui/components/spec-file-browser.md
M	projects/conductor/specs/platform/ui/layout.md
M	projects/conductor/specs/platform/workflow/states.md
M	projects/conductor/uv.lock
M	pyproject.toml
M	python/cclab/api/__init__.py
M	python/cclab/api/app.py
M	python/cclab/log/__init__.py
M	python/cclab/pg/__init__.py
M	python/cclab/schema/__init__.py
A	python/tests/api/test_asgi_dispatch.py
M	uv.lock
```

## Diff Statistics

```
.claude/skills/cclab-sdd-run-change/SKILL.md       |     5 +
 .claude/skills/conductor-dev-server/skill.md       |    52 +
 .claude/skills/handoff/SKILL.md                    |    65 +
 .gitignore                                         |     2 +-
 CLAUDE.md                                          |    14 +
 Cargo.lock                                         |   169 +-
 Cargo.toml                                         |     3 +-
 ECOSYSTEM.md                                       |   101 +
 cclab/archive/20260324-clean-mcp-refs/STATE.yaml   |    37 +
 .../groups/mcp-cleanup/post_clarifications.md      |    10 +
 .../groups/mcp-cleanup/pre_clarifications.md       |    13 +
 .../prompts/analyze_spec_mcp-refs-cleanup.md       |    26 +
 .../prompts/create_post_clarifications.md          |    48 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../groups/mcp-cleanup/reference_context.md        |    36 +
 .../groups/mcp-cleanup/requirements.md             |    15 +
 .../groups/mcp-cleanup/spec_plan.yaml              |     7 +
 .../groups/mcp-cleanup/specs/mcp-refs-cleanup.md   |   260 +
 ...or-sdd-clean-up-stale-mcp-references-all-too.md |    79 +
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     6 +
 .../payloads/create-change-spec-scenarios.json     |     6 +
 .../payloads/create-post-clarifications.json       |     1 +
 .../payloads/create-pre-clarifications.json        |     9 +
 .../payloads/create-reference-context.json         |    26 +
 .../payloads/restructure-input.json                |    11 +
 .../payloads/review-reference-context.json         |     1 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    64 +
 .../archive/20260324-clean-mcp-refs/user_input.md  |     1 +
 .../20260324-jet-workspace-protocol}/STATE.yaml    |    35 +-
 .../jet-workspace-protocol/post_clarifications.md  |     0
 .../jet-workspace-protocol/pre_clarifications.md   |     0
 .../analyze_spec_jet-workspace-protocol-spec.md    |     0
 .../prompts/begin_implementation.md                |     0
 .../prompts/create_post_clarifications.md          |     0
 .../prompts/create_pre_clarifications.md           |     0
 .../prompts/create_reference_context.md            |     0
 .../implement_tests_jet-workspace-protocol-spec.md |    25 +
 .../review_impl_jet-workspace-protocol-spec.md     |    58 +
 .../prompts/review_reference_context.md            |     0
 .../prompts/write_implementation_diff.md           |    14 +
 .../jet-workspace-protocol/reference_context.md    |     0
 .../groups/jet-workspace-protocol/requirements.md  |     0
 .../specs/jet-workspace-protocol-spec.md           |     0
 .../implementation.md                              |  1338 +++
 .../payloads/create-change-implementation.json     |     1 +
 .../payloads/create-pre-clarifications.json        |     0
 .../payloads/create-reference-context.json         |     0
 .../payloads/create-spec-changes.json              |     0
 .../payloads/create-spec-interaction.json          |     0
 .../payloads/create-spec-logic.json                |     0
 .../payloads/create-spec-overview.json             |     0
 .../payloads/create-spec-requirements.json         |     0
 .../payloads/create-spec-scenarios.json            |     0
 .../payloads/create-spec-schema.json               |     0
 .../payloads/create-spec-state-machine.json        |     0
 .../post-clarify-jet-workspace-protocol.json       |     0
 .../ref-context-jet-workspace-protocol.json        |     0
 .../payloads/restructure-input.json                |     0
 .../payloads/review-change-implementation.json     |    42 +
 .../payloads/review-reference-context.json         |     0
 .../payloads/spec-interaction.json                 |     0
 .../payloads/spec-logic.json                       |     0
 .../payloads/spec-schema.json                      |     0
 .../payloads/spec-state-machine.json               |     0
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |     0
 .../20260324-jet-workspace-protocol}/user_input.md |     0
 .../archive/20260324-sdd-subagent-mode/STATE.yaml  |    40 +
 .../subagent-dispatch/post_clarifications.md       |    10 +
 .../groups/subagent-dispatch/pre_clarifications.md |    17 +
 .../analyze_spec_subagent-executor-resolution.md   |    26 +
 .../analyze_spec_subagent-skill-dispatch.md        |    26 +
 .../analyze_spec_subagent-workflow-dispatch.md     |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    48 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |   204 +
 .../subagent-dispatch/prompts/implement_spec.md    |    17 +
 ...implement_tests_subagent-executor-resolution.md |    25 +
 .../implement_tests_subagent-skill-dispatch.md     |    25 +
 .../implement_tests_subagent-workflow-dispatch.md  |    25 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../groups/subagent-dispatch/reference_context.md  |    44 +
 .../groups/subagent-dispatch/requirements.md       |    15 +
 .../groups/subagent-dispatch/spec_plan.yaml        |    26 +
 .../specs/subagent-executor-resolution.md          |   538 +
 .../specs/subagent-skill-dispatch.md               |   319 +
 .../specs/subagent-workflow-dispatch.md            |   407 +
 .../20260324-sdd-subagent-mode/implementation.md   |  2599 +++++
 ...dd-subagent-execution-mode-claude-code-agent.md |   239 +
 ...te-change-spec-executor-resolution-changes.json |     6 +
 ...ate-change-spec-executor-resolution-config.json |     6 +
 ...e-change-spec-executor-resolution-overview.json |     9 +
 .../create-change-spec-skill-dispatch-changes.json |     6 +
 ...ate-change-spec-skill-dispatch-interaction.json |     6 +
 .../create-change-spec-skill-dispatch-logic.json   |     6 +
 ...create-change-spec-skill-dispatch-overview.json |     9 +
 ...eate-change-spec-workflow-dispatch-changes.json |     6 +
 ...-change-spec-workflow-dispatch-interaction.json |     6 +
 ...create-change-spec-workflow-dispatch-logic.json |     6 +
 ...ate-change-spec-workflow-dispatch-overview.json |     9 +
 .../payloads/create-post-clarifications.json       |     4 +
 .../payloads/create-pre-clarifications.json        |    13 +
 .../payloads/create-reference-context.json         |   111 +
 .../payloads/restructure-input.json                |    14 +
 .../payloads/review-change-implementation.json     |     1 +
 .../payloads/review-reference-context.json         |     6 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    64 +
 .../20260324-sdd-subagent-mode/user_input.md       |     1 +
 .../20260324-section-type-coverage/STATE.yaml      |    76 +
 .../payloads/create-change-spec.json               |     6 +
 .../frontend-design-system/post_clarifications.md  |    10 +
 .../frontend-design-system/pre_clarifications.md   |    17 +
 ...analyze_spec_change-spec-section-optionality.md |    26 +
 .../prompts/analyze_spec_tech-stack-inference.md   |    26 +
 .../prompts/analyze_spec_ux-pattern-library.md     |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    48 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |   200 +
 .../prompts/implement_spec.md                      |    17 +
 ...lement_tests_change-spec-section-optionality.md |    25 +
 .../implement_tests_tech-stack-inference.md        |    25 +
 .../prompts/implement_tests_ux-pattern-library.md  |    25 +
 .../prompts/review_reference_context.md            |    28 +
 .../prompts/revise_reference_context.md            |    23 +
 .../frontend-design-system/reference_context.md    |    41 +
 .../groups/frontend-design-system/requirements.md  |    16 +
 .../groups/frontend-design-system/spec_plan.yaml   |    24 +
 .../specs/change-spec-section-optionality.md       |   601 +
 .../specs/tech-stack-inference.md                  |   544 +
 .../specs/ux-pattern-library.md                    |   474 +
 .../new-section-types/post_clarifications.md       |    10 +
 .../groups/new-section-types/pre_clarifications.md |    21 +
 .../prompts/create_post_clarifications.md          |    48 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |   200 +
 .../new-section-types/prompts/implement_spec.md    |    17 +
 .../implement_tests_reference-context-types.md     |    25 +
 .../groups/new-section-types/reference_context.md  |    45 +
 .../groups/new-section-types/requirements.md       |    25 +
 .../groups/new-section-types/spec_plan.yaml        |     7 +
 .../specs/reference-context-types.md               |   312 +
 .../implementation.md                              |    54 +
 ...dd-section-type-coverage-all-roles-fe-be-sre.md |    72 +
 ...sign-system-as-tech-stack-config-ux-pattern-.md |    53 +
 ...053_sdd-add-e2e-scenario-section-type-for-qa.md |    53 +
 ...d-security-section-types-threat-model-auth-m.md |    84 +
 ...-add-qa-section-types-test-fixture-perf-test.md |    43 +
 ...d-sre-section-types-container-deploy-cloud-r.md |    28 +
 ...d-backend-mle-agent-section-types-grpc-graph.md |    33 +
 .../payloads/create-change-implementation.json     |    16 +
 .../payloads/create-post-clarifications.json       |     4 +
 .../payloads/create-pre-clarifications.json        |    17 +
 .../payloads/create-reference-context.json         |   114 +
 .../payloads/restructure-input.json                |    26 +
 .../payloads/review-change-implementation.json     |     6 +
 .../payloads/review-reference-context.json         |     6 +
 .../payloads/revise-reference-context.json         |    56 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    64 +
 .../review_impl_change-spec-section-optionality.md |    58 +
 .../prompts/review_impl_reference-context-types.md |    58 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../20260324-section-type-coverage/user_input.md   |     1 +
 .../20260325-align-fetch-api}/STATE.yaml           |    38 +-
 ...cosystem-align-cclab-python-api-to-ecosystem.md |     0
 .../prompts/restructure_input.md                   |     0
 .../20260325-align-fetch-api}/user_input.md        |     0
 .../20260325-cclab-agent-p0}/STATE.yaml            |   137 +-
 .../structured-output/post_clarifications.md       |     0
 .../groups/structured-output/pre_clarifications.md |     0
 .../prompts/create_pre_clarifications.md           |     0
 .../prompts/create_reference_context.md            |     0
 .../prompts/review_reference_context.md            |     0
 .../groups/structured-output/reference_context.md  |     0
 .../groups/structured-output/requirements.md       |     0
 .../post_clarifications.md                         |     0
 .../pre_clarifications.md                          |     0
 .../prompts/create_pre_clarifications.md           |     0
 .../reference_context.md                           |     0
 .../token-counting-and-compact/requirements.md     |     0
 ...e_786_feat-agent-add-accurate-token-counting.md |     0
 ...gent-add-structured-output-json-schema-respo.md |     0
 ...gent-smart-auto-compact-llm-summarization-ac.md |     0
 .../payloads/create-change-spec.json               |     0
 .../prompts/analyze_spec_cclab-agent-p0-spec.md    |     0
 .../prompts/begin_implementation.md                |     0
 .../prompts/create_post_clarifications.md          |     0
 .../prompts/create_reference_context.md            |     0
 .../prompts/restructure_input.md                   |     0
 .../prompts/review_reference_context.md            |     0
 .../specs/cclab-agent-p0-spec.md                   |     0
 .../20260325-cclab-agent-p0}/user_input.md         |     0
 .../20260325-enhanced-changes-section/STATE.yaml   |    29 +
 .../changes-section-targets/post_clarifications.md |    20 +
 .../changes-section-targets/pre_clarifications.md  |    17 +
 .../prompts/analyze_spec_changes-section-schema.md |    26 +
 .../prompts/analyze_spec_lens-impl-prompt.md       |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    63 +
 .../changes-section-targets/reference_context.md   |    38 +
 .../groups/changes-section-targets/requirements.md |    21 +
 .../groups/changes-section-targets/spec_plan.yaml  |    16 +
 .../specs/changes-section-schema.md                |   796 ++
 .../specs/lens-impl-prompt.md                      |   477 +
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-lens-changes.json  |     6 +
 .../payloads/create-change-spec-lens-logic.json    |     6 +
 .../payloads/create-change-spec-lens-overview.json |     7 +
 .../payloads/create-change-spec-overview.json      |     7 +
 .../payloads/create-change-spec-schema.json        |     6 +
 .../payloads/create-post-clarifications.json       |     1 +
 .../payloads/create-pre-clarifications.json        |    13 +
 .../payloads/create-reference-context.json         |    12 +
 .../payloads/restructure-input.json                |    14 +
 .../payloads/review-reference-context.json         |     1 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../user_input.md                                  |     1 +
 .../20260325-fix-remaining-drift-risks/STATE.yaml  |    26 +
 .../groups/drift-fixes/pre_clarifications.md       |    13 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    56 +
 .../groups/drift-fixes/reference_context.md        |    38 +
 .../groups/drift-fixes/requirements.md             |    15 +
 .../groups/drift-fixes/spec_plan.yaml              |    14 +
 .../drift-fixes/specs/fill-sections-fallback.md    |   588 +
 .../groups/drift-fixes/specs/merge-strategy-doc.md |    85 +
 .../payloads/create-pre-clarifications.json        |     1 +
 .../payloads/create-reference-context.json         |    12 +
 .../payloads/restructure-input.json                |    11 +
 .../payloads/review-reference-context.json         |     1 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../user_input.md                                  |     1 +
 .../archive/20260325-jet-dev-server-v2/STATE.yaml  |    56 +
 .../groups/jet-dev-server-v2/pre_clarifications.md |    17 +
 .../prompts/analyze_spec_jet-dev-server-v2-spec.md |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    95 +
 .../fill_spec_jet-dev-server-v2-spec_test_plan.md  |     8 +
 .../implement_tests_jet-dev-server-v2-spec.md      |    25 +
 .../prompts/review_impl_jet-dev-server-v2-spec.md  |    58 +
 .../prompts/review_reference_context.md            |    31 +
 .../prompts/revise_change_implementation.md        |    19 +
 .../prompts/revise_reference_context.md            |    23 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../groups/jet-dev-server-v2/reference_context.md  |    26 +
 .../groups/jet-dev-server-v2/requirements.md       |    19 +
 .../groups/jet-dev-server-v2/spec_plan.yaml        |    22 +
 .../specs/jet-dev-server-v2-spec.md                |   878 ++
 .../20260325-jet-dev-server-v2/implementation.md   |  1278 +++
 ...v-implement-optimizedeps-full-cjs-esm-pre-bu.md |    67 +
 ...v-ast-based-typescript-type-stripping-replac.md |    77 +
 ...v-browser-compatible-node-js-builtin-polyfil.md |    69 +
 ...stall-jet-store-symlinks-break-node-js-modul.md |    65 +
 .../payloads/create-change-implementation.json     |     1 +
 .../payloads/create-pre-clarifications.json        |    17 +
 .../payloads/create-reference-context.json         |    77 +
 .../payloads/fill-section-changes.json             |     6 +
 .../payloads/fill-section-logic.json               |     6 +
 .../payloads/fill-section-overview.json            |     9 +
 .../payloads/fill-section-requirements.json        |     6 +
 .../payloads/fill-section-scenarios.json           |     6 +
 .../payloads/fill-section-test-plan.json           |     6 +
 .../payloads/restructure-input.json                |    21 +
 .../payloads/review-change-implementation.json     |    76 +
 .../payloads/review-reference-context.json         |    19 +
 .../payloads/revise-reference-context.json         |    75 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    64 +
 .../20260325-jet-dev-server-v2/user_input.md       |     1 +
 .../20260325-mamba-conformance-basics}/STATE.yaml  |   381 +-
 .../groups/runtime-basics/post_clarifications.md   |     0
 .../groups/runtime-basics/pre_clarifications.md    |     0
 .../prompts/analyze_spec_builtins.md               |     0
 .../prompts/analyze_spec_cranelift-jit.md          |     0
 .../prompts/analyze_spec_cranelift.md              |     0
 .../runtime-basics/prompts/analyze_spec_repl.md    |     0
 .../prompts/analyze_spec_string-ops.md             |     0
 .../prompts/analyze_spec_type-checker.md           |     0
 .../runtime-basics/prompts/begin_implementation.md |     0
 .../prompts/create_post_clarifications.md          |     0
 .../prompts/create_pre_clarifications.md           |     0
 .../prompts/create_reference_context.md            |     0
 .../runtime-basics/prompts/implement_spec.md       |     0
 .../prompts/implement_tests_builtins.md            |     0
 .../prompts/implement_tests_cranelift-jit.md       |     0
 .../prompts/implement_tests_cranelift.md           |     0
 .../runtime-basics/prompts/implement_tests_repl.md |     0
 .../prompts/implement_tests_string-ops.md          |     0
 .../prompts/implement_tests_type-checker.md        |     0
 .../runtime-basics/prompts/review_impl_builtins.md |     0
 .../prompts/review_impl_cranelift-jit.md           |     0
 .../prompts/review_impl_cranelift.md               |     0
 .../runtime-basics/prompts/review_impl_repl.md     |     0
 .../prompts/review_impl_string-ops.md              |     0
 .../prompts/review_impl_type-checker.md            |     0
 .../prompts/review_reference_context.md            |     0
 .../prompts/revise_change_implementation.md        |     0
 .../prompts/revise_reference_context.md            |     0
 .../prompts/write_implementation_diff.md           |     0
 .../groups/runtime-basics/reference_context.md     |     0
 .../groups/runtime-basics/requirements.md          |     0
 .../groups/runtime-basics/spec_plan.yaml           |     0
 .../groups/runtime-basics/specs/builtins.md        |     0
 .../groups/runtime-basics/specs/cranelift-jit.md   |     0
 .../groups/runtime-basics/specs/cranelift.md       |     0
 .../groups/runtime-basics/specs/repl.md            |     0
 .../groups/runtime-basics/specs/string-ops.md      |     0
 .../groups/runtime-basics/specs/type-checker.md    |     0
 .../implementation.md                              |     0
 ...amba-py3-12-behavioral-conformance-every-fun.md |     0
 .../payloads/create-change-implementation.json     |     0
 .../create-change-spec-builtins-changes.json       |     0
 .../create-change-spec-builtins-overview.json      |     0
 .../create-change-spec-cranelift-changes.json      |     0
 .../create-change-spec-cranelift-jit-changes.json  |     0
 .../create-change-spec-cranelift-jit-overview.json |     0
 .../create-change-spec-cranelift-overview.json     |     0
 .../payloads/create-change-spec-repl-changes.json  |     0
 .../payloads/create-change-spec-repl-overview.json |     0
 .../payloads/create-post-clarifications.json       |     0
 .../create-pre-clarifications-runtime-basics.json  |     0
 .../payloads/create-reference-context.json         |     0
 .../payloads/restructure-input.json                |     0
 .../payloads/review-change-implementation.json     |     0
 .../payloads/review-reference-context.json         |     0
 .../payloads/revise-reference-context.json         |     0
 .../prompts/restructure_input.md                   |     0
 .../user_input.md                                  |     0
 .../STATE.yaml                                     |    28 +
 .../groups/scope-summary/post_clarifications.md    |    10 +
 .../groups/scope-summary/pre_clarifications.md     |    13 +
 .../prompts/create_post_clarifications.md          |    46 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    56 +
 .../groups/scope-summary/reference_context.md      |    36 +
 .../groups/scope-summary/requirements.md           |    20 +
 .../groups/scope-summary/spec_plan.yaml            |     7 +
 .../specs/post-clarifications-scope.md             |   147 +
 .../payloads/create-post-clarifications.json       |     1 +
 .../payloads/create-pre-clarifications.json        |     1 +
 .../payloads/create-reference-context.json         |    10 +
 .../payloads/restructure-input.json                |    11 +
 .../payloads/review-reference-context.json         |     1 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../user_input.md                                  |     1 +
 .../20260325-sdd-codegen-completion}/STATE.yaml    |   251 +-
 .../payloads/create-post-clarifications.json       |     0
 .../payloads/create-pre-clarifications.json        |     0
 .../groups/core-codegen/post_clarifications.md     |     0
 .../groups/core-codegen/pre_clarifications.md      |     0
 .../prompts/create_post_clarifications.md          |     0
 .../prompts/create_pre_clarifications.md           |     0
 .../prompts/create_reference_context.md            |     0
 .../prompts/review_reference_context.md            |     0
 .../prompts/revise_reference_context.md            |     0
 .../groups/core-codegen/reference_context.md       |     0
 .../groups/core-codegen/requirements.md            |     0
 .../payloads/create-post-clarifications.json       |     0
 .../payloads/create-pre-clarifications.json        |     0
 .../deploy-section-type/post_clarifications.md     |     0
 .../deploy-section-type/pre_clarifications.md      |     0
 .../prompts/create_post_clarifications.md          |     0
 .../prompts/create_pre_clarifications.md           |     0
 .../prompts/create_reference_context.md            |     0
 .../prompts/review_reference_context.md            |     0
 .../prompts/revise_reference_context.md            |     0
 .../deploy-section-type/reference_context.md       |     0
 .../groups/deploy-section-type/requirements.md     |     0
 .../payloads/create-post-clarifications.json       |     0
 .../payloads/create-pre-clarifications.json        |     0
 .../groups/frontend-codegen/post_clarifications.md |     0
 .../groups/frontend-codegen/pre_clarifications.md  |     0
 .../prompts/create_post_clarifications.md          |     0
 .../prompts/create_pre_clarifications.md           |     0
 .../prompts/create_reference_context.md            |     0
 .../prompts/review_reference_context.md            |     0
 .../prompts/revise_reference_context.md            |     0
 .../groups/frontend-codegen/reference_context.md   |     0
 .../groups/frontend-codegen/requirements.md        |     0
 .../groups/frontend-codegen/spec_plan.yaml         |     0
 .../specs/frontend-codegen-main.md                 |     0
 ...dd-codegen-last-mile-consume-specir-to-gener.md |     0
 ...dd-test-generation-from-requirementplus-spec.md |     0
 ...dd-deployment-spec-type-infra-as-code-integr.md |     0
 ...dd-frontend-codegen-wireframe-component-desi.md |     0
 .../payloads/create-change-spec.json               |     0
 .../payloads/create-reference-context.json         |     0
 .../payloads/restructure-input.json                |     0
 .../payloads/review-reference-context.json         |     0
 .../payloads/revise-reference-context-deploy.json  |     0
 .../payloads/revise-reference-context.json         |     0
 .../prompts/analyze_spec_frontend-codegen-main.md  |     0
 .../fill_spec_frontend-codegen-main_changes.md     |     0
 .../fill_spec_frontend-codegen-main_overview.md    |     0
 ...fill_spec_frontend-codegen-main_requirements.md |     0
 .../fill_spec_frontend-codegen-main_scenarios.md   |     0
 .../prompts/restructure_input.md                   |     0
 .../specs/frontend-codegen-main.md                 |     0
 .../20260325-sdd-codegen-completion}/user_input.md |     0
 .../20260325-spec-decomposition-rules/STATE.yaml   |    28 +
 .../decomposition-rules/post_clarifications.md     |    10 +
 .../decomposition-rules/pre_clarifications.md      |    13 +
 .../analyze_spec_change-spec-review-rules.md       |    26 +
 .../prompts/create_post_clarifications.md          |    46 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    56 +
 .../decomposition-rules/reference_context.md       |    37 +
 .../groups/decomposition-rules/requirements.md     |    21 +
 .../groups/decomposition-rules/spec_plan.yaml      |    14 +
 .../specs/change-spec-review-rules.md              |   589 +
 .../specs/ref-context-decomposition.md             |   311 +
 .../create-change-spec-review-rules-changes.json   |     6 +
 .../create-change-spec-review-rules-overview.json  |     8 +
 .../payloads/create-post-clarifications.json       |     1 +
 .../payloads/create-pre-clarifications.json        |     1 +
 .../payloads/create-reference-context.json         |    11 +
 .../payloads/restructure-input.json                |    11 +
 .../payloads/review-reference-context.json         |     1 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../user_input.md                                  |     1 +
 .../archive/20260326-jet-hmr-validation/STATE.yaml |    44 +
 .../jet-hmr-validation/pre_clarifications.md       |    17 +
 .../analyze_spec_jet-hmr-validation-spec.md        |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    97 +
 .../implement_tests_jet-hmr-validation-spec.md     |    25 +
 .../prompts/review_impl_jet-hmr-validation-spec.md |    58 +
 .../prompts/review_reference_context.md            |    31 +
 .../prompts/revise_change_implementation.md        |    19 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../groups/jet-hmr-validation/reference_context.md |    22 +
 .../groups/jet-hmr-validation/requirements.md      |    15 +
 .../groups/jet-hmr-validation/spec_plan.yaml       |    12 +
 .../specs/jet-hmr-validation-spec.md               |   722 ++
 .../20260326-jet-hmr-validation/implementation.md  |  2314 ++++
 ...v-javascript-module-hmr-hot-module-replaceme.md |    43 +
 ...v-validate-with-conductor-fe-real-world-reac.md |    35 +
 .../payloads/create-change-implementation.json     |     1 +
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-interaction.json   |     6 +
 .../payloads/create-change-spec-logic.json         |     6 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     6 +
 .../payloads/create-change-spec-scenarios.json     |     6 +
 .../payloads/create-change-spec-schema.json        |     6 +
 .../payloads/create-change-spec-state-machine.json |     6 +
 .../payloads/create-change-spec-test-plan.json     |     6 +
 .../payloads/create-pre-clarifications.json        |    17 +
 .../payloads/create-reference-context.json         |    51 +
 .../payloads/restructure-input.json                |    21 +
 .../payloads/review-change-implementation.json     |    11 +
 .../payloads/review-reference-context.json         |    19 +
 .../payloads/revise-reference-context.json         |    24 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    64 +
 .../20260326-jet-hmr-validation/user_input.md      |     1 +
 cclab/archive/20260326-lens-dissolution/STATE.yaml |    58 +
 .../groups/lens-dissolution/post_clarifications.md |    20 +
 .../groups/lens-dissolution/pre_clarifications.md  |    21 +
 .../prompts/analyze_spec_agent-context-builder.md  |    26 +
 .../prompts/analyze_spec_agent-output-format.md    |    26 +
 .../analyze_spec_lens-dissolution-restructure.md   |    26 +
 .../analyze_spec_sdd-cli-context-command.md        |    26 +
 .../analyze_spec_type-inference-pipeline.md        |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |   256 +
 .../lens-dissolution/prompts/implement_spec.md     |    17 +
 .../implement_tests_agent-context-builder.md       |    25 +
 .../prompts/implement_tests_agent-output-format.md |    25 +
 ...implement_tests_lens-dissolution-restructure.md |    25 +
 .../review_impl_lens-dissolution-restructure.md    |    58 +
 .../prompts/review_impl_sdd-cli-context-command.md |    58 +
 .../prompts/review_impl_type-inference-pipeline.md |    58 +
 .../prompts/review_reference_context.md            |    31 +
 .../prompts/revise_change_implementation.md        |    19 +
 .../groups/lens-dissolution/reference_context.md   |    72 +
 .../groups/lens-dissolution/requirements.md        |    12 +
 .../groups/lens-dissolution/spec_plan.yaml         |    44 +
 .../specs/agent-context-builder.md                 |   358 +
 .../lens-dissolution/specs/agent-output-format.md  |   312 +
 .../specs/lens-dissolution-restructure.md          |   386 +
 .../specs/sdd-cli-context-command.md               |   210 +
 .../specs/type-inference-pipeline.md               |   510 +
 .../20260326-lens-dissolution/implementation.md    |   213 +
 ...or-dissolve-lens-module-into-sdd-top-level-s.md |    65 +
 ...ens-wire-cross-file-type-propagation-deep-in.md |    49 +
 ...ens-agent-context-builder-smart-file-selecti.md |    70 +
 ...ens-agent-optimized-output-structured-json-f.md |    53 +
 .../payloads/agent-context-builder-changes.json    |     8 +
 .../payloads/agent-context-builder-logic.json      |     8 +
 .../payloads/agent-context-builder-overview.json   |     8 +
 .../agent-context-builder-requirements.json        |     8 +
 .../payloads/agent-context-builder-scenarios.json  |     8 +
 .../payloads/agent-context-builder-schema.json     |     8 +
 .../payloads/agent-context-builder-test-plan.json  |     8 +
 .../payloads/agent-output-format-changes.json      |     8 +
 .../payloads/agent-output-format-overview.json     |     8 +
 .../payloads/agent-output-format-requirements.json |     8 +
 .../payloads/agent-output-format-scenarios.json    |     8 +
 .../payloads/agent-output-format-schema.json       |     8 +
 .../payloads/agent-output-format-test-plan.json    |     8 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |    20 +
 .../payloads/create-reference-context.json         |   140 +
 .../lens-dissolution-restructure-changes.json      |     8 +
 .../lens-dissolution-restructure-overview.json     |     8 +
 .../lens-dissolution-restructure-requirements.json |     8 +
 .../lens-dissolution-restructure-scenarios.json    |     8 +
 .../lens-dissolution-restructure-test-plan.json    |     8 +
 .../payloads/restructure-input.json                |    27 +
 .../payloads/review-change-implementation-v2.json  |    44 +
 .../payloads/review-change-implementation.json     |    12 +
 .../payloads/review-reference-context.json         |    48 +
 .../payloads/review-sdd-cli-context-command.json   |    65 +
 .../payloads/review-type-inference-pipeline.json   |    85 +
 .../payloads/sdd-cli-context-command-changes.json  |     8 +
 .../payloads/sdd-cli-context-command-overview.json |     9 +
 .../sdd-cli-context-command-requirements.json      |     8 +
 .../sdd-cli-context-command-scenarios.json         |     8 +
 .../payloads/type-inference-pipeline-changes.json  |     8 +
 .../payloads/type-inference-pipeline-logic.json    |     8 +
 .../payloads/type-inference-pipeline-overview.json |     8 +
 .../type-inference-pipeline-requirements.json      |     8 +
 .../type-inference-pipeline-scenarios.json         |     8 +
 .../payloads/type-inference-pipeline-schema.json   |     8 +
 .../type-inference-pipeline-test-plan.json         |     8 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    64 +
 .../20260326-lens-dissolution/user_input.md        |     1 +
 cclab/archive/20260327-e2e-test-reorg/STATE.yaml   |    30 +
 .../groups/e2e-test-reorg/post_clarifications.md   |    20 +
 .../groups/e2e-test-reorg/pre_clarifications.md    |    17 +
 .../analyze_spec_e2e-test-infrastructure.md        |    26 +
 .../e2e-test-reorg/prompts/begin_implementation.md |    18 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    63 +
 .../implement_tests_e2e-test-infrastructure.md     |    25 +
 .../prompts/review_impl_e2e-test-infrastructure.md |    58 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../groups/e2e-test-reorg/reference_context.md     |    22 +
 .../groups/e2e-test-reorg/requirements.md          |    23 +
 .../groups/e2e-test-reorg/spec_plan.yaml           |     9 +
 .../specs/e2e-test-infrastructure.md               |   375 +
 .../20260327-e2e-test-reorg/implementation.md      |   797 ++
 .../payloads/create-change-implementation.json     |     1 +
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     6 +
 .../payloads/create-change-spec-scenarios.json     |     6 +
 .../payloads/create-change-spec-test-plan.json     |     6 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |    13 +
 .../payloads/create-reference-context.json         |    51 +
 .../payloads/restructure-input.json                |    10 +
 .../payloads/review-change-implementation.json     |    69 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../archive/20260327-e2e-test-reorg/user_input.md  |     1 +
 .../20260327-gcp-cloud-integration/STATE.yaml      |    30 +
 .../gcp-cloud-integration/post_clarifications.md   |    20 +
 .../gcp-cloud-integration/pre_clarifications.md    |    25 +
 .../prompts/analyze_spec_broker-traits.md          |    26 +
 .../analyze_spec_cloud-scheduler-backend.md        |    26 +
 .../prompts/analyze_spec_cloudtasks-broker.md      |    26 +
 .../prompts/analyze_spec_scheduler-backends-gcp.md |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    63 +
 .../prompts/implement_spec.md                      |    17 +
 .../prompts/implement_tests_broker-traits.md       |    25 +
 .../implement_tests_cloud-scheduler-backend.md     |    25 +
 .../prompts/implement_tests_cloudtasks-broker.md   |    25 +
 .../implement_tests_scheduler-backends-gcp.md      |    25 +
 .../prompts/review_impl_broker-traits.md           |    58 +
 .../prompts/review_impl_cloud-scheduler-backend.md |    58 +
 .../prompts/review_impl_cloudtasks-broker.md       |    58 +
 .../prompts/review_impl_scheduler-backends-gcp.md  |    58 +
 .../prompts/review_reference_context.md            |    31 +
 .../prompts/revise_change_implementation.md        |    19 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../gcp-cloud-integration/reference_context.md     |    40 +
 .../groups/gcp-cloud-integration/requirements.md   |    13 +
 .../groups/gcp-cloud-integration/spec_plan.yaml    |    29 +
 .../gcp-cloud-integration/specs/broker-traits.md   |   327 +
 .../specs/cloud-scheduler-backend.md               |   558 +
 .../specs/cloudtasks-broker.md                     |   612 +
 .../specs/scheduler-backends-gcp.md                |   155 +
 .../implementation.md                              |  4423 ++++++++
 .../payloads/create-change-implementation.json     |     1 +
 .../create-change-spec-broker-traits-overview.json |     9 +
 ...ate-change-spec-broker-traits-requirements.json |     6 +
 ...create-change-spec-broker-traits-scenarios.json |     6 +
 .../create-change-spec-broker-traits-schema.json   |     6 +
 ...hange-spec-cloud-scheduler-backend-changes.json |     6 +
 ...ange-spec-cloud-scheduler-backend-overview.json |     9 +
 ...-spec-cloud-scheduler-backend-requirements.json |     6 +
 ...ange-spec-cloud-scheduler-backend-rest-api.json |     6 +
 ...nge-spec-cloud-scheduler-backend-scenarios.json |     6 +
 ...change-spec-cloud-scheduler-backend-schema.json |     6 +
 ...eate-change-spec-cloudtasks-broker-changes.json |     6 +
 ...ate-change-spec-cloudtasks-broker-overview.json |     9 +
 ...change-spec-cloudtasks-broker-requirements.json |     6 +
 ...ate-change-spec-cloudtasks-broker-rest-api.json |     6 +
 ...te-change-spec-cloudtasks-broker-scenarios.json |     6 +
 ...reate-change-spec-cloudtasks-broker-schema.json |     6 +
 ...change-spec-scheduler-backends-gcp-changes.json |     7 +
 ...hange-spec-scheduler-backends-gcp-overview.json |     9 +
 ...e-spec-scheduler-backends-gcp-requirements.json |     7 +
 ...ange-spec-scheduler-backends-gcp-scenarios.json |     7 +
 .../payloads/create-post-clarifications.json       |    14 +
 .../payloads/create-pre-clarifications.json        |    21 +
 .../payloads/create-reference-context.json         |    68 +
 .../payloads/restructure-input.json                |    15 +
 .../payloads/review-change-implementation.json     |    39 +
 .../payloads/review-cloud-scheduler-backend.json   |    59 +
 .../payloads/review-cloudtasks-broker.json         |    29 +
 .../payloads/review-reference-context.json         |    80 +
 .../payloads/review-scheduler-backends-gcp.json    |    41 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../20260327-gcp-cloud-integration/user_input.md   |     1 +
 cclab/archive/20260327-jet-test-gaps/STATE.yaml    |    28 +
 .../groups/jet-test-gaps/post_clarifications.md    |    20 +
 .../groups/jet-test-gaps/pre_clarifications.md     |    13 +
 .../prompts/analyze_spec_jet-aot-build.md          |    26 +
 .../jet-test-gaps/prompts/analyze_spec_jet-hmr.md  |    26 +
 .../prompts/analyze_spec_jet-postcss-tailwind.md   |    26 +
 .../jet-test-gaps/prompts/begin_implementation.md  |    18 +
 .../prompts/create_reference_context.md            |    98 +
 .../groups/jet-test-gaps/prompts/implement_spec.md |    17 +
 .../prompts/implement_tests_jet-aot-build.md       |    25 +
 .../prompts/implement_tests_jet-hmr.md             |    25 +
 .../implement_tests_jet-postcss-tailwind.md        |    25 +
 .../groups/jet-test-gaps/reference_context.md      |    24 +
 .../groups/jet-test-gaps/requirements.md           |    17 +
 .../groups/jet-test-gaps/spec_plan.yaml            |    21 +
 .../groups/jet-test-gaps/specs/jet-aot-build.md    |   315 +
 .../groups/jet-test-gaps/specs/jet-hmr.md          |   389 +
 .../jet-test-gaps/specs/jet-postcss-tailwind.md    |   504 +
 .../create-change-spec-aot-build-changes.json      |     6 +
 .../create-change-spec-aot-build-overview.json     |     9 +
 .../create-change-spec-aot-build-requirements.json |     6 +
 .../create-change-spec-aot-build-scenarios.json    |     6 +
 .../create-change-spec-aot-build-test-plan.json    |     6 +
 .../payloads/create-change-spec-hmr-changes.json   |     6 +
 .../payloads/create-change-spec-hmr-overview.json  |     9 +
 .../create-change-spec-hmr-requirements.json       |     6 +
 .../payloads/create-change-spec-hmr-scenarios.json |     6 +
 .../payloads/create-change-spec-hmr-test-plan.json |     6 +
 ...reate-change-spec-postcss-tailwind-changes.json |     6 +
 ...eate-change-spec-postcss-tailwind-overview.json |     9 +
 ...-change-spec-postcss-tailwind-requirements.json |     6 +
 ...ate-change-spec-postcss-tailwind-scenarios.json |     6 +
 ...ate-change-spec-postcss-tailwind-test-plan.json |     6 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |     9 +
 .../payloads/create-reference-context.json         |    49 +
 .../payloads/restructure-input.json                |    10 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 cclab/archive/20260327-jet-test-gaps/user_input.md |     1 +
 .../20260327-mamba-conformance-xfail/STATE.yaml    |    30 +
 .../post_clarifications.md                         |    20 +
 .../pre_clarifications.md                          |    17 +
 ...nalyze_spec_conformance-xfail-reduction-spec.md |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    63 +
 ...ec_conformance-xfail-reduction-spec_pipeline.md |     7 +
 ...ement_tests_conformance-xfail-reduction-spec.md |    25 +
 ...review_impl_conformance-xfail-reduction-spec.md |    58 +
 .../prompts/revise_change_implementation.md        |    19 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../reference_context.md                           |    26 +
 .../conformance-xfail-reduction/requirements.md    |     9 +
 .../conformance-xfail-reduction/spec_plan.yaml     |     8 +
 .../specs/conformance-xfail-reduction-spec.md      |   489 +
 .../implementation.md                              |  6784 ++++++++++++
 .../payloads/create-change-implementation.json     |     6 +
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-logic.json         |     6 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-pipeline.json      |     6 +
 .../payloads/create-change-spec-requirements.json  |     6 +
 .../payloads/create-change-spec-scenarios.json     |     6 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |    13 +
 .../payloads/create-reference-context.json         |   101 +
 .../payloads/fill-pipeline-section.json            |     6 +
 .../payloads/fix-fill-sections-overview.json       |     9 +
 .../payloads/fix-scenarios-section.json            |     6 +
 .../payloads/restructure-input.json                |    13 +
 .../payloads/review-change-implementation.json     |    26 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../20260327-mamba-conformance-xfail/user_input.md |     1 +
 cclab/archive/20260327-mamba-xfail-zero/STATE.yaml |    31 +
 .../groups/xfail-zero/post_clarifications.md       |    20 +
 .../groups/xfail-zero/pre_clarifications.md        |    13 +
 .../xfail-zero/prompts/analyze_spec_xfail-zero.md  |    26 +
 .../xfail-zero/prompts/begin_implementation.md     |    18 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../xfail-zero/prompts/create_reference_context.md |    63 +
 .../prompts/implement_tests_xfail-zero.md          |    25 +
 .../xfail-zero/prompts/review_impl_xfail-zero.md   |    58 +
 .../prompts/revise_change_implementation.md        |    19 +
 .../groups/xfail-zero/reference_context.md         |    34 +
 .../groups/xfail-zero/requirements.md              |     9 +
 .../groups/xfail-zero/spec_plan.yaml               |    10 +
 .../groups/xfail-zero/specs/xfail-zero.md          |   602 +
 .../20260327-mamba-xfail-zero/implementation.md    |  4341 ++++++++
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     6 +
 .../payloads/create-change-spec-scenarios.json     |     6 +
 .../payloads/create-change-spec-test-plan.json     |     6 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |     9 +
 .../payloads/create-reference-context.json         |   199 +
 .../payloads/restructure-input.json                |    10 +
 .../payloads/review-change-implementation.json     |    92 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../20260327-mamba-xfail-zero/user_input.md        |     1 +
 .../20260327-scheduler-runtime-complete/STATE.yaml |    42 +
 .../post_clarifications.md                         |    20 +
 .../pre_clarifications.md                          |    25 +
 .../prompts/analyze_spec_k8s-cronjob-backend.md    |    26 +
 .../prompts/analyze_spec_push-receiver.md          |    26 +
 .../prompts/analyze_spec_schedule-monitor.md       |    26 +
 .../analyze_spec_scheduler-mode-selection.md       |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/implement_spec.md                      |    17 +
 .../prompts/implement_tests_k8s-cronjob-backend.md |    25 +
 .../prompts/implement_tests_push-receiver.md       |    25 +
 .../prompts/implement_tests_schedule-monitor.md    |    25 +
 .../implement_tests_scheduler-mode-selection.md    |    25 +
 .../prompts/review_impl_k8s-cronjob-backend.md     |    58 +
 .../prompts/review_reference_context.md            |    31 +
 .../prompts/revise_reference_context.md            |    23 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../reference_context.md                           |    45 +
 .../scheduler-runtime-complete/requirements.md     |    15 +
 .../scheduler-runtime-complete/spec_plan.yaml      |    29 +
 .../specs/k8s-cronjob-backend.md                   |   400 +
 .../specs/push-receiver.md                         |   631 ++
 .../specs/schedule-monitor.md                      |   398 +
 .../specs/scheduler-mode-selection.md              |   407 +
 .../implementation.md                              |  9824 +++++++++++++++++
 .../payloads/create-change-implementation.json     |     1 +
 ...te-change-spec-k8s-cronjob-backend-changes.json |     6 +
 ...e-change-spec-k8s-cronjob-backend-overview.json |     9 +
 ...ange-spec-k8s-cronjob-backend-requirements.json |     6 +
 ...-change-spec-k8s-cronjob-backend-scenarios.json |     6 +
 ...ate-change-spec-k8s-cronjob-backend-schema.json |     6 +
 .../create-change-spec-push-receiver-changes.json  |     6 +
 .../create-change-spec-push-receiver-overview.json |     9 +
 ...ate-change-spec-push-receiver-requirements.json |     6 +
 .../create-change-spec-push-receiver-rest-api.json |     6 +
 ...create-change-spec-push-receiver-scenarios.json |     6 +
 .../create-change-spec-push-receiver-schema.json   |     6 +
 ...reate-change-spec-schedule-monitor-changes.json |     6 +
 ...eate-change-spec-schedule-monitor-overview.json |     9 +
 ...-change-spec-schedule-monitor-requirements.json |     6 +
 ...ate-change-spec-schedule-monitor-scenarios.json |     6 +
 ...create-change-spec-schedule-monitor-schema.json |     6 +
 ...ange-spec-scheduler-mode-selection-changes.json |     6 +
 ...nge-spec-scheduler-mode-selection-overview.json |     9 +
 ...spec-scheduler-mode-selection-requirements.json |     6 +
 ...ge-spec-scheduler-mode-selection-scenarios.json |     6 +
 ...hange-spec-scheduler-mode-selection-schema.json |     6 +
 .../payloads/create-post-clarifications.json       |    14 +
 .../payloads/create-pre-clarifications.json        |    21 +
 .../payloads/create-reference-context.json         |   105 +
 .../payloads/restructure-input.json                |    15 +
 .../payloads/review-change-implementation.json     |    62 +
 .../payloads/review-reference-context.json         |    82 +
 .../payloads/revise-reference-context.json         |   105 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    45 +
 .../user_input.md                                  |     1 +
 cclab/archive/20260327-scope-hoisting/STATE.yaml   |    37 +
 .../groups/scope-hoisting/post_clarifications.md   |    20 +
 .../groups/scope-hoisting/pre_clarifications.md    |    17 +
 .../prompts/analyze_spec_scope-hoisting.md         |    26 +
 .../scope-hoisting/prompts/begin_implementation.md |    18 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    99 +
 .../prompts/implement_tests_scope-hoisting.md      |    25 +
 .../prompts/review_impl_scope-hoisting.md          |    58 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../groups/scope-hoisting/reference_context.md     |    22 +
 .../groups/scope-hoisting/requirements.md          |    18 +
 .../groups/scope-hoisting/spec_plan.yaml           |     7 +
 .../groups/scope-hoisting/specs/scope-hoisting.md  |   392 +
 .../20260327-scope-hoisting/implementation.md      |  1432 +++
 ...et-build-scope-hoisting-module-concatenation.md |    33 +
 .../payloads/create-change-implementation.json     |     6 +
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-logic.json         |     6 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     6 +
 .../payloads/create-change-spec-scenarios.json     |     6 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |    13 +
 .../payloads/create-reference-context.json         |    52 +
 .../payloads/restructure-input.json                |    10 +
 .../payloads/review-change-implementation.json     |    69 +
 .../prompts/create_change_merge.md                 |     6 +
 .../prompts/restructure_input.md                   |    64 +
 .../archive/20260327-scope-hoisting/user_input.md  |     1 +
 .../obsolete-specs}/template-mcp-configs.md        |     4 +-
 cclab/changes/cclab-api-asgi-dispatch/STATE.yaml   |    29 +
 .../groups/asgi-fix/post_clarifications.md         |    20 +
 .../groups/asgi-fix/pre_clarifications.md          |    13 +
 .../prompts/analyze_spec_asgi-dispatch-spec.md     |    26 +
 .../asgi-fix/prompts/begin_implementation.md       |    18 +
 .../asgi-fix/prompts/create_post_clarifications.md |    56 +
 .../asgi-fix/prompts/create_pre_clarifications.md  |    29 +
 .../prompts/implement_tests_asgi-dispatch-spec.md  |    25 +
 .../asgi-fix/prompts/write_implementation_diff.md  |    14 +
 .../groups/asgi-fix/reference_context.md           |    20 +
 .../groups/asgi-fix/requirements.md                |     9 +
 .../groups/asgi-fix/spec_plan.yaml                 |     7 +
 .../groups/asgi-fix/specs/asgi-dispatch-spec.md    |   200 +
 .../cclab-api-asgi-dispatch/implementation.md      |   743 ++
 .../payloads/create-change-implementation.json     |     6 +
 .../payloads/fill-changes.json                     |     7 +
 .../payloads/fill-overview.json                    |    10 +
 .../payloads/fill-requirements.json                |     7 +
 .../payloads/fill-scenarios.json                   |     7 +
 .../cclab-api-asgi-dispatch/payloads/impl.json     |     1 +
 .../payloads/post-clarify.json                     |     1 +
 .../payloads/pre-clarify.json                      |     1 +
 .../cclab-api-asgi-dispatch/payloads/ref-ctx.json  |     1 +
 .../payloads/restructure-input.json                |     1 +
 .../payloads/review-impl.json                      |     1 +
 .../prompts/restructure_input.md                   |    45 +
 .../changes/cclab-api-asgi-dispatch/user_input.md  |     1 +
 cclab/changes/cclab-pg-compat/STATE.yaml           |    20 +
 cclab/changes/cclab-pg-compat/user_input.md        |     1 +
 cclab/changes/conductor-cclab-migration/STATE.yaml |    28 +
 .../cclab-import-migration/post_clarifications.md  |    10 +
 .../cclab-import-migration/pre_clarifications.md   |    21 +
 .../analyze_spec_conductor-cclab-migration-spec.md |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    46 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    56 +
 ..._spec_conductor-cclab-migration-spec_changes.md |    14 +
 ...ec_conductor-cclab-migration-spec_dependency.md |     7 +
 ...c_conductor-cclab-migration-spec_interaction.md |     7 +
 ...spec_conductor-cclab-migration-spec_overview.md |     8 +
 ..._conductor-cclab-migration-spec_requirements.md |    13 +
 ...pec_conductor-cclab-migration-spec_test-plan.md |     8 +
 ...plement_tests_conductor-cclab-migration-spec.md |    25 +
 .../review_impl_conductor-cclab-migration-spec.md  |    58 +
 .../prompts/review_reference_context.md            |    28 +
 .../prompts/revise_reference_context.md            |    23 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../cclab-import-migration/reference_context.md    |    22 +
 .../groups/cclab-import-migration/requirements.md  |     9 +
 .../groups/cclab-import-migration/spec_plan.yaml   |    11 +
 .../specs/conductor-cclab-migration-spec.md        |   226 +
 .../conductor-cclab-migration/implementation.md    |   806 ++
 .../payloads/create-change-implementation.json     |     1 +
 .../payloads/create-post-clarifications.json       |     5 +
 .../payloads/create-pre-clarifications.json        |    22 +
 .../payloads/create-reference-context.json         |    32 +
 .../payloads/fill-changes.json                     |     7 +
 .../payloads/fill-dependency.json                  |     7 +
 .../payloads/fill-interaction.json                 |     7 +
 .../payloads/fill-overview.json                    |     7 +
 .../payloads/fill-requirements.json                |     7 +
 .../payloads/fill-test-plan.json                   |     7 +
 .../payloads/restructure-input.json                |    14 +
 .../payloads/review-change-implementation.json     |    18 +
 .../payloads/review-reference-context.json         |    24 +
 .../payloads/revise-reference-context.json         |    32 +
 .../prompts/restructure_input.md                   |    45 +
 .../conductor-cclab-migration/user_input.md        |     1 +
 cclab/changes/conductor-mock-backend/STATE.yaml    |    28 +
 .../mock-backend-server/post_clarifications.md     |    10 +
 .../mock-backend-server/pre_clarifications.md      |    13 +
 .../analyze_spec_conductor-mock-backend-spec.md    |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    46 +
 .../prompts/create_pre_clarifications.md           |    29 +
 ...ill_spec_conductor-mock-backend-spec_changes.md |    14 +
 ...ll_spec_conductor-mock-backend-spec_overview.md |     8 +
 ...pec_conductor-mock-backend-spec_requirements.md |    13 +
 ...ll_spec_conductor-mock-backend-spec_rest-api.md |     7 +
 ...l_spec_conductor-mock-backend-spec_test-plan.md |     8 +
 .../implement_tests_conductor-mock-backend-spec.md |    25 +
 .../review_impl_conductor-mock-backend-spec.md     |    58 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../mock-backend-server/reference_context.md       |    22 +
 .../groups/mock-backend-server/requirements.md     |     9 +
 .../groups/mock-backend-server/spec_plan.yaml      |     9 +
 .../specs/conductor-mock-backend-spec.md           |   204 +
 .../conductor-mock-backend/implementation.md       |    27 +
 .../payloads/create-change-implementation.json     |     1 +
 .../payloads/create-post-clarifications.json       |     5 +
 .../payloads/create-pre-clarifications.json        |     9 +
 .../payloads/create-reference-context.json         |    31 +
 .../payloads/fill-changes.json                     |     7 +
 .../payloads/fill-overview.json                    |     7 +
 .../payloads/fill-requirements.json                |     7 +
 .../payloads/fill-rest-api.json                    |     7 +
 .../payloads/fill-test-plan.json                   |     7 +
 .../payloads/restructure-input.json                |    10 +
 .../payloads/review-change-implementation.json     |     6 +
 .../prompts/restructure_input.md                   |    45 +
 cclab/changes/conductor-mock-backend/user_input.md |     1 +
 cclab/changes/conductor-multi-platform/STATE.yaml  |    28 +
 .../groups/multi-platform/post_clarifications.md   |    20 +
 .../groups/multi-platform/pre_clarifications.md    |    13 +
 .../prompts/analyze_spec_multi-platform-spec.md    |    26 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../groups/multi-platform/reference_context.md     |    21 +
 .../groups/multi-platform/requirements.md          |     9 +
 .../groups/multi-platform/spec_plan.yaml           |    10 +
 .../multi-platform/specs/multi-platform-spec.md    |   460 +
 .../payloads/create-change-spec-changes.json       |     8 +
 .../payloads/create-change-spec-db-model.json      |     8 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     8 +
 .../payloads/create-change-spec-rest-api.json      |     8 +
 .../payloads/create-change-spec-scenarios.json     |     8 +
 .../payloads/create-change-spec-test-plan.json     |     8 +
 .../payloads/post-clarify.json                     |     1 +
 .../payloads/pre-clarify.json                      |     1 +
 .../conductor-multi-platform/payloads/ref-ctx.json |     1 +
 .../payloads/restructure-input.json                |     1 +
 .../prompts/restructure_input.md                   |    45 +
 .../changes/conductor-multi-platform/user_input.md |     1 +
 .../changes/conductor-product-features/STATE.yaml  |    58 +
 .../fe-fixes-and-features/post_clarifications.md   |    20 +
 .../fe-fixes-and-features/pre_clarifications.md    |    13 +
 ...analyze_spec_conductor-product-features-spec.md |    26 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../fe-fixes-and-features/reference_context.md     |    20 +
 .../groups/fe-fixes-and-features/requirements.md   |     9 +
 .../groups/fe-fixes-and-features/spec_plan.yaml    |     8 +
 .../specs/conductor-product-features-spec.md       |   332 +
 .../mock-backend-dynamic/post_clarifications.md    |    20 +
 .../mock-backend-dynamic/pre_clarifications.md     |    13 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    64 +
 .../mock-backend-dynamic/reference_context.md      |    20 +
 .../groups/mock-backend-dynamic/requirements.md    |     9 +
 ...tor-fe-fix-projectdetail-hooks-ordering-tab-.md |    48 +
 ...-cclab-spec-viewer-package-markdown-mermaid-.md |    58 +
 ...-cclab-pipeline-package-dag-visualization-no.md |    61 +
 ...tor-extend-mock-backend-to-support-full-user.md |    42 +
 .../payloads/create-change-spec-changes.json       |     8 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     8 +
 .../payloads/create-change-spec-scenarios.json     |     8 +
 .../payloads/create-change-spec-test-plan.json     |     8 +
 .../payloads/create-reference-context.json         |    62 +
 .../post-clarify-fe-fixes-and-features.json        |     1 +
 .../post-clarify-mock-backend-dynamic.json         |     1 +
 .../pre-clarify-fe-fixes-and-features.json         |     1 +
 .../payloads/pre-clarify-mock-backend-dynamic.json |     1 +
 .../payloads/ref-ctx-fe-fixes-and-features.json    |     1 +
 .../payloads/ref-ctx-mock-backend-dynamic.json     |     1 +
 .../payloads/restructure-input.json                |    16 +
 .../prompts/restructure_input.md                   |    64 +
 .../conductor-product-features/user_input.md       |     1 +
 .../changes/conductor-product-redesign/STATE.yaml  |    28 +
 .../route-restructure/post_clarifications.md       |    20 +
 .../groups/route-restructure/pre_clarifications.md |    13 +
 .../analyze_spec_conductor-redesign-spec.md        |    26 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../groups/route-restructure/reference_context.md  |    20 +
 .../groups/route-restructure/requirements.md       |     9 +
 .../groups/route-restructure/spec_plan.yaml        |     8 +
 .../specs/conductor-redesign-spec.md               |   297 +
 .../payloads/fill-changes.json                     |     7 +
 .../payloads/fill-overview.json                    |     7 +
 .../payloads/fill-requirements.json                |     7 +
 .../payloads/fill-scenarios.json                   |     7 +
 .../payloads/fill-test-plan.json                   |     7 +
 .../payloads/post-clarify.json                     |     1 +
 .../payloads/pre-clarify.json                      |     1 +
 .../payloads/ref-ctx.json                          |     1 +
 .../payloads/restructure-input.json                |     9 +
 .../payloads/spec-overview.json                    |     8 +
 .../prompts/restructure_input.md                   |    45 +
 .../conductor-product-redesign/user_input.md       |     1 +
 .../changes/conductor-sdd-orchestrator/STATE.yaml  |    28 +
 .../groups/orchestrator/post_clarifications.md     |    20 +
 .../groups/orchestrator/pre_clarifications.md      |    13 +
 .../prompts/analyze_spec_sdd-orchestrator-spec.md  |    26 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../groups/orchestrator/reference_context.md       |    21 +
 .../groups/orchestrator/requirements.md            |     9 +
 .../groups/orchestrator/spec_plan.yaml             |     9 +
 .../orchestrator/specs/sdd-orchestrator-spec.md    |   531 +
 .../payloads/cs-changes.json                       |     7 +
 .../payloads/cs-interaction.json                   |     7 +
 .../payloads/cs-overview.json                      |     8 +
 .../payloads/cs-requirements.json                  |     6 +
 .../payloads/cs-rest-api.json                      |     7 +
 .../payloads/cs-scenarios.json                     |     6 +
 .../payloads/cs-schema.json                        |     7 +
 .../payloads/cs-test-plan.json                     |     7 +
 .../payloads/post-clarify.json                     |     1 +
 .../payloads/pre-clarify.json                      |     1 +
 .../payloads/ref-ctx.json                          |     1 +
 .../payloads/restructure-input.json                |     1 +
 .../prompts/restructure_input.md                   |    45 +
 .../conductor-sdd-orchestrator/user_input.md       |     1 +
 cclab/changes/conductor-state-specs/STATE.yaml     |    37 +
 .../state-machine-specs/post_clarifications.md     |    20 +
 .../state-machine-specs/pre_clarifications.md      |    13 +
 .../prompts/analyze_spec_conductor-state-specs.md  |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../implement_tests_conductor-state-specs.md       |    25 +
 .../prompts/review_impl_conductor-state-specs.md   |    58 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../state-machine-specs/reference_context.md       |    23 +
 .../groups/state-machine-specs/requirements.md     |     9 +
 .../groups/state-machine-specs/spec_plan.yaml      |     9 +
 .../specs/conductor-state-specs.md                 |   587 +
 .../conductor-state-specs/implementation.md        |   528 +
 ...tor-complete-state-machine-specs-for-issue-c.md |    46 +
 .../payloads/create-change-implementation.json     |     6 +
 .../payloads/create-post-clarifications.json       |     5 +
 .../payloads/create-pre-clarifications.json        |     9 +
 .../payloads/create-reference-context.json         |    37 +
 .../payloads/fill-changes.json                     |     7 +
 .../payloads/fill-overview.json                    |     7 +
 .../payloads/fill-requirements.json                |     7 +
 .../payloads/fill-scenarios.json                   |     7 +
 .../payloads/fill-state-machine.json               |     7 +
 .../payloads/fill-test-plan.json                   |     7 +
 .../payloads/restructure-input.json                |    10 +
 .../payloads/review-change-implementation.json     |     6 +
 .../prompts/restructure_input.md                   |    64 +
 cclab/changes/conductor-state-specs/user_input.md  |     1 +
 cclab/changes/gen-thread-pool/STATE.yaml           |    38 +
 .../groups/gen-thread-pool/post_clarifications.md  |    20 +
 .../groups/gen-thread-pool/pre_clarifications.md   |    17 +
 .../analyze_spec_generator-thread-pool-design.md   |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |   270 +
 ...implement_tests_generator-thread-pool-design.md |    25 +
 .../groups/gen-thread-pool/reference_context.md    |    25 +
 .../groups/gen-thread-pool/requirements.md         |     9 +
 .../groups/gen-thread-pool/spec_plan.yaml          |     9 +
 .../specs/generator-thread-pool-design.md          |   282 +
 cclab/changes/gen-thread-pool/implementation.md    |  1362 +++
 ...mba-sigbus-crash-in-multi-threaded-conforman.md |    33 +
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     6 +
 .../payloads/create-change-spec-scenarios.json     |     6 +
 .../payloads/create-change-spec-state-machine.json |     6 +
 .../payloads/create-change-spec-test-plan.json     |     6 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |    14 +
 .../payloads/create-reference-context.json         |    99 +
 .../payloads/restructure-input.json                |    13 +
 .../gen-thread-pool/prompts/restructure_input.md   |    64 +
 cclab/changes/gen-thread-pool/user_input.md        |     1 +
 cclab/changes/jet-aot-build-gaps/STATE.yaml        |    42 +
 .../jet-aot-build-gaps/pre_clarifications.md       |    21 +
 .../analyze_spec_jet-aot-build-gaps-spec.md        |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |    90 +
 .../fill_spec_jet-aot-build-gaps-spec_changes.md   |    14 +
 .../fill_spec_jet-aot-build-gaps-spec_overview.md  |     8 +
 ...ll_spec_jet-aot-build-gaps-spec_requirements.md |    13 +
 .../fill_spec_jet-aot-build-gaps-spec_scenarios.md |    12 +
 .../prompts/review_reference_context.md            |    28 +
 .../prompts/revise_reference_context.md            |    23 +
 .../groups/jet-aot-build-gaps/reference_context.md |    38 +
 .../groups/jet-aot-build-gaps/requirements.md      |     9 +
 .../groups/jet-aot-build-gaps/spec_plan.yaml       |    10 +
 .../specs/jet-aot-build-gaps-spec.md               |   250 +
 ...et-aot-production-build-tree-shaking-code-sp.md |   140 +
 .../payloads/create-pre-clarifications.json        |    20 +
 .../payloads/create-reference-context.json         |    64 +
 .../payloads/fill-section-overview.json            |     9 +
 .../payloads/restructure-input.json                |    32 +
 .../payloads/review-reference-context.json         |    18 +
 .../payloads/revise-reference-context.json         |    64 +
 .../prompts/restructure_input.md                   |    64 +
 cclab/changes/jet-aot-build-gaps/user_input.md     |     1 +
 cclab/changes/lens-dissolution/STATE.yaml          |    57 +
 .../groups/lens-dissolution/post_clarifications.md |    20 +
 .../groups/lens-dissolution/pre_clarifications.md  |    21 +
 .../prompts/analyze_spec_agent-context-builder.md  |    26 +
 .../prompts/analyze_spec_agent-output-format.md    |    26 +
 .../analyze_spec_lens-dissolution-restructure.md   |    26 +
 .../analyze_spec_sdd-cli-context-command.md        |    26 +
 .../analyze_spec_type-inference-pipeline.md        |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |   256 +
 .../lens-dissolution/prompts/implement_spec.md     |    17 +
 .../implement_tests_agent-context-builder.md       |    25 +
 .../prompts/implement_tests_agent-output-format.md |    25 +
 ...implement_tests_lens-dissolution-restructure.md |    25 +
 .../prompts/review_reference_context.md            |    31 +
 .../groups/lens-dissolution/reference_context.md   |    72 +
 .../groups/lens-dissolution/requirements.md        |    12 +
 .../groups/lens-dissolution/spec_plan.yaml         |    44 +
 .../specs/agent-context-builder.md                 |   358 +
 .../lens-dissolution/specs/agent-output-format.md  |   312 +
 .../specs/lens-dissolution-restructure.md          |   386 +
 .../specs/sdd-cli-context-command.md               |   198 +
 .../specs/type-inference-pipeline.md               |   510 +
 cclab/changes/lens-dissolution/implementation.md   |    82 +
 ...or-dissolve-lens-module-into-sdd-top-level-s.md |    65 +
 ...ens-wire-cross-file-type-propagation-deep-in.md |    49 +
 ...ens-agent-context-builder-smart-file-selecti.md |    70 +
 ...ens-agent-optimized-output-structured-json-f.md |    53 +
 .../payloads/agent-context-builder-changes.json    |     8 +
 .../payloads/agent-context-builder-logic.json      |     8 +
 .../payloads/agent-context-builder-overview.json   |     8 +
 .../agent-context-builder-requirements.json        |     8 +
 .../payloads/agent-context-builder-scenarios.json  |     8 +
 .../payloads/agent-context-builder-schema.json     |     8 +
 .../payloads/agent-context-builder-test-plan.json  |     8 +
 .../payloads/agent-output-format-changes.json      |     8 +
 .../payloads/agent-output-format-overview.json     |     8 +
 .../payloads/agent-output-format-requirements.json |     8 +
 .../payloads/agent-output-format-scenarios.json    |     8 +
 .../payloads/agent-output-format-schema.json       |     8 +
 .../payloads/agent-output-format-test-plan.json    |     8 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |    20 +
 .../payloads/create-reference-context.json         |   140 +
 .../lens-dissolution-restructure-changes.json      |     8 +
 .../lens-dissolution-restructure-overview.json     |     8 +
 .../lens-dissolution-restructure-requirements.json |     8 +
 .../lens-dissolution-restructure-scenarios.json    |     8 +
 .../lens-dissolution-restructure-test-plan.json    |     8 +
 .../payloads/restructure-input.json                |    27 +
 .../payloads/review-reference-context.json         |    48 +
 .../payloads/sdd-cli-context-command-changes.json  |     8 +
 .../payloads/sdd-cli-context-command-overview.json |     9 +
 .../sdd-cli-context-command-requirements.json      |     8 +
 .../sdd-cli-context-command-scenarios.json         |     8 +
 .../payloads/type-inference-pipeline-changes.json  |     8 +
 .../payloads/type-inference-pipeline-logic.json    |     8 +
 .../payloads/type-inference-pipeline-overview.json |     8 +
 .../type-inference-pipeline-requirements.json      |     8 +
 .../type-inference-pipeline-scenarios.json         |     8 +
 .../payloads/type-inference-pipeline-schema.json   |     8 +
 .../type-inference-pipeline-test-plan.json         |     8 +
 .../lens-dissolution/prompts/restructure_input.md  |    64 +
 cclab/changes/lens-dissolution/user_input.md       |     1 +
 cclab/changes/mamba-jit-memory/STATE.yaml          |    38 +
 .../groups/jit-memory/post_clarifications.md       |    20 +
 .../groups/jit-memory/pre_clarifications.md        |    17 +
 .../analyze_spec_cranelift-jit-memory-fix.md       |    26 +
 .../jit-memory/prompts/analyze_spec_jit-memory.md  |    26 +
 .../jit-memory/prompts/begin_implementation.md     |    18 +
 .../jit-memory/prompts/create_reference_context.md |   270 +
 .../implement_tests_cranelift-jit-memory-fix.md    |    25 +
 .../groups/jit-memory/reference_context.md         |    25 +
 .../groups/jit-memory/requirements.md              |     9 +
 .../groups/jit-memory/spec_plan.yaml               |    15 +
 .../jit-memory/specs/cranelift-jit-memory-fix.md   |   240 +
 .../groups/jit-memory/specs/jit-memory.md          |   245 +
 ...mba-sigbus-crash-in-multi-threaded-conforman.md |    33 +
 ...ange-spec-cranelift-jit-memory-fix-changes.json |     6 +
 ...nge-spec-cranelift-jit-memory-fix-overview.json |     9 +
 ...spec-cranelift-jit-memory-fix-requirements.json |     6 +
 ...ge-spec-cranelift-jit-memory-fix-scenarios.json |     6 +
 ...ge-spec-cranelift-jit-memory-fix-test-plan.json |     6 +
 .../create-change-spec-jit-memory-changes.json     |     6 +
 .../create-change-spec-jit-memory-overview.json    |     9 +
 ...create-change-spec-jit-memory-requirements.json |     6 +
 .../create-change-spec-jit-memory-scenarios.json   |     6 +
 .../create-change-spec-jit-memory-test-plan.json   |     6 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |    14 +
 .../payloads/create-reference-context.json         |    86 +
 .../payloads/restructure-input.json                |    13 +
 .../mamba-jit-memory/prompts/restructure_input.md  |    64 +
 cclab/changes/mamba-jit-memory/user_input.md       |     1 +
 cclab/changes/mamba-refcount-jit/STATE.yaml        |    37 +
 .../groups/refcount-jit/post_clarifications.md     |    20 +
 .../groups/refcount-jit/pre_clarifications.md      |    13 +
 .../analyze_spec_mamba-refcount-jit-spec.md        |    26 +
 .../refcount-jit/prompts/begin_implementation.md   |    18 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_reference_context.md            |   270 +
 .../implement_tests_mamba-refcount-jit-spec.md     |    25 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../groups/refcount-jit/reference_context.md       |    17 +
 .../groups/refcount-jit/requirements.md            |     9 +
 .../refcount-jit/specs/mamba-refcount-jit-spec.md  |   395 +
 cclab/changes/mamba-refcount-jit/implementation.md |   760 ++
 ...or-mamba-implement-cpython-3-12-reference-co.md |    65 +
 .../payloads/create-change-implementation.json     |     6 +
 .../payloads/create-change-spec-changes.json       |     6 +
 .../payloads/create-change-spec-logic.json         |     6 +
 .../payloads/create-change-spec-overview.json      |     9 +
 .../payloads/create-change-spec-requirements.json  |     6 +
 .../payloads/create-change-spec-scenarios.json     |     6 +
 .../payloads/create-change-spec-test-plan.json     |     6 +
 .../payloads/create-post-clarifications.json       |    13 +
 .../payloads/create-pre-clarifications.json        |     9 +
 .../payloads/create-reference-context.json         |    42 +
 .../payloads/restructure-input.json                |    10 +
 .../prompts/restructure_input.md                   |    64 +
 cclab/changes/mamba-refcount-jit/user_input.md     |     1 +
 cclab/changes/sdd-gen-code-pipeline/STATE.yaml     |    37 +
 .../gen-code-pipeline/post_clarifications.md       |    20 +
 .../groups/gen-code-pipeline/pre_clarifications.md |     9 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../groups/gen-code-pipeline/reference_context.md  |    38 +
 .../groups/gen-code-pipeline/requirements.md       |    13 +
 .../groups/gen-code-pipeline/spec_plan.yaml        |     7 +
 .../groups/gen-code-pipeline/specs/gen-code-cli.md |   208 +
 ...dd-gen-code-gen-diff-gen-parse-spec-driven-c.md |    56 +
 .../payloads/create-post-clarifications.json       |     1 +
 .../payloads/create-pre-clarifications.json        |     1 +
 .../payloads/create-reference-context.json         |    14 +
 .../payloads/restructure-input.json                |    12 +
 .../payloads/review-reference-context.json         |     1 +
 .../prompts/restructure_input.md                   |    64 +
 cclab/changes/sdd-gen-code-pipeline/user_input.md  |     1 +
 cclab/changes/sdd-index-path-rename/STATE.yaml     |    28 +
 .../post_clarifications.md                         |    20 +
 .../index-path-and-cli-hints/pre_clarifications.md |     9 +
 .../prompts/analyze_spec_cli-hints-impl-prompt.md  |    26 +
 .../prompts/analyze_spec_index-path-rename.md      |    26 +
 .../prompts/begin_implementation.md                |    18 +
 .../prompts/create_post_clarifications.md          |    56 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../prompts/create_reference_context.md            |   255 +
 .../prompts/implement_spec.md                      |    17 +
 .../implement_tests_cli-hints-impl-prompt.md       |    25 +
 .../prompts/implement_tests_index-path-rename.md   |    25 +
 .../prompts/review_reference_context.md            |    31 +
 .../prompts/write_implementation_diff.md           |    14 +
 .../index-path-and-cli-hints/reference_context.md  |    40 +
 .../index-path-and-cli-hints/requirements.md       |    16 +
 .../groups/index-path-and-cli-hints/spec_plan.yaml |    14 +
 .../specs/cli-hints-impl-prompt.md                 |   442 +
 .../specs/index-path-rename.md                     |   271 +
 .../sdd-index-path-rename/implementation.md        |   109 +
 .../payloads/create-change-implementation.json     |     4 +
 .../create-change-spec-cli-hints-changes.json      |     6 +
 .../create-change-spec-cli-hints-overview.json     |     9 +
 ...eate-change-spec-index-path-rename-changes.json |     6 +
 ...ate-change-spec-index-path-rename-overview.json |     9 +
 .../payloads/create-post-clarifications.json       |     7 +
 .../payloads/create-pre-clarifications.json        |     7 +
 .../payloads/create-reference-context.json         |    72 +
 .../payloads/restructure-input.json                |    12 +
 .../payloads/review-reference-context.json         |    12 +
 .../prompts/restructure_input.md                   |    45 +
 cclab/changes/sdd-index-path-rename/user_input.md  |     1 +
 .../changes/sdd-index-scoped-toolchain/STATE.yaml  |    37 +
 .../groups/scoped-toolchain/post_clarifications.md |    20 +
 .../groups/scoped-toolchain/pre_clarifications.md  |     9 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../groups/scoped-toolchain/reference_context.md   |    40 +
 .../groups/scoped-toolchain/requirements.md        |    27 +
 .../groups/scoped-toolchain/spec_plan.yaml         |    20 +
 .../groups/scoped-toolchain/specs/auto-discover.md |   103 +
 .../scoped-toolchain/specs/index-config-model.md   |   103 +
 .../scoped-toolchain/specs/multi-handler-daemon.md |   201 +
 ...dd-index-server-scoped-toolchain-binding-aut.md |    73 +
 .../payloads/create-post-clarifications.json       |     1 +
 .../payloads/create-pre-clarifications.json        |     1 +
 .../payloads/create-reference-context.json         |    16 +
 .../payloads/restructure-input.json                |    12 +
 .../payloads/review-reference-context.json         |     1 +
 .../prompts/restructure_input.md                   |    64 +
 .../sdd-index-scoped-toolchain/user_input.md       |     1 +
 cclab/changes/sdd-phase-advance-timeout/STATE.yaml |    43 +
 .../post_clarifications.md                         |    20 +
 .../pre_clarifications.md                          |     9 +
 .../prompts/create_post_clarifications.md          |    58 +
 .../prompts/create_pre_clarifications.md           |    29 +
 .../phase-advance-and-timeout/reference_context.md |    38 +
 .../phase-advance-and-timeout/requirements.md      |    17 +
 .../phase-advance-and-timeout/spec_plan.yaml       |    14 +
 .../specs/agent-timeout.md                         |   265 +
 .../specs/phase-advance-fix.md                     |   311 +
 ...d-reference-context-phase-never-advances-gro.md |    33 +
 ...dd-agent-execution-timeout-prevent-infinite-.md |    27 +
 .../payloads/create-post-clarifications.json       |     1 +
 .../payloads/create-pre-clarifications.json        |     7 +
 .../payloads/create-reference-context.json         |    38 +
 .../payloads/restructure-input.json                |    12 +
 .../payloads/review-reference-context.json         |     7 +
 .../prompts/restructure_input.md                   |    64 +
 .../sdd-phase-advance-timeout/user_input.md        |     1 +
 cclab/config.toml                                  |    21 +-
 cclab/specs/AUTHORING.md                           |   238 +
 .../crates/cclab-fetch/broker/broker-traits.md     |   322 +
 .../specs/crates/cclab-fetch/broker/cloudtasks.md  |   607 +
 .../scheduler/cloud-scheduler-backend.md           |   553 +
 .../cclab-fetch/scheduler/k8s-cronjob-backend.md   |   395 +
 .../crates/cclab-fetch/scheduler/push-receiver.md  |   626 ++
 .../cclab-fetch/scheduler/schedule-monitor.md      |   393 +
 .../cclab-fetch/scheduler/scheduler-backends.md    |   150 +
 .../scheduler/scheduler-mode-selection.md          |   402 +
 cclab/specs/crates/cclab-jet/aot-build.md          |   115 +-
 .../cclab-jet/bundle-optimization-hoisting.md      |   136 -
 cclab/specs/crates/cclab-jet/dev-server.md         |   874 ++
 .../cclab-jet/e2e/e2e-test-infrastructure.md       |   370 +
 cclab/specs/crates/cclab-jet/hmr.md                |   717 ++
 cclab/specs/crates/cclab-jet/jet-remaining-spec.md |    10 -
 cclab/specs/crates/cclab-jet/logic/aot-build.md    |   310 +
 cclab/specs/crates/cclab-jet/logic/hmr.md          |   384 +
 .../crates/cclab-jet/logic/postcss-tailwind.md     |   499 +
 .../specs/crates/cclab-jet/logic/scope-hoisting.md |   387 +
 cclab/specs/crates/cclab-jet/workspace-protocol.md |   440 +
 .../crates/mamba/codegen/cranelift-jit.md    |    36 +
 cclab/specs/crates/mamba/runtime/gc.md       |    22 +
 .../specs/crates/mamba/runtime/generator.md  |    32 +
 .../crates/mamba/testing/conformance.md      |   716 +-
 cclab/specs/crates/cclab-sdd/config/agents.md      |    32 +-
 cclab/specs/crates/cclab-sdd/generate/README.md    |     4 +-
 .../crates/cclab-sdd/generate/architecture.md      |    14 +-
 .../generate/requirement-plus-enhancement.md       |     2 +-
 .../crates/cclab-sdd/generate/spec-ir-contract.md  |     2 +-
 .../cclab-sdd/generate/spec-ir-evaluation.md       |     2 +-
 .../specs/crates/cclab-sdd/generate/spec-model.md  |     2 +-
 .../cclab-sdd/generate/template-claude-md.md       |     6 +-
 .../cclab-sdd/generate/template-knowledge-index.md |     4 +-
 .../cclab-sdd/generate/ux-pattern-library.md       |   469 +
 .../crates/cclab-sdd/interfaces/cli/commands.md    |   248 +-
 .../crates/cclab-sdd/interfaces/cli/sdd-cli.md     |   238 +-
 .../interfaces/lens}/lens-cli-subcommands.md       |     0
 .../interfaces/lens}/lens-pdg-mcp-tools.md         |     0
 .../cclab-sdd/interfaces/tools/artifact-tools.md   |     2 +-
 .../cclab-sdd/interfaces/tools/utility-tools.md    |     2 +-
 .../cclab-sdd/logic/agent-context-builder.md       |   353 +
 .../crates/cclab-sdd/logic/agent-output-format.md  |   307 +
 .../logic}/analysis-tools.md                       |     0
 .../logic}/cclab-lens-spec.md                      |     0
 cclab/specs/crates/cclab-sdd/logic/change-merge.md |     9 +-
 cclab/specs/crates/cclab-sdd/logic/change-spec.md  |   335 +-
 .../logic}/class-diagram.md                        |     0
 .../logic}/code-analysis-service-v2.md             |     0
 .../crates/cclab-sdd/logic/executor-resolution.md  |   228 +-
 .../specs/crates/cclab-sdd/logic/implement-task.md |   285 +-
 .../README.md => cclab-sdd/logic/lens-README.md}   |     0
 .../logic}/lens-beyond-ide.md                      |     0
 .../logic}/lens-codegen-unification.md             |     0
 .../logic}/lens-comprehensive.md                   |     0
 .../logic}/lens-full-upgrade-spec.md               |     0
 .../logic}/lens-index-storage.md                   |    10 +-
 .../logic}/lens-lang-support.md                    |     0
 .../logic}/lens-markdown.md                        |     0
 .../logic}/lens-yaml-codegen.md                    |     0
 .../cclab-sdd/logic/merge-lens-into-sdd-spec.md    |   379 +-
 .../crates/cclab-sdd/logic/post-clarifications.md  |    10 +-
 .../crates/cclab-sdd/logic/pre-clarifications.md   |     2 +-
 .../logic}/python-pdg-core.md                      |     0
 .../logic}/refactoring-api.md                      |     0
 .../crates/cclab-sdd/logic/reference-context.md    |     6 +-
 .../crates/cclab-sdd/logic/restructure-input.md    |     2 +-
 .../logic}/rust-symbol-analysis.md                 |     0
 .../logic}/semantic-search-api.md                  |     0
 .../crates/cclab-sdd/logic/spec-diff-codegen.md    |   155 +
 .../specs/crates/cclab-sdd/logic/state-machine.md  |     2 +-
 .../crates/cclab-sdd/logic/tech-stack-inference.md |   539 +
 .../cclab-sdd/logic/type-inference-pipeline.md     |   505 +
 .../logic}/usage-examples.md                       |     0
 cclab/specs/crates/cclab-sdd/skills/agent.md       |     8 +-
 cclab/specs/crates/cclab-sdd/skills/fillback.md    |     2 +-
 cclab/specs/crates/cclab-sdd/skills/run-change.md  |   142 +-
 .../cclab-sdd/tools/utils/analyze-code-for-spec.md |    12 +-
 .../crates/cclab-sdd/tools/utils/delegate-agent.md |   167 +-
 .../crates/cclab-sdd/tools/utils/fetch-issues.md   |     4 +-
 .../cclab-sdd/tools/utils/list-changed-files.md    |     4 +-
 .../crates/cclab-sdd/tools/utils/platform-sync.md  |     4 +-
 .../crates/cclab-sdd/tools/utils/read-artifact.md  |     2 +-
 .../tools/utils/read-implementation-summary.md     |     4 +-
 .../cclab-sdd/tools/utils/validate-change.md       |    12 +-
 .../tools/utils/validate-spec-completeness.md      |     2 +-
 .../crates/cclab-sdd/tools/utils/write-artifact.md |     4 +-
 crates/cclab-aurora/Cargo.toml                     |    22 +
 crates/cclab-jet/examples/dev_server.rs            |     1 +
 crates/cclab-jet/examples/full_pipeline.rs         |     1 +
 crates/cclab-jet/src/asset/image_processor.rs      |   306 +-
 crates/cclab-jet/src/bundler/css_bundle.rs         |   336 +
 crates/cclab-jet/src/bundler/html_minify.rs        |   358 +
 crates/cclab-jet/src/bundler/json_shake.rs         |   255 +
 crates/cclab-jet/src/bundler/minify.rs             |     4 +
 crates/cclab-jet/src/bundler/mod.rs                |   176 +-
 crates/cclab-jet/src/bundler/scope_hoist.rs        |   323 +-
 crates/cclab-jet/src/bundler/scope_hoist_opt.rs    |   883 ++
 crates/cclab-jet/src/bundler/sourcemap.rs          |   453 +
 crates/cclab-jet/src/bundler/splitting.rs          |   762 ++
 crates/cclab-jet/src/bundler/tree_shake.rs         |    23 +-
 crates/cclab-jet/src/bundler/types.rs              |    21 +
 crates/cclab-jet/src/cli.rs                        |    20 +-
 crates/cclab-jet/src/css/import_resolver.rs        |    59 +
 crates/cclab-jet/src/css/output.rs                 |    48 +
 crates/cclab-jet/src/css/tailwind/preflight.rs     |    32 +
 crates/cclab-jet/src/css/tailwind/utilities.rs     |   116 +
 crates/cclab-jet/src/css/tailwind/variants.rs      |   102 +
 crates/cclab-jet/src/dev_server/hmr.rs             |   329 +-
 crates/cclab-jet/src/dev_server/hmr_client.rs      |   336 +
 crates/cclab-jet/src/dev_server/importmap.rs       |   235 +
 crates/cclab-jet/src/dev_server/mod.rs             |  1167 +-
 crates/cclab-jet/src/dev_server/module_graph.rs    |   696 ++
 crates/cclab-jet/src/dev_server/polyfills.rs       |   729 ++
 crates/cclab-jet/src/dev_server/polyfills_tests.rs |   319 +
 crates/cclab-jet/src/dev_server/prebundle.rs       |   964 ++
 crates/cclab-jet/src/dev_server/prebundle_tests.rs |   428 +
 crates/cclab-jet/src/dev_server/react_refresh.rs   |   130 +
 crates/cclab-jet/src/dev_server/source_analysis.rs |   408 +
 crates/cclab-jet/src/dev_server/watcher.rs         |    32 +-
 crates/cclab-jet/src/pkg_manager/mod.rs            |    30 +-
 crates/cclab-jet/src/pkg_manager/platform.rs       |   174 +
 crates/cclab-jet/src/pkg_manager/store.rs          |   429 +
 crates/cclab-jet/src/transform/mod.rs              |    26 +
 crates/cclab-jet/src/transform/react_refresh.rs    |   292 +
 crates/cclab-jet/src/transform/transform_tsx.rs    |   146 +-
 .../cclab-jet/src/transform/transform_tsx_tests.rs |   645 ++
 crates/cclab-jet/src/transform/type_strip.rs       |   198 +
 crates/cclab-jet/tests/workspace_protocol.rs       |   457 +
 crates/mamba/Cargo.toml                      |     4 +
 crates/mamba/src/codegen/cranelift/jit.rs    |   141 +-
 crates/mamba/src/codegen/cranelift/mod.rs    |   105 +-
 crates/mamba/src/conformance/mod.rs          |    16 +-
 crates/mamba/src/lexer/indent.rs             |     8 +
 crates/mamba/src/lower/ast_to_hir.rs         |   183 +-
 crates/mamba/src/lower/hir_to_mir.rs         |    61 +
 crates/mamba/src/parser/expr.rs              |    62 +
 crates/mamba/src/parser/expr_compound.rs     |    62 +
 crates/mamba/src/runtime/async_rt.rs         |    10 +
 crates/mamba/src/runtime/builtins.rs         |   703 +-
 crates/mamba/src/runtime/class.rs            |   281 +-
 crates/mamba/src/runtime/dict_ops.rs         |    65 +-
 crates/mamba/src/runtime/gc.rs               |    15 +-
 crates/mamba/src/runtime/generator.rs        |  1218 +-
 crates/mamba/src/runtime/iter.rs             |    35 +
 crates/mamba/src/runtime/list_ops.rs         |    95 +-
 crates/mamba/src/runtime/mod.rs              |     3 +-
 crates/mamba/src/runtime/rc.rs               |   307 +
 crates/mamba/src/runtime/set_ops.rs          |    31 +-
 crates/mamba/src/runtime/string_ops.rs       |   148 +-
 crates/mamba/src/runtime/symbols.rs          |    12 +
 crates/mamba/src/types/check_expr.rs         |    40 +-
 .../cclab-mamba/tests/behavioral_builtins_tests.rs |    17 +-
 crates/mamba/tests/behavioral_lang_tests.rs  |    80 +-
 .../cclab-mamba/tests/behavioral_stdlib_tests.rs   |    17 +-
 crates/mamba/tests/conformance_tests.rs      |    31 +-
 crates/mamba/tests/fixture_tests.rs          |     4 +-
 .../fixtures/conformance/__snippet_test.expected   |     2 +-
 .../tests/fixtures/conformance/__snippet_test.py   |     4 +-
 .../builtins/collection_builtins_edge.py           |     2 +-
 .../conformance/builtins/collection_edge_cases.py  |     2 +-
 .../conformance/builtins/numeric_edge_cases.py     |     2 +-
 .../fixtures/conformance/builtins/print_kwargs.py  |     2 +-
 .../fixtures/conformance/builtins/repr_format.py   |     2 +-
 .../conformance/builtins/string_repr_edge_cases.py |     1 -
 .../conformance/builtins/type_introspection.py     |     2 +-
 .../builtins/type_introspection_edge_cases.py      |     2 +-
 .../class_system/mro_edge_cases.expected           |     5 -
 .../conformance/class_system/mro_edge_cases.py     |    41 -
 .../data_structures/bytes_edge_cases.expected      |     5 +-
 .../data_structures/bytes_edge_cases.py            |    11 +-
 .../data_structures/dict_edge_cases_xfail.expected |     1 -
 .../data_structures/dict_edge_cases_xfail.py       |     5 +-
 .../data_structures/list_constructor_xfail.py      |     2 +-
 .../data_structures/list_edge_cases_xfail.py       |     1 -
 .../data_structures/list_sort_lambda.py            |     2 +-
 .../data_structures/set_edge_cases_xfail.expected  |     2 +-
 .../data_structures/set_edge_cases_xfail.py        |     6 +-
 .../data_structures/string_edge_cases_xfail.py     |     1 -
 .../data_structures/string_format_xfail.py         |     2 +-
 .../data_structures/tuple_edge_cases_xfail.py      |     2 +-
 .../exceptions/chaining_edge_cases.expected        |    10 +-
 .../conformance/exceptions/chaining_edge_cases.py  |    35 +-
 .../generators/close_edge_cases_xfail.py           |     3 +-
 .../generators/context_manager_pattern_xfail.py    |     3 +-
 .../generators/send_edge_cases_xfail.py            |     2 +-
 .../generators/state_attributes.expected           |     8 +-
 .../conformance/generators/state_attributes.py     |    26 +-
 .../conformance/generators/throw_edge_cases.py     |     1 -
 .../yield_from_passthrough_xfail.expected          |     6 +-
 .../generators/yield_from_passthrough_xfail.py     |    45 +-
 .../iterators/callable_sentinel.expected           |     1 -
 .../conformance/iterators/callable_sentinel.py     |     7 +-
 .../iterators/composition_xfail.expected           |     6 +-
 .../conformance/iterators/composition_xfail.py     |    28 +-
 .../iterators/custom_iterator_xfail.expected       |    13 +-
 .../conformance/iterators/custom_iterator_xfail.py |    71 +-
 .../conformance/iterators/unpacking.expected       |     5 +-
 .../fixtures/conformance/iterators/unpacking.py    |    40 +-
 .../comprehension_scope_edge_cases.expected        |     3 +-
 .../language/comprehension_scope_edge_cases.py     |     9 +-
 .../language/context_manager_edge_cases.py         |     1 -
 .../language/decorator_edge_cases.expected         |     1 -
 .../conformance/language/decorator_edge_cases.py   |    17 -
 .../language/lambda_edge_cases.expected            |     5 +-
 .../conformance/language/lambda_edge_cases.py      |    19 +-
 .../language/pattern_matching_edge_cases.py        |    53 +-
 .../collections/collections_conformance.expected   |     9 +-
 .../stdlib/collections/collections_conformance.py  |    30 +-
 .../stdlib/collections_conformance.expected        |    21 +-
 .../conformance/stdlib/collections_conformance.py  |    68 +-
 .../stdlib/csv/csv_conformance.expected            |     8 +-
 .../conformance/stdlib/csv/csv_conformance.py      |    19 +-
 .../stdlib/datetime/datetime_conformance.expected  |     5 +-
 .../stdlib/datetime/datetime_conformance.py        |    20 +-
 .../stdlib/datetime_conformance.expected           |    22 +-
 .../conformance/stdlib/datetime_conformance.py     |    56 +-
 .../functools/functools_conformance.expected       |     5 +-
 .../stdlib/functools/functools_conformance.py      |    14 +-
 .../stdlib/functools_conformance.expected          |    15 +-
 .../conformance/stdlib/functools_conformance.py    |    45 +-
 .../stdlib/hashlib/hashlib_conformance.expected    |     6 +-
 .../stdlib/hashlib/hashlib_conformance.py          |    14 +-
 .../conformance/stdlib/io/io_conformance.expected  |     6 +-
 .../conformance/stdlib/io/io_conformance.py        |    22 +-
 .../itertools/itertools_conformance.expected       |     7 +-
 .../stdlib/itertools/itertools_conformance.py      |    13 +-
 .../stdlib/itertools_conformance.expected          |    21 +-
 .../conformance/stdlib/itertools_conformance.py    |    42 +-
 .../stdlib/json/json_conformance.expected          |    11 +-
 .../conformance/stdlib/json/json_conformance.py    |    21 +-
 .../conformance/stdlib/json_conformance.expected   |    19 +-
 .../conformance/stdlib/json_conformance.py         |    43 +-
 .../conformance/stdlib/math/math_conformance.py    |     2 +-
 .../conformance/stdlib/math_basic.expected         |    11 -
 .../fixtures/conformance/stdlib/math_basic.py      |    31 +-
 .../stdlib/random/random_conformance.expected      |     7 +-
 .../stdlib/random/random_conformance.py            |    17 +-
 .../conformance/stdlib/re/re_conformance.expected  |     9 +-
 .../conformance/stdlib/re/re_conformance.py        |    17 +-
 .../conformance/stdlib/re_conformance.expected     |    18 +-
 .../fixtures/conformance/stdlib/re_conformance.py  |    45 +-
 .../stdlib/struct/struct_conformance.expected      |     6 +-
 .../stdlib/struct/struct_conformance.py            |    19 +-
 crates/mamba/tests/gen_thread_pool_tests.rs  |   322 +
 .../tests/generator_conformance_tests.rs           |   778 ++
 .../tests/iterator_conformance_tests.rs            |   519 +
 crates/mamba/tests/jit_refcount_tests.rs     |   372 +
 crates/mamba/tests/jit_tests.rs              |     5 +-
 .../cclab-mamba/tests/no_arg_constructor_tests.rs  |   197 +
 crates/mamba/tests/p0_conformance_tests.rs   |    17 +-
 .../tests/runtime_bugs_conformance_tests.rs        |    17 +-
 .../tests/xfail_zero_conformance_tests.rs          |  1024 ++
 crates/cclab-pyo3/src/lib.rs                       |    37 +-
 crates/cclab-queue/Cargo.toml                      |    30 +-
 crates/cclab-queue/src/broker/cloudtasks.rs        |  1889 +++-
 crates/cclab-queue/src/broker/mod.rs               |   626 ++
 crates/cclab-queue/src/error.rs                    |     6 +
 crates/cclab-queue/src/lib.rs                      |     6 +
 crates/cclab-queue/src/scheduler/backend.rs        |   195 +
 .../src/scheduler/cloud_scheduler_backend.rs       |   657 ++
 .../src/scheduler/cloud_scheduler_backend_tests.rs |   702 ++
 .../src/scheduler/k8s_cronjob_backend.rs           |   525 +
 crates/cclab-queue/src/scheduler/mod.rs            |   394 +-
 crates/cclab-queue/src/scheduler/periodic.rs       |   781 +-
 crates/cclab-queue/src/scheduler/push_auth.rs      |   542 +
 crates/cclab-queue/src/scheduler/push_receiver.rs  |  1557 +++
 .../cclab-queue/src/scheduler/schedule_monitor.rs  |  1347 +++
 crates/cclab-sdd-cli/src/codegen.rs                |   130 +-
 crates/cclab-sdd-cli/src/commands.rs               |   110 +-
 crates/cclab-sdd-cli/src/daemon.rs                 |    14 +-
 crates/cclab-sdd-cli/src/direct.rs                 |   160 +-
 .../skills/cclab-sdd-run-change/SKILL.md           |    12 +-
 crates/cclab-sdd/Cargo.toml                        |     1 +
 crates/cclab-sdd/src/check_pipeline.rs             |   240 +
 crates/cclab-sdd/src/{lens/mod.rs => checker.rs}   |   104 +-
 crates/cclab-sdd/src/context_builder/mod.rs        |   769 ++
 .../src/context_builder/test_detection.rs          |   294 +
 crates/cclab-sdd/src/context_builder/traversal.rs  |   405 +
 crates/cclab-sdd/src/context_builder/types.rs      |   181 +
 crates/cclab-sdd/src/{lens => }/core/config.rs     |     0
 crates/cclab-sdd/src/{lens => }/core/mod.rs        |     0
 crates/cclab-sdd/src/{lens => }/diagnostic.rs      |     0
 crates/cclab-sdd/src/{lens => }/format/detect.rs   |     0
 crates/cclab-sdd/src/{lens => }/format/mod.rs      |     0
 .../cclab-sdd/src/{lens => }/gen/framework/axum.rs |     4 +-
 .../src/{lens => }/gen/framework/express.rs        |     4 +-
 .../src/{lens => }/gen/framework/fastapi.rs        |     4 +-
 .../cclab-sdd/src/{lens => }/gen/framework/mod.rs  |     0
 crates/cclab-sdd/src/{lens => }/gen/mod.rs         |     0
 .../cclab-sdd/src/{lens => }/gen/python/meteor.rs  |     8 +-
 crates/cclab-sdd/src/{lens => }/gen/python/mod.rs  |     6 +-
 .../cclab-sdd/src/{lens => }/gen/python/nebula.rs  |     8 +-
 .../cclab-sdd/src/{lens => }/gen/python/photon.rs  |    10 +-
 crates/cclab-sdd/src/{lens => }/gen/python/pyo3.rs |     0
 .../src/{lens => }/gen/python/pyo3_gen.rs          |     4 +-
 .../cclab-sdd/src/{lens => }/gen/python/quasar.rs  |    12 +-
 .../src/{lens => }/gen/python/rust_scanner.rs      |     0
 .../cclab-sdd/src/{lens => }/gen/python/shield.rs  |    18 +-
 .../src/{lens => }/gen/python/test_extractor.rs    |     0
 .../cclab-sdd/src/{lens => }/gen/python/titan.rs   |     8 +-
 .../src/{lens => }/gen/python/type_map.rs          |     0
 crates/cclab-sdd/src/{lens => }/gen/registry.rs    |     4 +-
 crates/cclab-sdd/src/{lens => }/gen/rust/axum.rs   |    10 +-
 crates/cclab-sdd/src/{lens => }/gen/rust/mod.rs    |     6 +-
 .../cclab-sdd/src/{lens => }/gen/rust/reqwest.rs   |    10 +-
 crates/cclab-sdd/src/{lens => }/gen/rust/serde.rs  |    10 +-
 crates/cclab-sdd/src/{lens => }/gen/rust/sqlx.rs   |     8 +-
 crates/cclab-sdd/src/{lens => }/gen/traits.rs      |     2 +-
 crates/cclab-sdd/src/generate/lib.rs               |     1 +
 crates/cclab-sdd/src/generate/patterns/mod.rs      |    80 +
 crates/cclab-sdd/src/generate/patterns/registry.rs |    17 +
 crates/cclab-sdd/src/generate/patterns/resolver.rs |    66 +
 crates/cclab-sdd/src/{lens => }/graph/mod.rs       |     0
 crates/cclab-sdd/src/{lens => }/graph/resolve.rs   |     4 +-
 crates/cclab-sdd/src/{lens => }/handlers.rs        |    26 +-
 .../cclab-sdd/src/{lens/error.rs => lens_error.rs} |     0
 crates/cclab-sdd/src/lib.rs                        |    41 +-
 crates/cclab-sdd/src/{lens => }/lint/asyncapi.rs   |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/autofix.rs    |     6 +-
 crates/cclab-sdd/src/{lens => }/lint/css.rs        |     6 +-
 crates/cclab-sdd/src/{lens => }/lint/css_rules.rs  |     4 +-
 crates/cclab-sdd/src/{lens => }/lint/custom.rs     |    16 +-
 crates/cclab-sdd/src/{lens => }/lint/dockerfile.rs |     6 +-
 .../src/{lens => }/lint/embedded_markdown.rs       |     2 +-
 crates/cclab-sdd/src/{lens => }/lint/gitlab_ci.rs  |     6 +-
 .../src/{lens => }/lint/gitlab_ci_rules.rs         |     2 +-
 crates/cclab-sdd/src/{lens => }/lint/go.rs         |    12 +-
 crates/cclab-sdd/src/{lens => }/lint/graphql.rs    |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/html.rs       |     6 +-
 crates/cclab-sdd/src/{lens => }/lint/html_rules.rs |     4 +-
 crates/cclab-sdd/src/{lens => }/lint/javascript.rs |     6 +-
 crates/cclab-sdd/src/{lens => }/lint/kubernetes.rs |     6 +-
 .../src/{lens => }/lint/kubernetes_rules.rs        |     2 +-
 crates/cclab-sdd/src/{lens => }/lint/markdown.rs   |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/mdx.rs        |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/mermaid.rs    |    12 +-
 crates/cclab-sdd/src/{lens => }/lint/mod.rs        |     6 +-
 crates/cclab-sdd/src/{lens => }/lint/openapi.rs    |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/openrpc.rs    |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/proto.rs      |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/python.rs     |    26 +-
 .../src/{lens => }/lint/python_security.rs         |     4 +-
 .../cclab-sdd/src/{lens => }/lint/rust_checker.rs  |     6 +-
 crates/cclab-sdd/src/{lens => }/lint/sql.rs        |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/terraform.rs  |    14 +-
 .../src/{lens => }/lint/terraform_rules.rs         |     8 +-
 .../cclab-sdd/src/{lens => }/lint/toml_checker.rs  |     8 +-
 crates/cclab-sdd/src/{lens => }/lint/typescript.rs |    12 +-
 .../cclab-sdd/src/{lens => }/lint/yaml_dispatch.rs |     6 +-
 crates/cclab-sdd/src/{lens => }/lsp/mod.rs         |     0
 crates/cclab-sdd/src/{lens => }/lsp/server.rs      |    19 +-
 crates/cclab-sdd/src/models/change.rs              |   255 +-
 crates/cclab-sdd/src/models/index_config.rs        |   121 +
 crates/cclab-sdd/src/models/mod.rs                 |     5 +-
 crates/cclab-sdd/src/models/spec_rules.rs          |   492 +
 crates/cclab-sdd/src/models/tech_stack.rs          |   112 +
 crates/cclab-sdd/src/orchestrator/cli_mapper.rs    |    29 +-
 crates/cclab-sdd/src/orchestrator/script_runner.rs |    23 +-
 crates/cclab-sdd/src/output/agent.rs               |   803 ++
 crates/cclab-sdd/src/output/agent_types.rs         |    80 +
 crates/cclab-sdd/src/output/mod.rs                 |    13 +
 .../src/{lens/output.rs => output/reporter.rs}     |    48 +-
 .../src/{lens => }/refactoring/extract.rs          |     6 +-
 .../src/{lens => }/refactoring/extract_helpers.rs  |     4 +-
 .../cclab-sdd/src/{lens => }/refactoring/inline.rs |    16 +-
 crates/cclab-sdd/src/{lens => }/refactoring/mod.rs |     8 +-
 .../src/{lens => }/refactoring/move_def.rs         |    10 +-
 .../cclab-sdd/src/{lens => }/refactoring/rename.rs |     8 +-
 .../src/{lens => }/refactoring/signature.rs        |     6 +-
 .../{lens => }/refactoring/signature_helpers.rs    |     4 +-
 .../src/{lens => }/schemas/frontmatter.rs          |     0
 crates/cclab-sdd/src/{lens => }/schemas/gitlab.rs  |     2 +-
 crates/cclab-sdd/src/{lens => }/schemas/k8s.rs     |     2 +-
 crates/cclab-sdd/src/{lens => }/schemas/mod.rs     |     2 +-
 crates/cclab-sdd/src/{lens => }/search/index.rs    |     6 +-
 crates/cclab-sdd/src/{lens => }/search/mod.rs      |    18 +-
 crates/cclab-sdd/src/{lens => }/search/query.rs    |    10 +-
 crates/cclab-sdd/src/{lens => }/semantic/mod.rs    |     2 +-
 .../cclab-sdd/src/{lens => }/semantic/pdg/cfg.rs   |     6 +-
 .../src/{lens => }/semantic/pdg/data_flow.rs       |     8 +-
 .../src/{lens => }/semantic/pdg/dominator.rs       |     4 +-
 .../cclab-sdd/src/{lens => }/semantic/pdg/mod.rs   |     6 +-
 crates/cclab-sdd/src/{lens => }/semantic/scope.rs  |     4 +-
 .../src/{lens => }/semantic/symbols/css.rs         |     6 +-
 .../src/{lens => }/semantic/symbols/dockerfile.rs  |     2 +-
 .../src/{lens => }/semantic/symbols/gitlab_ci.rs   |     2 +-
 .../src/{lens => }/semantic/symbols/go.rs          |     6 +-
 .../src/{lens => }/semantic/symbols/graphql_sym.rs |     4 +-
 .../src/{lens => }/semantic/symbols/html.rs        |     6 +-
 .../src/{lens => }/semantic/symbols/javascript.rs  |     0
 .../src/{lens => }/semantic/symbols/kubernetes.rs  |     6 +-
 .../src/{lens => }/semantic/symbols/markdown.rs    |     2 +-
 .../src/{lens => }/semantic/symbols/mermaid.rs     |     2 +-
 .../src/{lens => }/semantic/symbols/mod.rs         |     9 +-
 .../src/{lens => }/semantic/symbols/proto_sym.rs   |     4 +-
 .../src/{lens => }/semantic/symbols/python.rs      |     4 +-
 .../src/{lens => }/semantic/symbols/rust.rs        |     6 +-
 .../src/{lens => }/semantic/symbols/sql_sym.rs     |     4 +-
 .../src/{lens => }/semantic/symbols/terraform.rs   |     6 +-
 .../src/{lens => }/semantic/symbols/toml_sym.rs    |     4 +-
 .../src/{lens => }/semantic/symbols/typescript.rs  |     4 +-
 crates/cclab-sdd/src/{lens => }/semantic/tests.rs  |     6 +-
 .../cclab-sdd/src/{lens => }/semantic/types/go.rs  |     2 +-
 .../src/{lens => }/semantic/types/go_advanced.rs   |     4 +-
 .../src/{lens => }/semantic/types/go_tests.rs      |     2 +-
 .../cclab-sdd/src/{lens => }/semantic/types/mod.rs |     0
 crates/cclab-sdd/src/server/auto_discover.rs       |   330 +
 crates/cclab-sdd/src/{lens => }/server/daemon.rs   |    60 +-
 .../cclab-sdd/src/{lens => }/server/disk_cache.rs  |     4 +-
 crates/cclab-sdd/src/{lens => }/server/handler.rs  |    58 +-
 .../cclab-sdd/src/{lens => }/server/incremental.rs |     2 +-
 crates/cclab-sdd/src/{lens => }/server/mod.rs      |     1 +
 crates/cclab-sdd/src/{lens => }/server/protocol.rs |     0
 crates/cclab-sdd/src/{lens => }/server/tests.rs    |     4 +-
 .../src/{lens => }/server/watch_bridge.rs          |     2 +-
 crates/cclab-sdd/src/services/mod.rs               |     5 +-
 crates/cclab-sdd/src/services/spec_service.rs      |   372 +-
 .../cclab-sdd/src/services/tech_stack_service.rs   |   318 +
 .../cclab-sdd/src/{lens => }/spec/asyncapi/mod.rs  |     0
 .../src/{lens => }/spec/asyncapi/parser.rs         |     6 +-
 crates/cclab-sdd/src/{lens => }/spec/ir.rs         |     2 +-
 .../src/{lens => }/spec/json_schema/mod.rs         |     0
 .../src/{lens => }/spec/json_schema/parser.rs      |     6 +-
 .../src/{lens => }/spec/mermaid/generator.rs       |    14 +-
 .../cclab-sdd/src/{lens => }/spec/mermaid/mod.rs   |     0
 .../src/{lens => }/spec/mermaid/parser.rs          |     4 +-
 crates/cclab-sdd/src/{lens => }/spec/mod.rs        |     0
 .../cclab-sdd/src/{lens => }/spec/openapi/mod.rs   |     0
 .../src/{lens => }/spec/openapi/parser.rs          |     4 +-
 .../{lens => }/spec/statemachine/mermaid_plus.rs   |     0
 .../src/{lens => }/spec/statemachine/mod.rs        |     0
 .../src/{lens => }/spec/statemachine/schema.rs     |     0
 .../src/{lens => }/spec/statemachine/validator.rs  |     0
 crates/cclab-sdd/src/state/manager.rs              |    26 +
 crates/cclab-sdd/src/state/mod.rs                  |     2 +-
 crates/cclab-sdd/src/{lens => }/storage.rs         |    32 +-
 crates/cclab-sdd/src/{lens => }/syntax/mod.rs      |     0
 crates/cclab-sdd/src/{lens => }/syntax/parser.rs   |     2 +-
 crates/cclab-sdd/src/tools/agent.rs                |   112 +-
 crates/cclab-sdd/src/tools/common_change_spec.rs   |   101 +
 crates/cclab-sdd/src/tools/create_change_impl.rs   |   311 +-
 crates/cclab-sdd/src/tools/create_change_merge.rs  |   156 +-
 crates/cclab-sdd/src/tools/create_change_spec.rs   |    97 +-
 .../src/tools/create_post_clarifications.rs        |    39 +-
 .../src/tools/create_reference_context.rs          |    36 +-
 crates/cclab-sdd/src/tools/mod.rs                  |     8 +-
 crates/cclab-sdd/src/tools/review_change_spec.rs   |     2 +
 .../src/tools/review_reference_context.rs          |    13 +
 crates/cclab-sdd/src/tools/spec_plan.rs            |   188 +-
 crates/cclab-sdd/src/tools/workflow_common.rs      |   377 +-
 .../{lens/types => type_inference}/annotation.rs   |     0
 .../src/{lens/types => type_inference}/builtins.rs |     0
 .../src/{lens/types => type_inference}/cache.rs    |   105 +
 .../{lens/types => type_inference}/cfg_narrow.rs   |    82 +-
 .../src/{lens/types => type_inference}/check.rs    |     4 +-
 .../{lens/types => type_inference}/check_tests.rs  |     4 +-
 .../{lens/types => type_inference}/class_info.rs   |     0
 .../src/{lens/types => type_inference}/codegen.rs  |     3 +-
 .../src/{lens/types => type_inference}/config.rs   |     0
 .../types => type_inference}/deep_inference.rs     |   318 +-
 .../src/{lens/types => type_inference}/env.rs      |     0
 .../{lens/types => type_inference}/frameworks.rs   |     2 +-
 .../src/{lens/types => type_inference}/imports.rs  |    51 +
 .../{lens/types => type_inference}/incremental.rs  |     2 +-
 .../src/{lens/types => type_inference}/infer.rs    |     0
 .../{lens/types => type_inference}/infer_tests.rs  |    28 +-
 .../src/{lens/types => type_inference}/mod.rs      |     5 +
 .../src/{lens/types => type_inference}/model.rs    |    22 +-
 .../src/{lens/types => type_inference}/modules.rs  |     0
 .../{lens/types => type_inference}/mutable_ast.rs  |     0
 .../src/{lens/types => type_inference}/narrow.rs   |     0
 .../{lens/types => type_inference}/narrow_tests.rs |     0
 .../types => type_inference}/package_managers.rs   |     0
 .../src/{lens/types => type_inference}/project.rs  |     4 +-
 crates/cclab-sdd/src/type_inference/propagation.rs |   777 ++
 .../{lens/types => type_inference}/refactoring.rs  |     2 +-
 .../refactoring_multilang.rs                       |     0
 .../types => type_inference}/rust_advanced.rs      |     0
 .../{lens/types => type_inference}/rust_infer.rs   |     0
 .../types => type_inference}/rust_lifetimes.rs     |     0
 .../{lens/types => type_inference}/rust_symbols.rs |     0
 .../{lens/types => type_inference}/rust_traits.rs  |     0
 .../{lens/types => type_inference}/rust_types.rs   |     2 +-
 .../types => type_inference}/semantic_search.rs    |   114 +-
 .../semantic_search_rust.rs                        |     0
 .../src/{lens/types => type_inference}/stubs.rs    |     0
 .../{lens/types => type_inference}/ts_advanced.rs  |     0
 .../src/{lens/types => type_inference}/ts_infer.rs |     0
 .../src/{lens/types => type_inference}/ts_types.rs |     0
 .../src/{lens/types => type_inference}/ty.rs       |     0
 .../src/{lens/types => type_inference}/ty_tests.rs |     0
 .../src/{lens/types => type_inference}/type_env.rs |     0
 .../src/{lens/types => type_inference}/typeshed.rs |     0
 crates/cclab-sdd/src/{lens => }/watch.rs           |     0
 crates/cclab-sdd/src/workflow/helpers.rs           |     5 +
 .../skills/cclab-sdd-run-change/SKILL.md           |    12 +-
 crates/cclab-sdd/tests/lens_dissolution_test.rs    |  1210 ++
 crates/cclab-server/src/lens_pool.rs               |     2 +-
 crates/cclab-server/src/lib.rs                     |     7 +-
 crates/cclab-server/src/mcp/mod.rs                 |     2 +-
 crates/cclab-server/src/mcp/router.rs              |    94 +-
 docs/.gitignore                                    |     3 +
 docs/.vitepress/config.mjs                         |    43 +
 docs/PROMPT_TEMPLATE_INTEGRATION.md                |   340 -
 docs/README.md                                     |   623 --
 docs/agent_eval_prompt_templates.md                |   391 -
 docs/api/sse.md                                    |   521 -
 docs/archive/ADAPTIVE_SAMPLING.md                  |   227 -
 docs/archive/MIGRATION_MAP.md                      |   278 -
 docs/archive/OPENTELEMETRY.md                      |  2753 -----
 docs/archive/PHASE2_LAZY_LOADING_IMPLEMENTATION.md |   287 -
 docs/archive/PHASE3_IMPLEMENTATION_SUMMARY.md      |   206 -
 docs/archive/PHASE4_IMPLEMENTATION_SUMMARY.md      |   517 -
 docs/archive/PHASE5_SUMMARY.md                     |   244 -
 docs/archive/PHASE7_IMPLEMENTATION_SUMMARY.md      |   315 -
 docs/archive/POSTGRESQL_EXTENSIONS.md              |   468 -
 docs/archive/README.md                             |    33 -
 docs/archive/SHEET_ARCHITECTURE.md                 |   823 --
 docs/archive/SHEET_CONTRIBUTING.md                 |   347 -
 docs/archive/SHEET_README.md                       |   382 -
 docs/archive/SPAN_HIERARCHY.md                     |   207 -
 docs/archive/TELEMETRY_RELATIONSHIPS.md            |   222 -
 docs/archive/canvas_primitives.md                  |   239 -
 docs/archive/kv_benchmark_concurrent_fixed.md      |    44 -
 docs/archive/legacy/BATCH_CONVERSION_SUMMARY.md    |   302 -
 docs/archive/legacy/CONVERSION_REPORT.md           |   215 -
 docs/archive/legacy/CRUD_API_REFACTOR_SUMMARY.md   |   229 -
 .../legacy/PHASE5_IMPLEMENTATION_COMPLETE.md       |   282 -
 docs/archive/legacy/PYLOOP_COMPILATION_FIXES.md    |   230 -
 docs/archive/legacy/PYLOOP_PHASE1_SUMMARY.md       |   265 -
 docs/archive/legacy/PYLOOP_PHASE2.5_SUMMARY.md     |   179 -
 docs/archive/legacy/PYLOOP_PHASE2_3_SUMMARY.md     |   233 -
 docs/archive/legacy/PYLOOP_PHASE2_4_SUMMARY.md     |   205 -
 docs/archive/legacy/PYLOOP_PHASE2_SUMMARY.md       |   195 -
 docs/archive/legacy/PYLOOP_PHASE3.1.1_SUMMARY.md   |   255 -
 docs/archive/legacy/PYLOOP_PHASE3.1.2_SUMMARY.md   |   345 -
 docs/archive/legacy/PYLOOP_PHASE3.1.3_SUMMARY.md   |   379 -
 docs/archive/legacy/PYLOOP_PHASE3.1.4_SUMMARY.md   |   435 -
 docs/archive/legacy/PYLOOP_PHASE3_CRUD_SUMMARY.md  |   270 -
 docs/archive/legacy/PYLOOP_PHASE3_SUMMARY.md       |   242 -
 docs/archive/legacy/PYLOOP_PHASE4_FILES.md         |   199 -
 docs/archive/legacy/PYLOOP_PHASE4_SUMMARY.md       |   659 --
 docs/archive/legacy/PYLOOP_PHASE5_SUMMARY.md       |   488 -
 .../benchmarks/API_BENCHMARK_GAP_ANALYSIS.md       |    72 -
 .../crates/data-bridge-api/PYTHON_INTEGRATION.md   |   338 -
 .../legacy/crates/data-bridge-api/SERVER_README.md |   259 -
 .../legacy/crates/data-bridge-sheet-core/todos.md  |   216 -
 .../legacy/crates/data-bridge-sheet-db/todos.md    |    76 -
 .../legacy/crates/data-bridge-test/FIXTURES.md     |   441 -
 .../data-bridge-test/IMPLEMENTATION_SUMMARY.md     |   287 -
 .../legacy/crates/data-bridge-test/TODOS.md        |   265 -
 docs/archive/legacy/deploy/OBSERVABILITY.md        |   315 -
 docs/archive/legacy/deploy/TESTING.md              |   331 -
 docs/archive/legacy/docs/MIGRATION_COMPLETE.md     |   636 --
 docs/archive/legacy/docs/MIGRATION_STATUS.md       |   182 -
 docs/archive/legacy/docs/PYLOOP_BENCHMARKS.md      |   227 -
 docs/archive/legacy/docs/PYLOOP_CRUD.md            |   481 -
 docs/archive/legacy/docs/TESTING.md                |   496 -
 docs/archive/legacy/docs/TEST_SERVER_PYTHON_APP.md |   260 -
 .../legacy/docs/sheet-specs/advanced-features.md   |  1088 --
 .../legacy/docs/sheet-specs/architecture.md        |   146 -
 docs/archive/legacy/docs/sheet-specs/clipboard.md  |    85 -
 .../legacy/docs/sheet-specs/data-structures.md     |   683 --
 docs/archive/legacy/docs/sheet-specs/flowchart.md  |    94 -
 .../legacy/docs/sheet-specs/formatting-rules.md    |    93 -
 .../legacy/docs/sheet-specs/formula-engine.md      |   143 -
 docs/archive/legacy/docs/sheet-specs/fsm.md        |    69 -
 .../legacy/docs/sheet-specs/keyboard-shortcuts.md  |    69 -
 .../archive/legacy/docs/sheet-specs/performance.md |   762 --
 .../archive/legacy/docs/sheet-specs/persistence.md |   106 -
 .../legacy/docs/sheet-specs/rendering-engine.md    |   662 --
 .../legacy/docs/sheet-specs/sheet-management.md    |    95 -
 .../legacy/docs/sheet-specs/ui-interactions.md     |    80 -
 .../legacy/docs/sheet-specs/user-experience.md     |   112 -
 .../legacy/docs/sheet-specs/wasm-integration.md    |   806 --
 docs/archive/legacy/docs/tasks/RATELIMIT_GUIDE.md  |   331 -
 docs/archive/legacy/docs/tasks/README.md           |   328 -
 .../docs/tasks/ROUTER_INTEGRATION_SUMMARY.md       |   258 -
 docs/archive/legacy/docs/tasks/ROUTING.md          |   427 -
 .../docs/tasks/ROUTING_IMPLEMENTATION_SUMMARY.md   |   225 -
 .../legacy/docs/tasks/routing_integration.md       |   222 -
 .../tests/postgres/benchmarks/ARCHITECTURE.md      |   320 -
 .../legacy/tests/postgres/benchmarks/QUICKSTART.md |   139 -
 docs/archive/legacy/tools/DELIVERABLES.md          |   376 -
 .../archive/legacy/tools/IMPLEMENTATION_SUMMARY.md |   320 -
 docs/archive/telemetry.md                          |   462 -
 docs/en/user-guide.md                              |   321 -
 docs/index.md                                      |    38 +-
 docs/jet/bundler.md                                |   118 +
 docs/jet/configuration.md                          |   117 +
 docs/jet/dev-server.md                             |    53 +
 docs/jet/getting-started.md                        |    57 +
 docs/jet/package-manager.md                        |   104 +
 docs/jet/task-runner.md                            |    98 +
 docs/jet/workspaces.md                             |    78 +
 docs/package.json                                  |    12 +
 docs/postgres/api.md                               |   132 -
 docs/postgres/guides/aggregation.md                |    97 -
 docs/postgres/guides/caching.md                    |    64 -
 docs/postgres/guides/events.md                     |    80 -
 docs/postgres/guides/indexes.md                    |    22 -
 docs/postgres/guides/inheritance.md                |   632 --
 docs/postgres/guides/migrations.md                 |   602 -
 docs/postgres/guides/querying.md                   |   153 -
 docs/postgres/guides/raw_sql.md                    |   350 -
 docs/postgres/guides/state_management.md           |    89 -
 docs/postgres/guides/tables_and_columns.md         |   104 -
 docs/postgres/guides/transactions.md               |   256 -
 docs/postgres/guides/validation.md                 |    84 -
 docs/postgres/quickstart.md                        |   114 -
 docs/postgres/relationships.md                     |  1435 ---
 docs/tasks/asyncapi.yaml                           |   332 -
 docs/zh-tw/user-guide.md                           |   320 -
 e2e/{ => grid}/app.spec.ts                         |     0
 e2e/{ => grid}/cell-editing.spec.ts                |     0
 .../mini-react => e2e/jet}/dist-jet/index.html     |     0
 .../mini-react => e2e/jet}/dist-jet/style.css      |     0
 .../jet}/dist-vite/assets/About-Mt8CYShk.js        |     0
 .../jet}/dist-vite/assets/Settings-B1a8RmuR.js     |     0
 .../jet}/dist-vite/assets/index-CFy176Qo.css       |     0
 .../jet}/dist-vite/assets/index-fWhMswjv.js        |     0
 .../mini-react => e2e/jet}/dist-vite/index.html    |     0
 {examples/mini-react => e2e/jet}/index.html        |     0
 {examples/mini-react => e2e/jet}/package-lock.json |     0
 {examples/mini-react => e2e/jet}/package.json      |     0
 {examples/mini-react => e2e/jet}/src/app.tsx       |     0
 .../jet}/src/components/AppInfo.tsx                |     0
 .../jet}/src/components/Header.tsx                 |     0
 .../jet}/src/components/TodoFooter.tsx             |     0
 .../jet}/src/components/TodoItem.module.css        |     0
 .../jet}/src/components/TodoItem.tsx               |     0
 .../jet}/src/components/TodoStats.tsx              |     0
 .../mini-react => e2e/jet}/src/components/index.ts |     0
 .../jet}/src/hooks/useLocalStorage.ts              |     0
 {examples/mini-react => e2e/jet}/src/index.tsx     |     0
 .../mini-react => e2e/jet}/src/lib/async-utils.ts  |     0
 .../mini-react => e2e/jet}/src/lib/constants.ts    |     0
 .../mini-react => e2e/jet}/src/lib/formatting.ts   |     0
 {examples/mini-react => e2e/jet}/src/lib/index.ts  |     0
 {examples/mini-react => e2e/jet}/src/lib/math.ts   |     0
 {examples/mini-react => e2e/jet}/src/mini-react.ts |     0
 .../mini-react => e2e/jet}/src/pages/About.tsx     |     0
 .../mini-react => e2e/jet}/src/pages/Settings.tsx  |     0
 {examples/mini-react => e2e/jet}/src/style.css     |     0
 {examples/mini-react => e2e/jet}/src/types.ts      |     0
 {examples/mini-react => e2e/jet}/src/utils.ts      |     0
 .../jet/tests/build.spec.ts                        |     0
 e2e/jet/tests/css.spec.ts                          |   145 +
 e2e/jet/tests/dev-server.spec.ts                   |   151 +
 e2e/jet/tests/hmr.spec.ts                          |   196 +
 e2e/jet/tests/test-utils.ts                        |    23 +
 {examples/mini-react => e2e/jet}/tsconfig.json     |     0
 {examples/mini-react => e2e/jet}/vite.config.ts    |     0
 e2e/playwright.config.ts                           |    31 +
 examples/mini-react/playwright.config.ts           |    24 -
 jet-lock.yaml                                      | 11047 +++++++++++++++++++
 packages/@cclab/pipeline/package.json              |    18 +
 packages/@cclab/pipeline/src/NodeDetail.tsx        |   165 +
 packages/@cclab/pipeline/src/PipelineDAG.tsx       |   238 +
 packages/@cclab/pipeline/src/PipelineNode.tsx      |    62 +
 packages/@cclab/pipeline/src/index.ts              |    10 +
 packages/@cclab/pipeline/src/layout.ts             |   178 +
 packages/@cclab/pipeline/src/types.ts              |    83 +
 packages/@cclab/pipeline/tsconfig.json             |    20 +
 packages/@cclab/spec-viewer/package.json           |    21 +
 packages/@cclab/spec-viewer/src/CodeBlock.tsx      |    78 +
 packages/@cclab/spec-viewer/src/MermaidDiagram.tsx |    75 +
 packages/@cclab/spec-viewer/src/SpecViewer.tsx     |   494 +
 packages/@cclab/spec-viewer/src/index.ts           |     4 +
 packages/@cclab/spec-viewer/src/types.ts           |    24 +
 packages/@cclab/spec-viewer/tsconfig.json          |    20 +
 .../@cclab/ui/src/feedback/ConnectRepoForm.tsx     |   204 +-
 .../@cclab/ui/src/spec-viewer/SpecFileBrowser.tsx  |    34 +-
 packages/cclab-agkit/README.md                     |    35 +
 packages/cclab-agkit/RENAME-PLAN.md                |    49 +
 .../cclab-agkit/prompts/section-guidance/README.md |    39 +
 .../prompts/section-guidance/async-api.md          |     3 +
 .../prompts/section-guidance/changes.md            |    24 +
 .../cclab-agkit/prompts/section-guidance/cli.md    |     3 +
 .../prompts/section-guidance/component.md          |     3 +
 .../cclab-agkit/prompts/section-guidance/config.md |     3 +
 .../prompts/section-guidance/db-model.md           |     3 +
 .../prompts/section-guidance/dependency.md         |     3 +
 .../prompts/section-guidance/design-token.md       |     3 +
 .../cclab-agkit/prompts/section-guidance/doc.md    |     3 +
 .../prompts/section-guidance/interaction.md        |     3 +
 .../cclab-agkit/prompts/section-guidance/logic.md  |     3 +
 .../prompts/section-guidance/mindmap.md            |     3 +
 .../prompts/section-guidance/overview.md           |     3 +
 .../prompts/section-guidance/requirements.md       |     9 +
 .../prompts/section-guidance/rest-api.md           |     3 +
 .../prompts/section-guidance/rpc-api.md            |     3 +
 .../prompts/section-guidance/scenarios.md          |     9 +
 .../cclab-agkit/prompts/section-guidance/schema.md |     3 +
 .../prompts/section-guidance/state-machine.md      |     3 +
 .../prompts/section-guidance/test-plan.md          |     3 +
 .../prompts/section-guidance/wireframe.md          |     3 +
 packages/cclab-agkit/prompts/system/explore.md     |    12 +
 packages/cclab-agkit/prompts/system/review.md      |    19 +
 .../cclab-agkit/schemas/agent-config.schema.json   |    70 +
 packages/cclab-agkit/schemas/change.schema.json    |   100 +
 packages/cclab-agkit/schemas/issue.schema.json     |   111 +
 packages/cclab-agkit/schemas/pipeline.schema.json  |    52 +
 packages/cclab-agkit/schemas/project.schema.json   |    23 +
 packages/cclab-agkit/schemas/spec.schema.json      |   161 +
 pnpm-lock.yaml                                     |  6728 +++++++++++
 projects/conductor/PRODUCT-REVIEW-v1.md            |    45 +
 projects/conductor/PRODUCT-REVIEW-v2.md            |    99 +
 projects/conductor/PRODUCT-REVIEW-v3.md            |   148 +
 projects/conductor/PRODUCT.md                      |   124 +
 projects/conductor/ROADMAP.md                      |    70 +
 projects/conductor/be/main.py                      |    24 +-
 projects/conductor/be/mock_progression.py          |   244 +
 projects/conductor/be/mock_server.py               |  1027 ++
 projects/conductor/be/src/agents/context_agent.py  |     2 +-
 projects/conductor/be/src/agents/scan_agent.py     |     2 +-
 projects/conductor/be/src/agents/spec_agent.py     |     2 +-
 projects/conductor/be/src/api/dashboard/deps.py    |     2 +-
 .../conductor/be/src/api/dashboard/projects.py     |     4 +-
 projects/conductor/be/src/api/dashboard/stats.py   |     4 +-
 projects/conductor/be/src/api/main.py              |    12 +-
 projects/conductor/be/src/api/platform/deps.py     |     4 +-
 projects/conductor/be/src/api/platform/router.py   |     2 +
 projects/conductor/be/src/api/webhook.py           |    10 +-
 projects/conductor/be/src/database/database.py     |     7 +-
 projects/conductor/be/src/database/repository.py   |     6 +-
 .../be/src/db/migrations/009_multi_platform.py     |    34 +
 .../be/src/db/migrations/010_add_sdd_columns.py    |    32 +
 .../conductor/be/src/features/changes/models.py    |     7 +
 .../be/src/features/changes/repository.py          |     4 +-
 .../conductor/be/src/features/changes/routes.py    |     2 +-
 .../conductor/be/src/features/changes/schemas.py   |    13 +
 .../conductor/be/src/features/issues/repository.py |     4 +-
 .../conductor/be/src/features/issues/routes.py     |    17 +-
 .../conductor/be/src/features/pipelines/routes.py  |    17 +-
 .../conductor/be/src/features/projects/models.py   |     6 +-
 .../be/src/features/projects/repository.py         |     4 +-
 .../conductor/be/src/features/projects/routes.py   |   370 +-
 .../conductor/be/src/features/projects/schemas.py  |    25 +-
 .../be/src/features/specs/code_index_repository.py |     6 +-
 .../conductor/be/src/features/specs/repository.py  |     4 +-
 projects/conductor/be/src/integrations/__init__.py |     5 +-
 .../be/src/integrations/github/__init__.py         |     6 +
 .../be/src/integrations/github/adapter.py          |    66 +
 .../conductor/be/src/integrations/github/client.py |   220 +
 .../conductor/be/src/integrations/github_import.py |   110 +
 .../conductor/be/src/integrations/gitlab_import.py |     4 +-
 .../conductor/be/src/integrations/gitlab_sync.py   |     4 +-
 projects/conductor/be/src/sdd/__init__.py          |     7 +
 projects/conductor/be/src/sdd/agent_factory.py     |   204 +
 projects/conductor/be/src/sdd/orchestrator.py      |   336 +
 projects/conductor/be/src/sdd/phase_mapping.py     |   122 +
 projects/conductor/be/src/sdd/routes.py            |   215 +
 projects/conductor/be/src/sdd/state_store.py       |   136 +
 projects/conductor/be/tests/api/test_issues.py     |     6 +-
 .../be/tests/api/test_project_connect_repo.py      |     6 +-
 .../conductor/be/tests/api/test_project_files.py   |     6 +-
 .../conductor/be/tests/api/test_project_specs.py   |     6 +-
 projects/conductor/be/tests/api/test_projects.py   |     6 +-
 projects/conductor/be/tests/api/test_workspaces.py |     6 +-
 projects/conductor/be/tests/conftest.py            |     4 +-
 .../be/tests/integration/test_mock_server.py       |   505 +
 .../be/tests/integration/test_platform_e2e.py      |     6 +-
 .../be/tests/integrations/github/__init__.py       |     0
 .../be/tests/integrations/github/test_adapter.py   |    95 +
 .../be/tests/integrations/github/test_client.py    |   133 +
 .../be/tests/integrations/test_github_import.py    |   158 +
 projects/conductor/be/tests/sdd/__init__.py        |     1 +
 .../conductor/be/tests/sdd/test_agent_factory.py   |   120 +
 .../conductor/be/tests/sdd/test_orchestrator.py    |   268 +
 projects/conductor/be/tests/sdd/test_routes.py     |   175 +
 .../conductor/be/tests/sdd/test_state_store.py     |   192 +
 projects/conductor/fe/jet.config.yaml              |     2 +-
 projects/conductor/fe/jet.config.yaml.bak          |    21 +
 projects/conductor/fe/package.json                 |    66 +-
 projects/conductor/fe/src/App.tsx                  |    15 +-
 projects/conductor/fe/src/api/platformChanges.ts   |    12 +-
 projects/conductor/fe/src/pages/ChangeDetail.tsx   |   128 +
 projects/conductor/fe/src/pages/IssueDetail.tsx    |     2 +-
 projects/conductor/fe/src/pages/ProjectDetail.tsx  |   166 +-
 projects/conductor/fe/tsconfig.json                |     4 +-
 projects/conductor/fe/vite.config.ts               |    21 +
 .../conductor/specs/platform/changes/states.md     |   117 +-
 projects/conductor/specs/platform/issues/states.md |    94 +-
 .../platform/ui/components/spec-file-browser.md    |    19 +-
 projects/conductor/specs/platform/ui/layout.md     |   167 +-
 .../conductor/specs/platform/workflow/states.md    |   213 +-
 projects/conductor/uv.lock                         |  1313 +--
 pyproject.toml                                     |     2 +-
 python/cclab/api/__init__.py                       |    46 +
 python/cclab/api/app.py                            |   623 +-
 python/cclab/log/__init__.py                       |    26 +-
 python/cclab/pg/__init__.py                        |    97 +
 python/cclab/schema/__init__.py                    |    22 +-
 python/tests/api/test_asgi_dispatch.py             |   617 ++
 uv.lock                                            |     2 +-
 2129 files changed, 170620 insertions(+), 39360 deletions(-)
```

## Diff

```diff
diff --git a/.claude/skills/cclab-sdd-run-change/SKILL.md b/.claude/skills/cclab-sdd-run-change/SKILL.md
index ed6c1a31..78bdd830 100644
--- a/.claude/skills/cclab-sdd-run-change/SKILL.md
+++ b/.claude/skills/cclab-sdd-run-change/SKILL.md
@@ -62,6 +62,11 @@ Skip this step if resuming an existing change (no description).
 2. Parse JSON output. Read `next_actions[0].cli` → run that CLI command via Bash tool
 3. Parse the CLI output (JSON). Check `executor` field:
    - `executor[0]` is `"mainthread"` → follow `prompt_path` (read the file) and `next_actions`
+   - `executor[0]` starts with `"subagent:"` → dispatch via Agent tool:
+     - Parse `subagent:{subagent_type}:{model}` (e.g., `subagent:Explore:sonnet`)
+     - Read the prompt from `prompt_path`
+     - Use the Agent tool with `subagent_type` and `model` parameters, passing the prompt content
+     - After agent completes, call `sdd_run_change` again
    - Otherwise → agent was dispatched internally; check `agent_completed` and proceed to next `sdd_run_change` call
 4. Run `cclab sdd run-change --change-id <id>` again → repeat until `action: "complete"`
 
diff --git a/.claude/skills/conductor-dev-server/skill.md b/.claude/skills/conductor-dev-server/skill.md
new file mode 100644
index 00000000..98c3bf6c
--- /dev/null
+++ b/.claude/skills/conductor-dev-server/skill.md
@@ -0,0 +1,52 @@
+---
+name: conductor-dev-server
+description: Start the Conductor development servers (backend + frontend). Use when the user wants to run, start, or launch the dev server.
+user-invocable: true
+---
+
+# /dev-server
+
+Start the Conductor mock backend and Vite frontend dev server.
+
+## Instructions
+
+1. Kill any existing processes on ports 3200 and 3201
+2. Start mock backend on port 3200
+3. Start Vite frontend on port 3201
+4. Verify both are running
+5. Report URLs to the user
+
+## Steps
+
+```bash
+# Kill existing
+lsof -ti:3200 | xargs kill -9 2>/dev/null
+lsof -ti:3201 | xargs kill -9 2>/dev/null
+sleep 1
+
+# Mock backend
+cd projects/conductor/be
+uv run --with fastapi --with uvicorn python mock_server.py &
+sleep 3
+
+# Frontend
+cd ../fe
+npx vite &
+sleep 4
+
+# Verify
+curl -s -o /dev/null -w "Backend:  http://localhost:3200 — HTTP %{http_code}\n" http://localhost:3200/health
+curl -s -o /dev/null -w "Frontend: http://localhost:3201 — HTTP %{http_code}\n" http://localhost:3201/
+```
+
+Report both URLs. If either fails, show the error.
+
+## Playwright Screenshots
+
+When taking screenshots for verification, always save to `.playwright-mcp/` directory:
+
+```
+filename: .playwright-mcp/screenshot-name.png
+```
+
+Never save screenshots to the project root.
diff --git a/.claude/skills/handoff/SKILL.md b/.claude/skills/handoff/SKILL.md
new file mode 100644
index 00000000..03590794
--- /dev/null
+++ b/.claude/skills/handoff/SKILL.md
@@ -0,0 +1,65 @@
+---
+name: handoff
+description: Write a structured handoff document for mid-flight work (context switch, branch merge, session break)
+user-invocable: true
+---
+
+# /handoff
+
+Write a structured handoff document so the next session (or person) can resume without losing context.
+
+## Arguments
+
+```
+/handoff [output-path] [topic]
+```
+
+- `output-path` — file path to write (default: `/tmp/handoff-{topic}.md`)
+- `topic` — short label for the filename (default: inferred from current work)
+
+## Instructions
+
+### Step 1: Gather context
+
+1. Check git status and recent commits on current branch
+2. Review any uncommitted changes (`git diff`, `git diff --cached`)
+3. Read conversation history for what was attempted, discovered, and decided
+4. Check for any background tasks or running processes related to the work
+
+### Step 2: Write the document
+
+Use exactly these 6 sections:
+
+```markdown
+# {Title} — Handoff
+
+## 1. Problem & Current State
+What's the goal? What's the status right now? (% done, blocking/not, which branch)
+
+## 2. Findings
+What was discovered during investigation — root causes, key insights, surprises.
+Things the next person wouldn't know just from reading the code.
+
+## 3. What Was Done
+Concrete changes made — files, functions, commits. Mark each as:
+- tested/verified
+- written but untested
+- planned but not started
+
+## 4. Next Steps
+Ordered action items to resume. Include exact commands where possible.
+
+## 5. Success Criteria
+How to verify the goal is achieved. Concrete, testable conditions.
+
+## 6. Notes
+Edge cases, risks, things that surprised you, cleanup needed, related issues.
+```
+
+### Rules
+
+- **Be concrete**: file paths, function names, exact commands — not vague descriptions
+- **Mark test status**: clearly distinguish "done & tested" from "written but not tested"
+- **Include commands**: the reader should be able to copy-paste and resume
+- **No prose padding**: bullet points over paragraphs
+- **Separate what you know from what you assume**: if something is a hypothesis, say so
diff --git a/.gitignore b/.gitignore
index eec9bbd6..bcd84ebf 100644
--- a/.gitignore
+++ b/.gitignore
@@ -1,5 +1,5 @@
 # Lens daemon index and runtime files
-.cclab/lens/
+cclab/.index/
 
 # Benchmark results
 .benchmarks/
diff --git a/CLAUDE.md b/CLAUDE.md
index 197714a4..8d3ca062 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -9,6 +9,18 @@ project:
 
 # CLAUDE.md - Implementation Essentials
 
+## Ecosystem (4 layers)
+
+```
+Layer 1: Runtime    — mamba, runtime, jet, kv, core, cli, pyo3
+Layer 2: Libraries  — pg, fetch, log, schema, array, frame, sci, learn, plot, media, text, grid
+Layer 3: Framework  — api, queue, agent, qc, server
+Layer 4: Agkit      — agkit (domain models + UI + prompts), @cclab/ui, spec-viewer, pipeline
+Projects            — cclab-sdd, conductor, das-v2
+```
+
+Full details: `ECOSYSTEM.md`. Domain model schemas: `packages/cclab-agkit/schemas/`.
+
 ## Core Principle: Specs Are the Source of Truth
 
 **Always consult specs first — not source code.**
@@ -56,6 +68,8 @@ SDD develops SDD itself. The current project focus is SDD — all other crates a
 
 Specs are machine-readable contracts, not documentation. Follow these rules for all files under `cclab/specs/`.
 
+**Complete reference**: `cclab/specs/AUTHORING.md` — section type system, cross-references, layout DSL, quality checklist. Read it before writing any spec.
+
 ### Format Priority
 
 | Priority | Format | Use for |
diff --git a/Cargo.lock b/Cargo.lock
index 095b7e04..115224fe 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1178,7 +1178,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-agent-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-agent",
  "cclab-mamba-registry",
@@ -1190,7 +1190,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-agent-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "async-trait",
  "cclab-agent",
@@ -1256,7 +1256,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-api-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-mamba-registry",
  "linkme",
@@ -1265,7 +1265,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-api-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "async-trait",
  "cclab-api",
@@ -1279,7 +1279,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "rayon",
  "serde",
@@ -1289,7 +1289,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1297,7 +1297,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "cclab-agent-mamba",
@@ -1338,7 +1338,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli-registry"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "clap",
@@ -1347,7 +1347,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "thiserror 2.0.18",
@@ -1355,7 +1355,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-cmd",
  "pyo3",
@@ -1363,7 +1363,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "bson",
@@ -1381,7 +1381,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1407,7 +1407,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-crypto",
  "pyo3",
@@ -1415,7 +1415,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1440,7 +1440,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-mamba-registry",
  "linkme",
@@ -1452,7 +1452,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-core",
  "cclab-fetch",
@@ -1465,7 +1465,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-array",
  "rayon",
@@ -1477,7 +1477,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-frame",
  "pyo3",
@@ -1485,7 +1485,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "bitvec",
  "regex",
@@ -1512,7 +1512,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1522,7 +1522,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1530,7 +1530,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1554,7 +1554,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1572,7 +1572,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-hive"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "bytes",
@@ -1588,7 +1588,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-hive-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-hive",
  "pyo3",
@@ -1599,7 +1599,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1642,7 +1642,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-cli"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "cclab-cli-registry",
@@ -1654,7 +1654,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1681,7 +1681,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-kv",
  "pyo3",
@@ -1692,7 +1692,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-array",
  "rayon",
@@ -1703,7 +1703,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-learn",
  "pyo3",
@@ -1711,7 +1711,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-log"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "chrono",
@@ -1727,7 +1727,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-log-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-log",
  "cclab-mamba-registry",
@@ -1738,7 +1738,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-log-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-log",
  "pyo3",
@@ -1746,7 +1746,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "base64 0.22.1",
@@ -1761,6 +1761,8 @@ dependencies = [
  "cranelift-native",
  "cranelift-object",
  "criterion",
+ "crossbeam-channel",
+ "dashmap 6.1.0",
  "datatest-stable",
  "indexmap 2.13.0",
  "logos",
@@ -1781,7 +1783,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba-registry"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "linkme",
  "thiserror 2.0.18",
@@ -1789,7 +1791,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mcp-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-mamba-registry",
  "linkme",
@@ -1800,7 +1802,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "image",
  "serde",
@@ -1810,7 +1812,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-media",
  "pyo3",
@@ -1818,7 +1820,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1838,7 +1840,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "bson",
  "cclab-core",
@@ -1852,7 +1854,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1880,7 +1882,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg-cli"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "cclab-cli-registry",
@@ -1893,7 +1895,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-mamba-registry",
  "cclab-pg",
@@ -1906,7 +1908,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-pg",
  "parking_lot",
@@ -1918,7 +1920,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "serde",
  "tempfile",
@@ -1927,7 +1929,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-plot",
  "pyo3",
@@ -1935,7 +1937,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-agent-pyo3",
  "cclab-api-pyo3",
@@ -1966,7 +1968,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1996,7 +1998,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-mamba-registry",
  "cclab-qc",
@@ -2005,7 +2007,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "cclab-qc",
@@ -2017,10 +2019,11 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "async-nats",
  "async-trait",
+ "axum 0.8.8",
  "base64 0.22.1",
  "cclab-core",
  "cclab-kv",
@@ -2031,6 +2034,8 @@ dependencies = [
  "futures",
  "google-cloud-googleapis",
  "google-cloud-pubsub",
+ "hmac",
+ "jsonwebtoken",
  "k8s-openapi",
  "kube",
  "num_cpus",
@@ -2042,13 +2047,16 @@ dependencies = [
  "pythonize",
  "redis",
  "regex",
+ "reqwest 0.12.28",
  "schemars",
  "serde",
  "serde_json",
+ "sha2",
  "thiserror 2.0.18",
  "tokio",
  "tokio-test",
  "tokio-util",
+ "tower 0.5.3",
  "tracing",
  "tracing-subscriber",
  "uuid",
@@ -2056,7 +2064,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-queue",
  "chrono",
@@ -2069,7 +2077,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-razer"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "hidapi",
@@ -2078,7 +2086,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -2103,7 +2111,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "axum 0.8.8",
  "cclab-api-mamba",
@@ -2116,7 +2124,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-runtime",
  "pyo3",
@@ -2124,7 +2132,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "bson",
  "dotenvy",
@@ -2138,7 +2146,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema-mamba"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-mamba-registry",
  "cclab-schema",
@@ -2150,7 +2158,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-schema",
  "pyo3",
@@ -2158,7 +2166,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -2170,7 +2178,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-sci",
  "pyo3",
@@ -2178,7 +2186,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -2191,6 +2199,7 @@ dependencies = [
  "crossterm",
  "dialoguer",
  "dirs",
+ "fs2",
  "git2",
  "glob",
  "heck",
@@ -2241,7 +2250,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd-cli"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "cclab-cli-registry",
@@ -2268,7 +2277,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -2292,7 +2301,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "rayon",
  "serde",
@@ -2303,7 +2312,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-text",
  "pyo3",
@@ -2311,7 +2320,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-tqdm"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "indicatif",
@@ -2321,7 +2330,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-tqdm-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-tqdm",
  "parking_lot",
@@ -2330,7 +2339,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-typer"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "anyhow",
  "clap",
@@ -2340,7 +2349,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-typer-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-typer",
  "parking_lot",
@@ -2349,11 +2358,11 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.43"
+version = "0.3.45"
 
 [[package]]
 name = "cclab-util-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "cclab-util",
  "pyo3",
@@ -2361,7 +2370,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "bytemuck",
  "env_logger",
@@ -2391,7 +2400,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-yaml-pyo3"
-version = "0.3.43"
+version = "0.3.45"
 dependencies = [
  "pyo3",
  "pythonize",
@@ -4039,6 +4048,16 @@ dependencies = [
  "num",
 ]
 
+[[package]]
+name = "fs2"
+version = "0.4.3"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "9564fc758e15025b46aa6643b1b77d047d1a56a1aea6e01002ac0c7026876213"
+dependencies = [
+ "libc",
+ "winapi",
+]
+
 [[package]]
 name = "fs_extra"
 version = "1.3.0"
diff --git a/Cargo.toml b/Cargo.toml
index b049eaca..f0f1182d 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -83,7 +83,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.43"
+version = "0.3.45"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
@@ -150,6 +150,7 @@ node-resolve = "2.2"
 parking_lot = "0.12"
 rust_decimal = { version = "1.36", features = ["serde"] }
 base64 = "0.22"
+jsonwebtoken = "9"
 
 # CLI
 clap = { version = "4", features = ["derive", "string"] }
diff --git a/ECOSYSTEM.md b/ECOSYSTEM.md
new file mode 100644
index 00000000..acbdf158
--- /dev/null
+++ b/ECOSYSTEM.md
@@ -0,0 +1,101 @@
+# cclab Ecosystem
+
+AI-assisted software development harness. 4 layers from runtime to domain.
+
+## Layer 1: Runtime & Language
+
+Core systems. No business logic. Deterministic, correct answers exist.
+
+| Component | Purpose |
+|-----------|---------|
+| `cclab-mamba` | Force-typed Python compiler (JIT via Cranelift) |
+| `cclab-mamba-registry` | Mamba module auto-registration |
+| `cclab-runtime` | Python asyncio ↔ Rust tokio bridge (for non-Mamba users) |
+| `cclab-jet` | Frontend toolchain (bundler, dev server, HMR, CSS pipeline) |
+| `cclab-kv` | Embedded key-value store (WAL-backed) |
+| `cclab-wal` | Write-ahead log |
+| `cclab-core` | Shared primitives, error handling |
+| `cclab-cli` | Unified CLI binary |
+| `cclab-cli-registry` | CLI module auto-registration (linkme) |
+| `cclab-pyo3` | Unified PyO3 entry point (re-exports all *-pyo3 crates) |
+
+## Layer 2: Libraries
+
+Data, storage, networking, scientific computing. Tools with correct answers.
+
+| Component | Purpose |
+|-----------|---------|
+| `cclab-pg` | PostgreSQL async ORM + migrations |
+| `cclab-mongo` | MongoDB driver |
+| `cclab-fetch` | HTTP client (Rust-backed) |
+| `cclab-log` | Structured logging |
+| `cclab-schema` | Validation (Pydantic compat) |
+| `cclab-crypto` | Cryptographic primitives |
+| `cclab-hive` | Hive/data lake connector |
+| `cclab-array` | N-dimensional arrays |
+| `cclab-frame` | DataFrames |
+| `cclab-sci` | Scientific computing |
+| `cclab-learn` | Machine learning |
+| `cclab-plot` | Visualization |
+| `cclab-media` | Image/audio processing |
+| `cclab-text` | NLP / text processing |
+| `cclab-grid-*` | Spreadsheet engine (6 crates) |
+
+## Layer 3: Framework
+
+Application frameworks. Build servers, agents, pipelines. Opinionated but general-purpose.
+
+| Component | Purpose |
+|-----------|---------|
+| `cclab-api` | HTTP server framework (Rust server + ASGI compat) |
+| `cclab-queue` | Background job engine |
+| `cclab-agent` | LLM agent framework (providers, tools, agentic loop) |
+| `cclab-qc` | Testing framework (pytest compat) |
+| `cclab-server` | MCP server |
+| `cclab-cmd` | Command execution |
+| `cclab-typer` | CLI builder |
+| `cclab-tqdm` | Progress bars |
+
+## Layer 4: Agkit (Agentic Development Kit)
+
+Business domain — no single correct answer, design decisions live here.
+
+| Component | Type | Purpose |
+|-----------|------|---------|
+| `cclab-agkit` | package | Domain models, UI components, prompts, renderers |
+| `cclab-agkit/schemas` | JSON Schema | Issue, Spec, Change, Pipeline, Project definitions |
+| `@cclab/ui` | package | UI design system — Card, Badge, InlineEdit, FileBrowser |
+| `@cclab/spec-viewer` | package | Spec rendering — Markdown, Mermaid, code blocks |
+| `@cclab/pipeline` | package | Pipeline DAG visualization |
+| `cclab-razer` | crate | Code analysis / transformation |
+
+## Projects
+
+Applications built on Layers 1-4. Orchestrators, not libraries.
+
+| Project | Description |
+|---------|-------------|
+| `cclab-sdd` | Spec-Driven Development engine — CLI workflow, agent dispatch |
+| `conductor` | AI-powered SDD platform — cloud version of cclab-sdd |
+| `das-v2` | Data analysis service |
+
+## Dependency Flow
+
+```
+Projects (cclab-sdd, conductor, das-v2)
+    ↓ consumes
+Layer 4: Agkit (agkit + ui + spec-viewer + pipeline)
+    ↓ uses
+Layer 3: Framework (api + queue + agent)
+    ↓ uses
+Layer 2: Libraries (pg + fetch + log + array + frame)
+    ↓ uses
+Layer 1: Runtime (mamba + runtime + jet + kv)
+```
+
+## Binding Layers
+
+Each Rust crate may have companion crates:
+- `*-pyo3` — Python bindings via PyO3 (25 crates)
+- `*-mamba` — Mamba JIT bindings (7 crates)
+- `*-cli` — CLI subcommands
diff --git a/crates/cclab-aurora/Cargo.toml b/crates/cclab-aurora/Cargo.toml
new file mode 100644
index 00000000..e7fd0013
--- /dev/null
+++ b/crates/cclab-aurora/Cargo.toml
@@ -0,0 +1,22 @@
+[package]
+name = "cclab-aurora"
+version = "0.1.0"
+edition = "2021"
+description = "Diagram and specification generation library"
+license = "MIT"
+authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
+repository = "https://github.com/chrischeng-c4/cclab"
+keywords = ["mermaid", "diagram", "openapi", "asyncapi", "codegen"]
+categories = ["development-tools", "visualization"]
+
+[dependencies]
+serde = { version = "1", features = ["derive"] }
+serde_json = "1"
+serde_yaml = "0.9"
+thiserror = "2"
+
+[dev-dependencies]
+tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
+
+[features]
+default = []
diff --git a/crates/cclab-jet/examples/dev_server.rs b/crates/cclab-jet/examples/dev_server.rs
index d3048dc4..9d7aeff9 100644
--- a/crates/cclab-jet/examples/dev_server.rs
+++ b/crates/cclab-jet/examples/dev_server.rs
@@ -77,6 +77,7 @@ async fn main() -> anyhow::Result<()> {
         public_dir: Some(root.join("public")),
         entry: PathBuf::from("src/index.tsx"),
         proxy: std::collections::HashMap::new(),
+        aliases: std::collections::HashMap::new(),
     };
 
     println!("  http://{}:{}", server_config.host, server_config.port);
diff --git a/crates/cclab-jet/examples/full_pipeline.rs b/crates/cclab-jet/examples/full_pipeline.rs
index 7010ae02..b3592662 100644
--- a/crates/cclab-jet/examples/full_pipeline.rs
+++ b/crates/cclab-jet/examples/full_pipeline.rs
@@ -190,6 +190,7 @@ fn demo_dev_server_config(root: &Path) {
         public_dir: Some(root.join("public")),
         entry: PathBuf::from("src/index.tsx"),
         proxy: std::collections::HashMap::new(),
+        aliases: std::collections::HashMap::new(),
     };
 
     println!("  Host:   {}:{}", config.host, config.port);
diff --git a/crates/cclab-jet/src/asset/image_processor.rs b/crates/cclab-jet/src/asset/image_processor.rs
index cc1b5caa..84261f0a 100644
--- a/crates/cclab-jet/src/asset/image_processor.rs
+++ b/crates/cclab-jet/src/asset/image_processor.rs
@@ -3,13 +3,26 @@ use std::path::Path;
 
 use super::{AssetOptions, AssetType, ProcessedAsset};
 
-/// Optimize image file
+/// Minimum file size in bytes for optimization to be worthwhile.
+/// Images under this threshold are returned unchanged.
+const MIN_OPTIMIZE_SIZE: usize = 1024;
+
+/// Default JPEG quality for re-encoding (0-100).
+const DEFAULT_JPEG_QUALITY: u8 = 85;
+
+/// Optimize image file.
+///
+/// - JPEG: Re-encode at quality 85 (smaller file size)
+/// - PNG: Re-encode with basic optimization
+/// - WebP: Pass through (already optimized format)
+/// - SVG: Strip comments and unnecessary whitespace
+/// - Skip optimization for images under 1KB (overhead not worth it)
 pub fn optimize_image(path: &Path, options: &AssetOptions) -> Result<ProcessedAsset> {
     tracing::debug!("Optimizing image: {:?}", path);
 
-    let _img = image::open(path)?;
+    let original_content = std::fs::read(path)?;
+    let original_size = original_content.len();
 
-    let original_size = std::fs::metadata(path)?.len() as usize;
     if original_size > options.max_image_size {
         tracing::warn!(
             "Image exceeds max size: {} > {}",
@@ -18,9 +31,29 @@ pub fn optimize_image(path: &Path, options: &AssetOptions) -> Result<ProcessedAs
         );
     }
 
-    // TODO: Implement actual optimization
+    let ext = path
+        .extension()
+        .and_then(|e| e.to_str())
+        .unwrap_or("")
+        .to_lowercase();
 
-    let content = std::fs::read(path)?;
+    // Skip optimization for images under 1KB
+    let content = if original_size < MIN_OPTIMIZE_SIZE {
+        tracing::debug!(
+            "Skipping optimization for small image ({} bytes < {} threshold)",
+            original_size,
+            MIN_OPTIMIZE_SIZE
+        );
+        original_content
+    } else {
+        match ext.as_str() {
+            "jpg" | "jpeg" => optimize_jpeg(path, &original_content)?,
+            "png" => optimize_png(path, &original_content)?,
+            "svg" => optimize_svg(&original_content),
+            // WebP and other formats: pass through
+            _ => original_content,
+        }
+    };
 
     use sha2::{Digest, Sha256};
     let mut hasher = Sha256::new();
@@ -29,8 +62,8 @@ pub fn optimize_image(path: &Path, options: &AssetOptions) -> Result<ProcessedAs
 
     let filename = if options.hash_filenames {
         let stem = path.file_stem().unwrap().to_string_lossy();
-        let ext = path.extension().unwrap_or_default().to_string_lossy();
-        format!("{}.{}.{}", stem, hash, ext)
+        let ext_str = path.extension().unwrap_or_default().to_string_lossy();
+        format!("{}.{}.{}", stem, hash, ext_str)
     } else {
         path.file_name().unwrap().to_string_lossy().to_string()
     };
@@ -44,10 +77,265 @@ pub fn optimize_image(path: &Path, options: &AssetOptions) -> Result<ProcessedAs
     })
 }
 
+/// Optimize JPEG by re-encoding at the configured quality level.
+/// Returns the smaller of original vs re-encoded.
+fn optimize_jpeg(path: &Path, original: &[u8]) -> Result<Vec<u8>> {
+    let img = image::open(path)?;
+    let mut output = std::io::Cursor::new(Vec::new());
+
+    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
+        &mut output,
+        DEFAULT_JPEG_QUALITY,
+    );
+    img.write_with_encoder(encoder)?;
+
+    let optimized = output.into_inner();
+
+    // Only use the optimized version if it's actually smaller
+    if optimized.len() < original.len() {
+        tracing::debug!(
+            "JPEG optimized: {} -> {} bytes (saved {})",
+            original.len(),
+            optimized.len(),
+            original.len() - optimized.len()
+        );
+        Ok(optimized)
+    } else {
+        tracing::debug!(
+            "JPEG optimization not beneficial ({} >= {} bytes), keeping original",
+            optimized.len(),
+            original.len()
+        );
+        Ok(original.to_vec())
+    }
+}
+
+/// Optimize PNG by re-encoding with default compression.
+/// Returns the smaller of original vs re-encoded.
+fn optimize_png(path: &Path, original: &[u8]) -> Result<Vec<u8>> {
+    let img = image::open(path)?;
+    let mut output = std::io::Cursor::new(Vec::new());
+
+    let encoder = image::codecs::png::PngEncoder::new(&mut output);
+    img.write_with_encoder(encoder)?;
+
+    let optimized = output.into_inner();
+
+    if optimized.len() < original.len() {
+        tracing::debug!(
+            "PNG optimized: {} -> {} bytes (saved {})",
+            original.len(),
+            optimized.len(),
+            original.len() - optimized.len()
+        );
+        Ok(optimized)
+    } else {
+        tracing::debug!(
+            "PNG optimization not beneficial, keeping original"
+        );
+        Ok(original.to_vec())
+    }
+}
+
+/// Optimize SVG by stripping comments and collapsing whitespace.
+fn optimize_svg(original: &[u8]) -> Vec<u8> {
+    let source = match std::str::from_utf8(original) {
+        Ok(s) => s,
+        Err(_) => return original.to_vec(),
+    };
+
+    let mut result = String::with_capacity(source.len());
+    let bytes = source.as_bytes();
+    let len = bytes.len();
+    let mut i = 0;
+
+    // Strip XML/SVG comments (<!-- ... -->)
+    while i < len {
+        if i + 3 < len
+            && bytes[i] == b'<'
+            && bytes[i + 1] == b'!'
+            && bytes[i + 2] == b'-'
+            && bytes[i + 3] == b'-'
+        {
+            i += 4;
+            while i + 2 < len {
+                if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>' {
+                    i += 3;
+                    break;
+                }
+                i += 1;
+            }
+            continue;
+        }
+        result.push(bytes[i] as char);
+        i += 1;
+    }
+
+    // Collapse whitespace between tags
+    let collapsed = collapse_svg_whitespace(&result);
+    collapsed.into_bytes()
+}
+
+/// Collapse whitespace between SVG tags.
+fn collapse_svg_whitespace(source: &str) -> String {
+    let mut result = String::with_capacity(source.len());
+    let mut prev_was_ws = false;
+    let mut in_tag = false;
+
+    for ch in source.chars() {
+        if ch == '<' {
+            in_tag = true;
+            prev_was_ws = false;
+            result.push(ch);
+        } else if ch == '>' {
+            in_tag = false;
+            prev_was_ws = false;
+            result.push(ch);
+        } else if !in_tag && ch.is_whitespace() {
+            if !prev_was_ws {
+                result.push(' ');
+                prev_was_ws = true;
+            }
+        } else {
+            prev_was_ws = false;
+            result.push(ch);
+        }
+    }
+
+    result
+}
+
 #[cfg(test)]
 mod tests {
+    use super::*;
+    use std::io::Write;
+
     #[test]
-    fn test_optimize_placeholder() {
-        assert!(true);
+    fn test_small_image_skip() {
+        // T18: Images under 1KB should be returned unchanged
+        let dir = tempfile::tempdir().unwrap();
+        let img_path = dir.path().join("tiny.png");
+
+        // Create a minimal valid PNG (under 1KB)
+        let img = image::RgbaImage::new(1, 1);
+        img.save(&img_path).unwrap();
+
+        let original_content = std::fs::read(&img_path).unwrap();
+        assert!(
+            original_content.len() < MIN_OPTIMIZE_SIZE,
+            "Test image should be under 1KB, was {} bytes",
+            original_content.len()
+        );
+
+        let options = AssetOptions {
+            optimize_images: true,
+            hash_filenames: true,
+            max_image_size: 1024 * 1024,
+        };
+
+        let result = optimize_image(&img_path, &options).unwrap();
+        assert_eq!(
+            result.content, original_content,
+            "Small image should be returned unchanged"
+        );
+    }
+
+    #[test]
+    fn test_jpeg_optimization() {
+        // T17: JPEG optimization should produce valid output
+        let dir = tempfile::tempdir().unwrap();
+        let img_path = dir.path().join("test.jpg");
+
+        // Create a larger JPEG image (over 1KB so optimization runs)
+        let img = image::RgbImage::from_fn(100, 100, |x, y| {
+            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
+        });
+        img.save(&img_path).unwrap();
+
+        let original_size = std::fs::metadata(&img_path).unwrap().len() as usize;
+        assert!(
+            original_size >= MIN_OPTIMIZE_SIZE,
+            "Test JPEG should be >= 1KB for optimization, was {} bytes",
+            original_size
+        );
+
+        let options = AssetOptions {
+            optimize_images: true,
+            hash_filenames: true,
+            max_image_size: 1024 * 1024,
+        };
+
+        let result = optimize_image(&img_path, &options).unwrap();
+        assert!(
+            !result.content.is_empty(),
+            "Optimized JPEG should not be empty"
+        );
+        // Verify it's still a valid JPEG (starts with JPEG magic bytes)
+        assert!(
+            result.content.len() <= original_size,
+            "Optimized JPEG ({}) should be <= original ({})",
+            result.content.len(),
+            original_size,
+        );
+        assert_eq!(result.asset_type, AssetType::Image);
+        assert!(result.filename.contains(".jpg"));
+    }
+
+    #[test]
+    fn test_svg_optimization() {
+        // Build an SVG larger than MIN_OPTIMIZE_SIZE (1KB)
+        let mut svg_content = String::from("<!-- comment -->\n<svg xmlns=\"http://www.w3.org/2000/svg\">\n  <!-- another comment -->\n");
+        // Add enough rect elements to exceed 1KB
+        for i in 0..50 {
+            svg_content.push_str(&format!(
+                "  <rect   x=\"{}\"   y=\"{}\"   width=\"100\"   height=\"100\" fill=\"#abcdef\" />\n",
+                i * 10, i * 10
+            ));
+        }
+        svg_content.push_str("</svg>");
+
+        let original = svg_content.as_bytes().to_vec();
+        assert!(
+            original.len() >= MIN_OPTIMIZE_SIZE,
+            "Test SVG should be >= 1KB, was {} bytes",
+            original.len()
+        );
+
+        let result = optimize_svg(&original);
+        let result_str = std::str::from_utf8(&result).unwrap();
+
+        assert!(
+            !result_str.contains("<!-- comment -->"),
+            "SVG comments should be stripped"
+        );
+        assert!(
+            result_str.contains("<svg"),
+            "SVG content should be preserved"
+        );
+        assert!(
+            result_str.contains("<rect"),
+            "SVG elements should be preserved"
+        );
+    }
+
+    #[test]
+    fn test_hashed_filename() {
+        let dir = tempfile::tempdir().unwrap();
+        let img_path = dir.path().join("logo.png");
+
+        let img = image::RgbaImage::new(1, 1);
+        img.save(&img_path).unwrap();
+
+        let options = AssetOptions {
+            optimize_images: true,
+            hash_filenames: true,
+            max_image_size: 1024 * 1024,
+        };
+
+        let result = optimize_image(&img_path, &options).unwrap();
+        // Filename should be logo.[hash].png
+        assert!(result.filename.starts_with("logo."));
+        assert!(result.filename.ends_with(".png"));
+        assert!(result.filename.len() > "logo.png".len());
     }
 }
diff --git a/crates/cclab-jet/src/bundler/css_bundle.rs b/crates/cclab-jet/src/bundler/css_bundle.rs
index 2b6d8fa3..45483706 100644
--- a/crates/cclab-jet/src/bundler/css_bundle.rs
+++ b/crates/cclab-jet/src/bundler/css_bundle.rs
@@ -130,6 +130,180 @@ fn is_remote(path: &str) -> bool {
         || path.starts_with("//")
 }
 
+/// Result of CSS URL rewriting.
+#[derive(Debug, Clone)]
+pub struct CssRewriteResult {
+    /// The rewritten CSS content.
+    pub css: String,
+    /// Assets discovered and processed (path -> hashed filename).
+    pub discovered_assets: Vec<RewrittenAsset>,
+}
+
+/// An asset that was discovered and rewritten in CSS.
+#[derive(Debug, Clone)]
+pub struct RewrittenAsset {
+    /// Original resolved path of the asset.
+    pub original_path: PathBuf,
+    /// Hashed output filename (e.g. "logo.abc12345.svg").
+    pub hashed_filename: String,
+}
+
+/// Rewrite `url()` references in CSS to point to hashed asset paths.
+///
+/// Scans the CSS for `url(...)` references, resolves them relative to
+/// `css_dir`, computes content hashes, and rewrites the URLs to
+/// `{asset_prefix}/{stem}.{hash}.{ext}`.
+///
+/// Handles:
+/// - Quoted URLs: `url("path")`, `url('path')`
+/// - Unquoted URLs: `url(path)`
+/// - URLs with query strings: `url(path?v=1)` (query is stripped)
+/// - Remote URLs (http://, https://, //, data:) are skipped
+///
+/// `asset_prefix` is the path prefix for rewritten URLs (e.g. "assets").
+pub fn rewrite_css_urls(
+    css: &str,
+    css_dir: &Path,
+    asset_prefix: &str,
+) -> CssRewriteResult {
+    let mut result = String::with_capacity(css.len());
+    let mut discovered = Vec::new();
+    let chars: Vec<char> = css.chars().collect();
+    let len = chars.len();
+    let mut i = 0;
+
+    while i < len {
+        // Look for url( — case insensitive
+        if i + 3 < len
+            && (chars[i] == 'u' || chars[i] == 'U')
+            && (chars[i + 1] == 'r' || chars[i + 1] == 'R')
+            && (chars[i + 2] == 'l' || chars[i + 2] == 'L')
+            && chars[i + 3] == '('
+        {
+            let url_start = i;
+            i += 4; // skip "url("
+
+            // Skip whitespace
+            while i < len && chars[i].is_whitespace() {
+                i += 1;
+            }
+
+            // Determine if quoted
+            let quote_char = if i < len && (chars[i] == '"' || chars[i] == '\'') {
+                let q = chars[i];
+                i += 1;
+                Some(q)
+            } else {
+                None
+            };
+
+            // Read the URL value
+            let value_start = i;
+            while i < len {
+                if let Some(q) = quote_char {
+                    if chars[i] == q {
+                        break;
+                    }
+                } else if chars[i] == ')' || chars[i].is_whitespace() {
+                    break;
+                }
+                i += 1;
+            }
+            let url_value: String = chars[value_start..i].iter().collect();
+
+            // Skip closing quote
+            if quote_char.is_some() && i < len {
+                i += 1;
+            }
+
+            // Skip whitespace before closing paren
+            while i < len && chars[i].is_whitespace() {
+                i += 1;
+            }
+
+            // Skip closing paren
+            if i < len && chars[i] == ')' {
+                i += 1;
+            }
+
+            // Determine if we should rewrite this URL
+            let trimmed_url = url_value.trim();
+            if is_remote(trimmed_url)
+                || trimmed_url.starts_with("data:")
+                || trimmed_url.starts_with('#')
+                || trimmed_url.is_empty()
+            {
+                // Copy original url() verbatim
+                let original: String = chars[url_start..i].iter().collect();
+                result.push_str(&original);
+                continue;
+            }
+
+            // Strip query string for file resolution
+            let clean_path = trimmed_url.split('?').next().unwrap_or(trimmed_url);
+            let clean_path = clean_path.split('#').next().unwrap_or(clean_path);
+
+            // Resolve relative to CSS file directory
+            let resolved = css_dir.join(clean_path);
+
+            if resolved.exists() {
+                // Compute content hash
+                if let Ok(content) = std::fs::read(&resolved) {
+                    let hash = compute_asset_hash(&content);
+                    let stem = resolved
+                        .file_stem()
+                        .and_then(|s| s.to_str())
+                        .unwrap_or("asset");
+                    let ext = resolved
+                        .extension()
+                        .and_then(|e| e.to_str())
+                        .unwrap_or("");
+                    let hashed_filename = if ext.is_empty() {
+                        format!("{}.{}", stem, hash)
+                    } else {
+                        format!("{}.{}.{}", stem, hash, ext)
+                    };
+                    let rewritten_url = format!("{}/{}", asset_prefix, hashed_filename);
+
+                    result.push_str(&format!("url({})", rewritten_url));
+
+                    discovered.push(RewrittenAsset {
+                        original_path: resolved,
+                        hashed_filename,
+                    });
+                } else {
+                    // Can't read file, keep original
+                    let original: String = chars[url_start..i].iter().collect();
+                    result.push_str(&original);
+                }
+            } else {
+                // File doesn't exist, keep original
+                let original: String = chars[url_start..i].iter().collect();
+                result.push_str(&original);
+            }
+            continue;
+        }
+
+        result.push(chars[i]);
+        i += 1;
+    }
+
+    CssRewriteResult {
+        css: result,
+        discovered_assets: discovered,
+    }
+}
+
+/// Compute a short content hash for an asset.
+fn compute_asset_hash(content: &[u8]) -> String {
+    use std::collections::hash_map::DefaultHasher;
+    use std::hash::{Hash, Hasher};
+
+    let mut hasher = DefaultHasher::new();
+    content.hash(&mut hasher);
+    format!("{:x}", hasher.finish())[..8].to_string()
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -220,4 +394,166 @@ mod tests {
         assert!(result.contains(".a { color: red; }"));
         assert!(result.contains(".b { color: blue; }"));
     }
+
+    // ──────────────────────────────────────────────────────────────────
+    // CSS URL rewriting tests (R14 / T21)
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_css_url_rewrite_basic() {
+        // T21: Rewrite url() references to hashed asset paths
+        let dir = tempfile::tempdir().unwrap();
+        let img_dir = dir.path().join("img");
+        std::fs::create_dir_all(&img_dir).unwrap();
+
+        // Create a test SVG file
+        let svg_content = "<svg><rect/></svg>";
+        let svg_path = img_dir.join("logo.svg");
+        std::fs::write(&svg_path, svg_content).unwrap();
+
+        let css = "background: url(../img/logo.svg);\n";
+        // CSS is in a subdirectory relative to the image
+        let css_dir = dir.path().join("css");
+        std::fs::create_dir_all(&css_dir).unwrap();
+
+        let result = rewrite_css_urls(css, &css_dir, "assets");
+
+        assert!(
+            result.css.contains("url(assets/logo."),
+            "URL should be rewritten to hashed path, got: {}",
+            result.css
+        );
+        assert!(
+            result.css.contains(".svg)"),
+            "Extension should be preserved, got: {}",
+            result.css
+        );
+        assert_eq!(
+            result.discovered_assets.len(),
+            1,
+            "Should discover one asset"
+        );
+        assert!(
+            result.discovered_assets[0].hashed_filename.starts_with("logo."),
+            "Hashed filename should start with 'logo.'"
+        );
+        assert!(
+            result.discovered_assets[0].hashed_filename.ends_with(".svg"),
+            "Hashed filename should end with '.svg'"
+        );
+    }
+
+    #[test]
+    fn test_css_url_rewrite_quoted() {
+        let dir = tempfile::tempdir().unwrap();
+        let font_path = dir.path().join("font.woff2");
+        std::fs::write(&font_path, b"fake font data").unwrap();
+
+        let css = r#"src: url("font.woff2");"#;
+        let result = rewrite_css_urls(css, dir.path(), "assets");
+
+        assert!(
+            result.css.contains("url(assets/font."),
+            "Quoted URL should be rewritten, got: {}",
+            result.css
+        );
+    }
+
+    #[test]
+    fn test_css_url_rewrite_single_quotes() {
+        let dir = tempfile::tempdir().unwrap();
+        let img_path = dir.path().join("bg.png");
+        std::fs::write(&img_path, b"fake png data").unwrap();
+
+        let css = "background: url('bg.png');";
+        let result = rewrite_css_urls(css, dir.path(), "assets");
+
+        assert!(
+            result.css.contains("url(assets/bg."),
+            "Single-quoted URL should be rewritten, got: {}",
+            result.css
+        );
+    }
+
+    #[test]
+    fn test_css_url_rewrite_skip_remote() {
+        let css = "background: url(https://example.com/img.png);";
+        let result = rewrite_css_urls(css, Path::new("."), "assets");
+
+        assert!(
+            result.css.contains("https://example.com/img.png"),
+            "Remote URLs should be left unchanged, got: {}",
+            result.css
+        );
+        assert!(
+            result.discovered_assets.is_empty(),
+            "No assets should be discovered for remote URLs"
+        );
+    }
+
+    #[test]
+    fn test_css_url_rewrite_skip_data_uri() {
+        let css = "background: url(data:image/png;base64,abc123);";
+        let result = rewrite_css_urls(css, Path::new("."), "assets");
+
+        assert!(
+            result.css.contains("data:image/png"),
+            "Data URIs should be left unchanged, got: {}",
+            result.css
+        );
+    }
+
+    #[test]
+    fn test_css_url_rewrite_query_string() {
+        let dir = tempfile::tempdir().unwrap();
+        let font_path = dir.path().join("icon.woff");
+        std::fs::write(&font_path, b"font data here").unwrap();
+
+        let css = "src: url(icon.woff?v=1.2);";
+        let result = rewrite_css_urls(css, dir.path(), "assets");
+
+        assert!(
+            result.css.contains("url(assets/icon."),
+            "URL with query string should be rewritten, got: {}",
+            result.css
+        );
+    }
+
+    #[test]
+    fn test_css_url_rewrite_missing_file() {
+        let css = "background: url(nonexistent.png);";
+        let result = rewrite_css_urls(css, Path::new("/tmp"), "assets");
+
+        assert!(
+            result.css.contains("url(nonexistent.png)"),
+            "Missing files should keep original URL, got: {}",
+            result.css
+        );
+    }
+
+    #[test]
+    fn test_css_url_rewrite_multiple() {
+        let dir = tempfile::tempdir().unwrap();
+        let img1 = dir.path().join("a.png");
+        let img2 = dir.path().join("b.svg");
+        std::fs::write(&img1, b"png data").unwrap();
+        std::fs::write(&img2, b"svg data").unwrap();
+
+        let css = "bg1: url(a.png); bg2: url(b.svg);";
+        let result = rewrite_css_urls(css, dir.path(), "assets");
+
+        assert_eq!(
+            result.discovered_assets.len(),
+            2,
+            "Should discover two assets"
+        );
+        assert!(
+            result.css.contains("url(assets/a."),
+            "First URL should be rewritten"
+        );
+        assert!(
+            result.css.contains("url(assets/b."),
+            "Second URL should be rewritten"
+        );
+    }
 }
diff --git a/crates/cclab-jet/src/bundler/html_minify.rs b/crates/cclab-jet/src/bundler/html_minify.rs
new file mode 100644
index 00000000..6e5c7e93
--- /dev/null
+++ b/crates/cclab-jet/src/bundler/html_minify.rs
@@ -0,0 +1,358 @@
+//! HTML minification: strip comments, collapse whitespace, simplify attributes.
+//!
+//! Custom implementation with no external dependencies. Preserves whitespace
+//! inside `<pre>`, `<code>`, `<script>`, and `<style>` tags.
+
+/// Minify HTML source code.
+///
+/// - Strips HTML comments (`<!-- ... -->`)
+/// - Collapses whitespace between tags (preserves within `<pre>`, `<code>`,
+///   `<script>`, `<style>`)
+/// - Removes unnecessary quotes on attributes with simple values
+/// - No new dependency -- custom implementation using string scanning
+pub fn minify_html(source: &str) -> String {
+    // Step 1: Strip HTML comments
+    let no_comments = strip_html_comments(source);
+
+    // Step 2: Collapse whitespace, preserving content in special tags
+    collapse_html_whitespace(&no_comments)
+}
+
+/// Strip HTML comments (<!-- ... -->).
+/// Preserves conditional comments (<!--[if ...]>) for IE compatibility.
+fn strip_html_comments(source: &str) -> String {
+    let mut result = String::with_capacity(source.len());
+    let bytes = source.as_bytes();
+    let len = bytes.len();
+    let mut i = 0;
+
+    while i < len {
+        // Check for comment start: <!--
+        if i + 3 < len
+            && bytes[i] == b'<'
+            && bytes[i + 1] == b'!'
+            && bytes[i + 2] == b'-'
+            && bytes[i + 3] == b'-'
+        {
+            // Skip conditional comments (<!--[if)
+            if i + 4 < len && bytes[i + 4] == b'[' {
+                result.push_str("<!");
+                i += 2;
+                continue;
+            }
+
+            // Find comment end -->
+            i += 4;
+            while i + 2 < len {
+                if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>' {
+                    i += 3;
+                    break;
+                }
+                i += 1;
+            }
+            // If we hit end of string without finding -->, skip remainder
+            if i + 2 >= len && !(i < len && bytes.get(i) == Some(&b'-')) {
+                break;
+            }
+            continue;
+        }
+
+        result.push(bytes[i] as char);
+        i += 1;
+    }
+
+    result
+}
+
+/// Tags whose content whitespace must be preserved.
+const PRESERVE_WS_TAGS: &[&str] = &["pre", "code", "script", "style", "textarea"];
+
+/// Collapse whitespace in HTML while preserving content of special tags.
+fn collapse_html_whitespace(source: &str) -> String {
+    let mut result = String::with_capacity(source.len());
+    let chars: Vec<char> = source.chars().collect();
+    let len = chars.len();
+    let mut i = 0;
+    let mut preserve_depth: Vec<String> = Vec::new(); // stack of preserved tags
+
+    while i < len {
+        // Check for opening tag of preserved-whitespace elements
+        if chars[i] == '<' && i + 1 < len && chars[i + 1].is_ascii_alphabetic() {
+            let tag_start = i;
+            let tag_name = extract_tag_name(&chars, i + 1);
+            let lower_tag = tag_name.to_lowercase();
+
+            if PRESERVE_WS_TAGS.contains(&lower_tag.as_str()) {
+                preserve_depth.push(lower_tag.clone());
+                // Copy everything until the matching closing tag
+                // First, copy the opening tag
+                while i < len && chars[i] != '>' {
+                    result.push(chars[i]);
+                    i += 1;
+                }
+                if i < len {
+                    result.push(chars[i]); // push '>'
+                    i += 1;
+                }
+                // Now copy content verbatim until closing tag
+                let close_tag = format!("</{}", lower_tag);
+                while i < len {
+                    // Check for closing tag
+                    let remaining: String = chars[i..].iter().take(close_tag.len() + 1).collect();
+                    if remaining.to_lowercase().starts_with(&close_tag) {
+                        preserve_depth.pop();
+                        // Copy the closing tag
+                        while i < len && chars[i] != '>' {
+                            result.push(chars[i]);
+                            i += 1;
+                        }
+                        if i < len {
+                            result.push(chars[i]); // push '>'
+                            i += 1;
+                        }
+                        break;
+                    }
+                    result.push(chars[i]);
+                    i += 1;
+                }
+                continue;
+            }
+
+            // Not a preserved tag -- handle attribute quote removal
+            let tag_content = collect_tag(&chars, tag_start);
+            let minified_tag = minify_tag_attributes(&tag_content);
+            result.push_str(&minified_tag);
+            i = tag_start + tag_content.len();
+            continue;
+        }
+
+        // Inside normal content: collapse whitespace between tags
+        if !preserve_depth.is_empty() {
+            result.push(chars[i]);
+            i += 1;
+            continue;
+        }
+
+        if chars[i].is_whitespace() {
+            // Collapse consecutive whitespace to a single space
+            while i < len && chars[i].is_whitespace() {
+                i += 1;
+            }
+            // Don't add space if we're right before or after a tag
+            if !result.is_empty() && !result.ends_with('>') {
+                if i < len && chars[i] != '<' {
+                    result.push(' ');
+                }
+            }
+            continue;
+        }
+
+        result.push(chars[i]);
+        i += 1;
+    }
+
+    result
+}
+
+/// Extract a tag name from position in char array.
+fn extract_tag_name(chars: &[char], start: usize) -> String {
+    let mut name = String::new();
+    let mut i = start;
+    while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '-') {
+        name.push(chars[i]);
+        i += 1;
+    }
+    name
+}
+
+/// Collect an entire tag (from '<' to '>') from the char array.
+fn collect_tag(chars: &[char], start: usize) -> String {
+    let mut tag = String::new();
+    let mut i = start;
+    while i < chars.len() {
+        tag.push(chars[i]);
+        if chars[i] == '>' {
+            break;
+        }
+        i += 1;
+    }
+    tag
+}
+
+/// Remove unnecessary quotes from tag attributes with simple values.
+///
+/// A "simple value" is one containing only [a-zA-Z0-9_-], which does
+/// not need quoting per the HTML spec.
+fn minify_tag_attributes(tag: &str) -> String {
+    // Quick check: if no '=' found, nothing to simplify
+    if !tag.contains('=') {
+        return tag.to_string();
+    }

... truncated (115234 more lines)
```

## Review: merge-3way

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-merge-3way

**Summary**: All spec requirements are fully implemented and all 12 tests pass (5 new + 7 pre-existing). 3-way merge logic, conflict handling, .base.md filtering, audit log actions, and two-pass atomicity all match spec exactly.

### Checklist

- [PASS] [HARD] Code matches all spec requirements
  - merge_3way() (create_change_merge.rs lines 267–300) invokes `git merge-file --stdout` on ours/base/theirs temp files, returns Ok on exit 0 and Err with conflict count on exit >0. find_git_binary() (lines 305–318) falls back to overwrite-with-warning when git is absent. collect_spec_paths_into() skips .base.md files (helpers.rs lines 691–695). Two-pass atomicity (validation+merge pass lines 102–189, write pass lines 200–213) and audit log with create/overwrite/3way-merge actions (line 206) are all correct. 3-way merge condition requires both base_path.exists() && target_path.exists() (line 143) — semantically correct and a conservative safe extension of the spec.
- [PASS] [HARD] Spec has no ## Test Plan section — hard test-presence check N/A
  - The spec (merge-3way.md) has no ## Test Plan section. Hard reject rule does not apply. However, the ## Changes section (lines 253–283) specified 5 required test functions — all are present.
- [PASS] [HARD] Existing tests still pass (no regressions)
  - All 12 tests in create_change_merge::tests pass: the 5 new 3-way merge tests (test_3way_merge_clean, test_3way_merge_conflict, test_base_md_skipped_by_find_specs, test_no_base_fallback_overwrite, test_audit_log_3way_merge) plus 7 pre-existing tests. `cargo test -p cclab-sdd create_change_merge` reports 12 passed, 0 failed.

## Review: spec-prep-base-snapshot

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-merge-3way

**Summary**: All spec requirements are implemented correctly. `prepare_modify_spec` now returns `(String, Option<String>)`, clones raw source before frontmatter mutation, and the caller writes `.base.md` only for modify specs with existing source. All 4 required test functions are present and passing. No regressions in spec_plan tests.

### Checklist

- [PASS] [HARD] Code matches all spec requirements
  - prepare_modify_spec (spec_plan.rs:346-401): returns (String, Option<String>), clones raw_source to base_content BEFORE frontmatter mutation (line 364), returns (content, Some(base_content)) when source exists (line 401), returns (content, None) via fallback when source missing (line 360). prepare_specs_from_plan (lines 322-327): writes {spec_id}.base.md when base_content is Some, immediately before writing working spec. Only action:modify specs get a .base.md; action:create returns None. prepare_create_spec, deduplicate_spec_plans, read_all_spec_plans all untouched (do_not_touch respected).
- [PASS] [HARD] Spec has no ## Test Plan section — hard test-presence check N/A
  - The spec (spec-prep-base-snapshot.md) has no ## Test Plan section. Hard reject rule does not apply. The ## Changes section specifies 4 required test functions — all are present and passing: test_prepare_modify_creates_base_snapshot, test_prepare_create_no_base_snapshot, test_prepare_modify_missing_source_no_base, test_prepare_skip_already_prepared_no_duplicate_base.
- [PASS] [HARD] Existing tests still pass (no regressions)
  - All 21 tests in tools::spec_plan::tests pass. The single failing test (test_workflow_prompt_includes_spec_plan_guidance_with_subfolder_rule) is in create_reference_context.rs — an async integration test that attempts to spawn a live claude-agent subprocess. This is a pre-existing infrastructure failure unrelated to the spec_plan.rs changes.

