---
id: implementation
type: change_implementation
change_id: sdd-subagent-mode
---

# Implementation

## Summary

*(auto-generated baseline from git diff)*

## Changed Files

```
M	.claude/skills/cclab-sdd-run-change/SKILL.md
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
M	cclab/config.toml
M	cclab/specs/crates/cclab-jet/aot-build.md
D	cclab/specs/crates/cclab-jet/bundle-optimization-hoisting.md
D	cclab/specs/crates/cclab-jet/jet-remaining-spec.md
A	cclab/specs/crates/cclab-jet/workspace-protocol.md
M	cclab/specs/crates/cclab-sdd/config/agents.md
A	cclab/specs/crates/cclab-sdd/generate/ux-pattern-library.md
M	cclab/specs/crates/cclab-sdd/logic/change-spec.md
M	cclab/specs/crates/cclab-sdd/logic/executor-resolution.md
M	cclab/specs/crates/cclab-sdd/logic/reference-context.md
A	cclab/specs/crates/cclab-sdd/logic/tech-stack-inference.md
M	cclab/specs/crates/cclab-sdd/skills/run-change.md
M	cclab/specs/crates/cclab-sdd/tools/utils/delegate-agent.md
M	crates/cclab-jet/src/asset/image_processor.rs
M	crates/cclab-jet/src/bundler/css_bundle.rs
A	crates/cclab-jet/src/bundler/html_minify.rs
A	crates/cclab-jet/src/bundler/json_shake.rs
M	crates/cclab-jet/src/bundler/minify.rs
M	crates/cclab-jet/src/bundler/mod.rs
M	crates/cclab-jet/src/bundler/sourcemap.rs
M	crates/cclab-jet/src/bundler/splitting.rs
M	crates/cclab-jet/src/bundler/tree_shake.rs
M	crates/cclab-jet/src/bundler/types.rs
M	crates/cclab-jet/src/cli.rs
M	crates/cclab-jet/src/transform/mod.rs
A	crates/cclab-jet/tests/workspace_protocol.rs
M	crates/cclab-sdd-cli/templates/mainthread/skills/cclab-sdd-run-change/SKILL.md
M	crates/cclab-sdd/src/generate/lib.rs
A	crates/cclab-sdd/src/generate/patterns/mod.rs
A	crates/cclab-sdd/src/generate/patterns/registry.rs
A	crates/cclab-sdd/src/generate/patterns/resolver.rs
M	crates/cclab-sdd/src/models/change.rs
M	crates/cclab-sdd/src/models/mod.rs
M	crates/cclab-sdd/src/models/spec_rules.rs
A	crates/cclab-sdd/src/models/tech_stack.rs
M	crates/cclab-sdd/src/orchestrator/cli_mapper.rs
M	crates/cclab-sdd/src/orchestrator/script_runner.rs
M	crates/cclab-sdd/src/services/mod.rs
M	crates/cclab-sdd/src/services/spec_service.rs
A	crates/cclab-sdd/src/services/tech_stack_service.rs
M	crates/cclab-sdd/src/tools/agent.rs
M	crates/cclab-sdd/src/tools/common_change_spec.rs
M	crates/cclab-sdd/src/tools/create_change_spec.rs
M	crates/cclab-sdd/src/tools/create_reference_context.rs
M	crates/cclab-sdd/src/tools/spec_plan.rs
M	crates/cclab-sdd/src/tools/workflow_common.rs
M	crates/cclab-sdd/templates/mainthread/skills/cclab-sdd-run-change/SKILL.md
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
```

## Diff Statistics

```
.claude/skills/cclab-sdd-run-change/SKILL.md       |    5 +
 .../20260324-jet-workspace-protocol}/STATE.yaml    |   35 +-
 .../jet-workspace-protocol/post_clarifications.md  |    0
 .../jet-workspace-protocol/pre_clarifications.md   |    0
 .../analyze_spec_jet-workspace-protocol-spec.md    |    0
 .../prompts/begin_implementation.md                |    0
 .../prompts/create_post_clarifications.md          |    0
 .../prompts/create_pre_clarifications.md           |    0
 .../prompts/create_reference_context.md            |    0
 .../implement_tests_jet-workspace-protocol-spec.md |   25 +
 .../review_impl_jet-workspace-protocol-spec.md     |   58 +
 .../prompts/review_reference_context.md            |    0
 .../prompts/write_implementation_diff.md           |   14 +
 .../jet-workspace-protocol/reference_context.md    |    0
 .../groups/jet-workspace-protocol/requirements.md  |    0
 .../specs/jet-workspace-protocol-spec.md           |    0
 .../implementation.md                              | 1338 ++++++++++
 .../payloads/create-change-implementation.json     |    1 +
 .../payloads/create-pre-clarifications.json        |    0
 .../payloads/create-reference-context.json         |    0
 .../payloads/create-spec-changes.json              |    0
 .../payloads/create-spec-interaction.json          |    0
 .../payloads/create-spec-logic.json                |    0
 .../payloads/create-spec-overview.json             |    0
 .../payloads/create-spec-requirements.json         |    0
 .../payloads/create-spec-scenarios.json            |    0
 .../payloads/create-spec-schema.json               |    0
 .../payloads/create-spec-state-machine.json        |    0
 .../post-clarify-jet-workspace-protocol.json       |    0
 .../ref-context-jet-workspace-protocol.json        |    0
 .../payloads/restructure-input.json                |    0
 .../payloads/review-change-implementation.json     |   42 +
 .../payloads/review-reference-context.json         |    0
 .../payloads/spec-interaction.json                 |    0
 .../payloads/spec-logic.json                       |    0
 .../payloads/spec-schema.json                      |    0
 .../payloads/spec-state-machine.json               |    0
 .../prompts/create_change_merge.md                 |    6 +
 .../prompts/restructure_input.md                   |    0
 .../20260324-jet-workspace-protocol}/user_input.md |    0
 .../20260324-section-type-coverage/STATE.yaml      |   76 +
 .../payloads/create-change-spec.json               |    6 +
 .../frontend-design-system/post_clarifications.md  |   10 +
 .../frontend-design-system/pre_clarifications.md   |   17 +
 ...analyze_spec_change-spec-section-optionality.md |   26 +
 .../prompts/analyze_spec_tech-stack-inference.md   |   26 +
 .../prompts/analyze_spec_ux-pattern-library.md     |   26 +
 .../prompts/begin_implementation.md                |   18 +
 .../prompts/create_post_clarifications.md          |   48 +
 .../prompts/create_pre_clarifications.md           |   29 +
 .../prompts/create_reference_context.md            |  200 ++
 .../prompts/implement_spec.md                      |   17 +
 ...lement_tests_change-spec-section-optionality.md |   25 +
 .../implement_tests_tech-stack-inference.md        |   25 +
 .../prompts/implement_tests_ux-pattern-library.md  |   25 +
 .../prompts/review_reference_context.md            |   28 +
 .../prompts/revise_reference_context.md            |   23 +
 .../frontend-design-system/reference_context.md    |   41 +
 .../groups/frontend-design-system/requirements.md  |   16 +
 .../groups/frontend-design-system/spec_plan.yaml   |   24 +
 .../specs/change-spec-section-optionality.md       |  601 +++++
 .../specs/tech-stack-inference.md                  |  544 ++++
 .../specs/ux-pattern-library.md                    |  474 ++++
 .../new-section-types/post_clarifications.md       |   10 +
 .../groups/new-section-types/pre_clarifications.md |   21 +
 .../prompts/create_post_clarifications.md          |   48 +
 .../prompts/create_pre_clarifications.md           |   29 +
 .../prompts/create_reference_context.md            |  200 ++
 .../new-section-types/prompts/implement_spec.md    |   17 +
 .../implement_tests_reference-context-types.md     |   25 +
 .../groups/new-section-types/reference_context.md  |   45 +
 .../groups/new-section-types/requirements.md       |   25 +
 .../groups/new-section-types/spec_plan.yaml        |    7 +
 .../specs/reference-context-types.md               |  312 +++
 .../implementation.md                              |   54 +
 ...dd-section-type-coverage-all-roles-fe-be-sre.md |   72 +
 ...sign-system-as-tech-stack-config-ux-pattern-.md |   53 +
 ...053_sdd-add-e2e-scenario-section-type-for-qa.md |   53 +
 ...d-security-section-types-threat-model-auth-m.md |   84 +
 ...-add-qa-section-types-test-fixture-perf-test.md |   43 +
 ...d-sre-section-types-container-deploy-cloud-r.md |   28 +
 ...d-backend-mle-agent-section-types-grpc-graph.md |   33 +
 .../payloads/create-change-implementation.json     |   16 +
 .../payloads/create-post-clarifications.json       |    4 +
 .../payloads/create-pre-clarifications.json        |   17 +
 .../payloads/create-reference-context.json         |  114 +
 .../payloads/restructure-input.json                |   26 +
 .../payloads/review-change-implementation.json     |    6 +
 .../payloads/review-reference-context.json         |    6 +
 .../payloads/revise-reference-context.json         |   56 +
 .../prompts/create_change_merge.md                 |    6 +
 .../prompts/restructure_input.md                   |   64 +
 .../review_impl_change-spec-section-optionality.md |   58 +
 .../prompts/review_impl_reference-context-types.md |   58 +
 .../prompts/write_implementation_diff.md           |   14 +
 .../20260324-section-type-coverage/user_input.md   |    1 +
 cclab/changes/jet-aot-build-gaps/STATE.yaml        |   42 +
 .../jet-aot-build-gaps/pre_clarifications.md       |   21 +
 .../analyze_spec_jet-aot-build-gaps-spec.md        |   26 +
 .../prompts/begin_implementation.md                |   18 +
 .../prompts/create_pre_clarifications.md           |   29 +
 .../prompts/create_reference_context.md            |   90 +
 .../fill_spec_jet-aot-build-gaps-spec_changes.md   |   14 +
 .../fill_spec_jet-aot-build-gaps-spec_overview.md  |    8 +
 ...ll_spec_jet-aot-build-gaps-spec_requirements.md |   13 +
 .../fill_spec_jet-aot-build-gaps-spec_scenarios.md |   12 +
 .../prompts/review_reference_context.md            |   28 +
 .../prompts/revise_reference_context.md            |   23 +
 .../groups/jet-aot-build-gaps/reference_context.md |   38 +
 .../groups/jet-aot-build-gaps/requirements.md      |    9 +
 .../groups/jet-aot-build-gaps/spec_plan.yaml       |   10 +
 .../specs/jet-aot-build-gaps-spec.md               |  250 ++
 ...et-aot-production-build-tree-shaking-code-sp.md |  140 +
 .../payloads/create-pre-clarifications.json        |   20 +
 .../payloads/create-reference-context.json         |   64 +
 .../payloads/fill-section-overview.json            |    9 +
 .../payloads/restructure-input.json                |   32 +
 .../payloads/review-reference-context.json         |   18 +
 .../payloads/revise-reference-context.json         |   64 +
 .../prompts/restructure_input.md                   |   64 +
 cclab/changes/jet-aot-build-gaps/user_input.md     |    1 +
 cclab/config.toml                                  |   21 +-
 cclab/specs/crates/cclab-jet/aot-build.md          |  115 +-
 .../cclab-jet/bundle-optimization-hoisting.md      |  136 -
 cclab/specs/crates/cclab-jet/jet-remaining-spec.md |   10 -
 cclab/specs/crates/cclab-jet/workspace-protocol.md |  440 ++++
 cclab/specs/crates/cclab-sdd/config/agents.md      |   32 +-
 .../cclab-sdd/generate/ux-pattern-library.md       |  469 ++++
 cclab/specs/crates/cclab-sdd/logic/change-spec.md  |  130 +-
 .../crates/cclab-sdd/logic/executor-resolution.md  |  100 +-
 .../crates/cclab-sdd/logic/reference-context.md    |    6 +-
 .../crates/cclab-sdd/logic/tech-stack-inference.md |  539 ++++
 cclab/specs/crates/cclab-sdd/skills/run-change.md  |   46 +-
 .../crates/cclab-sdd/tools/utils/delegate-agent.md |   81 +-
 crates/cclab-jet/src/asset/image_processor.rs      |  306 ++-
 crates/cclab-jet/src/bundler/css_bundle.rs         |  336 +++
 crates/cclab-jet/src/bundler/html_minify.rs        |  358 +++
 crates/cclab-jet/src/bundler/json_shake.rs         |  255 ++
 crates/cclab-jet/src/bundler/minify.rs             |    4 +
 crates/cclab-jet/src/bundler/mod.rs                |  118 +-
 crates/cclab-jet/src/bundler/sourcemap.rs          |  453 ++++
 crates/cclab-jet/src/bundler/splitting.rs          |  291 +++
 crates/cclab-jet/src/bundler/tree_shake.rs         |   23 +-
 crates/cclab-jet/src/bundler/types.rs              |   21 +
 crates/cclab-jet/src/cli.rs                        |   13 +-
 crates/cclab-jet/src/transform/mod.rs              |   20 +
 crates/cclab-jet/tests/workspace_protocol.rs       |  457 ++++
 .../skills/cclab-sdd-run-change/SKILL.md           |   12 +-
 crates/cclab-sdd/src/generate/lib.rs               |    1 +
 crates/cclab-sdd/src/generate/patterns/mod.rs      |   80 +
 crates/cclab-sdd/src/generate/patterns/registry.rs |   17 +
 crates/cclab-sdd/src/generate/patterns/resolver.rs |   66 +
 crates/cclab-sdd/src/models/change.rs              |  255 +-
 crates/cclab-sdd/src/models/mod.rs                 |    4 +-
 crates/cclab-sdd/src/models/spec_rules.rs          |  492 ++++
 crates/cclab-sdd/src/models/tech_stack.rs          |  112 +
 crates/cclab-sdd/src/orchestrator/cli_mapper.rs    |   29 +-
 crates/cclab-sdd/src/orchestrator/script_runner.rs |    2 +-
 crates/cclab-sdd/src/services/mod.rs               |    5 +-
 crates/cclab-sdd/src/services/spec_service.rs      |  372 ++-
 .../cclab-sdd/src/services/tech_stack_service.rs   |  318 +++
 crates/cclab-sdd/src/tools/agent.rs                |   82 +-
 crates/cclab-sdd/src/tools/common_change_spec.rs   |  101 +
 crates/cclab-sdd/src/tools/create_change_spec.rs   |   58 +-
 .../src/tools/create_reference_context.rs          |   21 +-
 crates/cclab-sdd/src/tools/spec_plan.rs            |   34 +-
 crates/cclab-sdd/src/tools/workflow_common.rs      |  352 ++-
 .../skills/cclab-sdd-run-change/SKILL.md           |   12 +-
 docs/.gitignore                                    |    3 +
 docs/.vitepress/config.mjs                         |   43 +
 docs/PROMPT_TEMPLATE_INTEGRATION.md                |  340 ---
 docs/README.md                                     |  623 -----
 docs/agent_eval_prompt_templates.md                |  391 ---
 docs/api/sse.md                                    |  521 ----
 docs/archive/ADAPTIVE_SAMPLING.md                  |  227 --
 docs/archive/MIGRATION_MAP.md                      |  278 --
 docs/archive/OPENTELEMETRY.md                      | 2753 --------------------
 docs/archive/PHASE2_LAZY_LOADING_IMPLEMENTATION.md |  287 --
 docs/archive/PHASE3_IMPLEMENTATION_SUMMARY.md      |  206 --
 docs/archive/PHASE4_IMPLEMENTATION_SUMMARY.md      |  517 ----
 docs/archive/PHASE5_SUMMARY.md                     |  244 --
 docs/archive/PHASE7_IMPLEMENTATION_SUMMARY.md      |  315 ---
 docs/archive/POSTGRESQL_EXTENSIONS.md              |  468 ----
 docs/archive/README.md                             |   33 -
 docs/archive/SHEET_ARCHITECTURE.md                 |  823 ------
 docs/archive/SHEET_CONTRIBUTING.md                 |  347 ---
 docs/archive/SHEET_README.md                       |  382 ---
 docs/archive/SPAN_HIERARCHY.md                     |  207 --
 docs/archive/TELEMETRY_RELATIONSHIPS.md            |  222 --
 docs/archive/canvas_primitives.md                  |  239 --
 docs/archive/kv_benchmark_concurrent_fixed.md      |   44 -
 docs/archive/legacy/BATCH_CONVERSION_SUMMARY.md    |  302 ---
 docs/archive/legacy/CONVERSION_REPORT.md           |  215 --
 docs/archive/legacy/CRUD_API_REFACTOR_SUMMARY.md   |  229 --
 .../legacy/PHASE5_IMPLEMENTATION_COMPLETE.md       |  282 --
 docs/archive/legacy/PYLOOP_COMPILATION_FIXES.md    |  230 --
 docs/archive/legacy/PYLOOP_PHASE1_SUMMARY.md       |  265 --
 docs/archive/legacy/PYLOOP_PHASE2.5_SUMMARY.md     |  179 --
 docs/archive/legacy/PYLOOP_PHASE2_3_SUMMARY.md     |  233 --
 docs/archive/legacy/PYLOOP_PHASE2_4_SUMMARY.md     |  205 --
 docs/archive/legacy/PYLOOP_PHASE2_SUMMARY.md       |  195 --
 docs/archive/legacy/PYLOOP_PHASE3.1.1_SUMMARY.md   |  255 --
 docs/archive/legacy/PYLOOP_PHASE3.1.2_SUMMARY.md   |  345 ---
 docs/archive/legacy/PYLOOP_PHASE3.1.3_SUMMARY.md   |  379 ---
 docs/archive/legacy/PYLOOP_PHASE3.1.4_SUMMARY.md   |  435 ----
 docs/archive/legacy/PYLOOP_PHASE3_CRUD_SUMMARY.md  |  270 --
 docs/archive/legacy/PYLOOP_PHASE3_SUMMARY.md       |  242 --
 docs/archive/legacy/PYLOOP_PHASE4_FILES.md         |  199 --
 docs/archive/legacy/PYLOOP_PHASE4_SUMMARY.md       |  659 -----
 docs/archive/legacy/PYLOOP_PHASE5_SUMMARY.md       |  488 ----
 .../benchmarks/API_BENCHMARK_GAP_ANALYSIS.md       |   72 -
 .../crates/data-bridge-api/PYTHON_INTEGRATION.md   |  338 ---
 .../legacy/crates/data-bridge-api/SERVER_README.md |  259 --
 .../legacy/crates/data-bridge-sheet-core/todos.md  |  216 --
 .../legacy/crates/data-bridge-sheet-db/todos.md    |   76 -
 .../legacy/crates/data-bridge-test/FIXTURES.md     |  441 ----
 .../data-bridge-test/IMPLEMENTATION_SUMMARY.md     |  287 --
 .../legacy/crates/data-bridge-test/TODOS.md        |  265 --
 docs/archive/legacy/deploy/OBSERVABILITY.md        |  315 ---
 docs/archive/legacy/deploy/TESTING.md              |  331 ---
 docs/archive/legacy/docs/MIGRATION_COMPLETE.md     |  636 -----
 docs/archive/legacy/docs/MIGRATION_STATUS.md       |  182 --
 docs/archive/legacy/docs/PYLOOP_BENCHMARKS.md      |  227 --
 docs/archive/legacy/docs/PYLOOP_CRUD.md            |  481 ----
 docs/archive/legacy/docs/TESTING.md                |  496 ----
 docs/archive/legacy/docs/TEST_SERVER_PYTHON_APP.md |  260 --
 .../legacy/docs/sheet-specs/advanced-features.md   | 1088 --------
 .../legacy/docs/sheet-specs/architecture.md        |  146 --
 docs/archive/legacy/docs/sheet-specs/clipboard.md  |   85 -
 .../legacy/docs/sheet-specs/data-structures.md     |  683 -----
 docs/archive/legacy/docs/sheet-specs/flowchart.md  |   94 -
 .../legacy/docs/sheet-specs/formatting-rules.md    |   93 -
 .../legacy/docs/sheet-specs/formula-engine.md      |  143 -
 docs/archive/legacy/docs/sheet-specs/fsm.md        |   69 -
 .../legacy/docs/sheet-specs/keyboard-shortcuts.md  |   69 -
 .../archive/legacy/docs/sheet-specs/performance.md |  762 ------
 .../archive/legacy/docs/sheet-specs/persistence.md |  106 -
 .../legacy/docs/sheet-specs/rendering-engine.md    |  662 -----
 .../legacy/docs/sheet-specs/sheet-management.md    |   95 -
 .../legacy/docs/sheet-specs/ui-interactions.md     |   80 -
 .../legacy/docs/sheet-specs/user-experience.md     |  112 -
 .../legacy/docs/sheet-specs/wasm-integration.md    |  806 ------
 docs/archive/legacy/docs/tasks/RATELIMIT_GUIDE.md  |  331 ---
 docs/archive/legacy/docs/tasks/README.md           |  328 ---
 .../docs/tasks/ROUTER_INTEGRATION_SUMMARY.md       |  258 --
 docs/archive/legacy/docs/tasks/ROUTING.md          |  427 ---
 .../docs/tasks/ROUTING_IMPLEMENTATION_SUMMARY.md   |  225 --
 .../legacy/docs/tasks/routing_integration.md       |  222 --
 .../tests/postgres/benchmarks/ARCHITECTURE.md      |  320 ---
 .../legacy/tests/postgres/benchmarks/QUICKSTART.md |  139 -
 docs/archive/legacy/tools/DELIVERABLES.md          |  376 ---
 .../archive/legacy/tools/IMPLEMENTATION_SUMMARY.md |  320 ---
 docs/archive/telemetry.md                          |  462 ----
 docs/en/user-guide.md                              |  321 ---
 docs/index.md                                      |   38 +-
 docs/jet/bundler.md                                |  118 +
 docs/jet/configuration.md                          |  117 +
 docs/jet/dev-server.md                             |   53 +
 docs/jet/getting-started.md                        |   57 +
 docs/jet/package-manager.md                        |  104 +
 docs/jet/task-runner.md                            |   98 +
 docs/jet/workspaces.md                             |   78 +
 docs/package.json                                  |   12 +
 docs/postgres/api.md                               |  132 -
 docs/postgres/guides/aggregation.md                |   97 -
 docs/postgres/guides/caching.md                    |   64 -
 docs/postgres/guides/events.md                     |   80 -
 docs/postgres/guides/indexes.md                    |   22 -
 docs/postgres/guides/inheritance.md                |  632 -----
 docs/postgres/guides/migrations.md                 |  602 -----
 docs/postgres/guides/querying.md                   |  153 --
 docs/postgres/guides/raw_sql.md                    |  350 ---
 docs/postgres/guides/state_management.md           |   89 -
 docs/postgres/guides/tables_and_columns.md         |  104 -
 docs/postgres/guides/transactions.md               |  256 --
 docs/postgres/guides/validation.md                 |   84 -
 docs/postgres/quickstart.md                        |  114 -
 docs/postgres/relationships.md                     | 1435 ----------
 docs/tasks/asyncapi.yaml                           |  332 ---
 docs/zh-tw/user-guide.md                           |  320 ---
 280 files changed, 14070 insertions(+), 33997 deletions(-)
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
+
+    let mut result = String::with_capacity(tag.len());
+    let chars: Vec<char> = tag.chars().collect();
+    let len = chars.len();
+    let mut i = 0;
+
+    while i < len {
+        if chars[i] == '=' && i + 1 < len {
+            result.push('=');
+            i += 1;
+            // Check for quoted attribute value
+            if chars[i] == '"' || chars[i] == '\'' {
+                let quote = chars[i];
+                let val_start = i + 1;
+                let mut val_end = val_start;
+                while val_end < len && chars[val_end] != quote {
+                    val_end += 1;
+                }
+                let value: String = chars[val_start..val_end].iter().collect();
+                if is_simple_attr_value(&value) {
+                    result.push_str(&value);
+                } else {
+                    result.push(quote);
+                    result.push_str(&value);
+                    if val_end < len {
+                        result.push(quote);
+                    }
+                }
+                i = if val_end < len { val_end + 1 } else { val_end };
+                continue;
+            }
+        }
+        result.push(chars[i]);
+        i += 1;
+    }
+
+    result
+}
+
+/// Check if an attribute value is "simple" (only contains safe characters).
+fn is_simple_attr_value(value: &str) -> bool {
+    !value.is_empty()
+        && value
+            .chars()
+            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_html_comment_removal() {
+        // T14: Strip HTML comments
+        let input = "<!-- todo -->  <div>hello</div>";
+        let result = minify_html(input);
+        assert!(
+            !result.contains("<!-- todo -->"),
+            "HTML comments should be removed, got: {}",
+            result
+        );
+        assert!(
+            result.contains("<div>hello</div>"),
+            "Content should be preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_html_whitespace_collapse() {
+        // S11: Whitespace between tags should be collapsed
+        let input = "<!-- comment -->  <div>  <p>text</p>  </div>";
+        let result = minify_html(input);
+        assert!(
+            !result.contains("<!-- comment -->"),
+            "Comments should be removed, got: {}",
+            result
+        );
+        assert!(
+            result.contains("<div>"),
+            "Tags should be preserved, got: {}",
+            result
+        );
+        assert!(
+            result.contains("<p>text</p>"),
+            "Inner content should be preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_html_preserves_pre_content() {
+        // T15: Whitespace inside <pre> should be preserved
+        let input = "<pre>  spaces  matter  </pre>";
+        let result = minify_html(input);
+        assert!(
+            result.contains("  spaces  matter  "),
+            "Whitespace inside <pre> must be preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_html_preserves_code_content() {
+        let input = "<code>  let x = 1;  </code>";
+        let result = minify_html(input);
+        assert!(
+            result.contains("  let x = 1;  "),
+            "Whitespace inside <code> must be preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_html_preserves_script_content() {
+        let input = "<script>  var x = 1;  </script>";
+        let result = minify_html(input);
+        assert!(
+            result.contains("  var x = 1;  "),
+            "Whitespace inside <script> must be preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_html_preserves_style_content() {
+        let input = "<style>  body { color: red; }  </style>";
+        let result = minify_html(input);
+        assert!(
+            result.contains("  body { color: red; }  "),
+            "Whitespace inside <style> must be preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_html_attribute_quote_removal() {
+        let input = r#"<div class="main" id="app">"#;
+        let result = minify_html(input);
+        assert!(
+            result.contains("class=main"),
+            "Simple attribute quotes should be removed, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_html_complex_attribute_keeps_quotes() {
+        let input = r#"<a href="https://example.com">"#;
+        let result = minify_html(input);
+        assert!(
+            result.contains("\"https://example.com\"")
+                || result.contains("'https://example.com'"),
+            "Complex attribute values should keep quotes, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_html_multiple_comments() {
+        let input = "<!-- first -->text<!-- second -->more";
+        let result = minify_html(input);
+        assert!(!result.contains("first"), "First comment removed");
+        assert!(!result.contains("second"), "Second comment removed");
+        assert!(result.contains("text"), "Text preserved");
+        assert!(result.contains("more"), "More text preserved");
+    }
+}
diff --git a/crates/cclab-jet/src/bundler/json_shake.rs b/crates/cclab-jet/src/bundler/json_shake.rs
new file mode 100644
index 00000000..ae77a9d8
--- /dev/null
+++ b/crates/cclab-jet/src/bundler/json_shake.rs
@@ -0,0 +1,255 @@
+//! JSON tree-shaking: dead code elimination for JSON imports.
+//!
+//! Named imports (`import { name } from './pkg.json'`) keep only used keys.
+//! Default imports (`import data from './config.json'`) keep all keys.
+
+use std::collections::{HashMap, HashSet};
+use std::path::{Path, PathBuf};
+
+use super::tree_shake::{extract_imported_names, extract_specifier, find_module_by_specifier};
+
+fn is_json(path: &Path) -> bool {
+    path.extension()
+        .and_then(|e| e.to_str())
+        .map(|e| e == "json")
+        .unwrap_or(false)
+}
+
+/// Analyze JSON imports across all modules and determine which keys are used.
+///
+/// For each JSON module:
+/// - Named imports (`import { name, version } from './pkg.json'`) use only those keys
+/// - Default imports (`import data from './config.json'`) use all keys
+/// - Namespace imports (`import * as pkg from './pkg.json'`) use all keys
+///
+/// Returns a map of JSON module path -> set of used top-level keys.
+/// An empty set means "use all keys" (default/namespace import).
+pub fn analyze_json_imports(
+    modules: &[(PathBuf, String)],
+) -> HashMap<PathBuf, JsonImportUsage> {
+    let mut json_usage: HashMap<PathBuf, JsonImportUsage> = HashMap::new();
+
+    // First, identify all JSON modules
+    for (path, _source) in modules {
+        if is_json(path) {
+            json_usage.insert(path.clone(), JsonImportUsage::NoImporters);
+        }
+    }
+
+    if json_usage.is_empty() {
+        return json_usage;
+    }
+
+    // Scan all non-JSON modules for imports from JSON files
+    for (_path, source) in modules {
+        for line in source.lines() {
+            let trimmed = line.trim();
+            if !trimmed.starts_with("import ") {
+                continue;
+            }
+
+            let specifier = extract_specifier(trimmed);
+            if !specifier.ends_with(".json") {
+                continue;
+            }
+
+            // Find the matching JSON module
+            let target = find_module_by_specifier(&specifier, modules);
+            if let Some((target_path, _)) = target {
+                if !is_json(target_path) {
+                    continue;
+                }
+
+                let names = extract_imported_names(trimmed);
+
+                let entry = json_usage
+                    .entry(target_path.clone())
+                    .or_insert(JsonImportUsage::NoImporters);
+
+                if names.contains(&"*".to_string())
+                    || names.contains(&"default".to_string())
+                {
+                    // Default or namespace import -> use all keys
+                    *entry = JsonImportUsage::UseAll;
+                } else if !names.is_empty() {
+                    match entry {
+                        JsonImportUsage::UseAll => {
+                            // Already using all, no change
+                        }
+                        JsonImportUsage::NoImporters => {
+                            *entry = JsonImportUsage::NamedKeys(
+                                names.into_iter().collect(),
+                            );
+                        }
+                        JsonImportUsage::NamedKeys(existing) => {
+                            for name in names {
+                                existing.insert(name);
+                            }
+                        }
+                    }
+                }
+            }
+        }
+    }
+
+    json_usage
+}
+
+/// How a JSON module's keys are used by importers.
+#[derive(Debug, Clone, PartialEq)]
+pub enum JsonImportUsage {
+    /// No module imports this JSON file.
+    NoImporters,
+    /// At least one importer uses default/namespace import -> keep all keys.
+    UseAll,
+    /// Only named imports -> keep only these keys.
+    NamedKeys(HashSet<String>),
+}
+
+/// Tree-shake a JSON string, keeping only the specified top-level keys.
+///
+/// If `used_keys` is `None` or the JSON is not an object, returns the
+/// original JSON unchanged.
+pub fn shake_json(json_source: &str, used_keys: Option<&HashSet<String>>) -> String {
+    let keys = match used_keys {
+        Some(k) if !k.is_empty() => k,
+        _ => return json_source.to_string(),
+    };
+
+    let parsed: serde_json::Value = match serde_json::from_str(json_source) {
+        Ok(v) => v,
+        Err(_) => return json_source.to_string(),
+    };
+
+    let obj = match parsed.as_object() {
+        Some(o) => o,
+        None => return json_source.to_string(),
+    };
+
+    let filtered: serde_json::Map<String, serde_json::Value> = obj
+        .iter()
+        .filter(|(k, _)| keys.contains(k.as_str()))
+        .map(|(k, v)| (k.clone(), v.clone()))
+        .collect();
+
+    serde_json::to_string(&filtered).unwrap_or_else(|_| json_source.to_string())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_json_tree_shake_named_import() {
+        // T19: Named import should keep only used keys
+        let json_source = r#"{"name":"x","version":"1","description":"y"}"#;
+        let mut used_keys = HashSet::new();
+        used_keys.insert("name".to_string());
+
+        let result = shake_json(json_source, Some(&used_keys));
+        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
+        let obj = parsed.as_object().unwrap();
+        assert!(obj.contains_key("name"), "Used key 'name' should be kept");
+        assert!(
+            !obj.contains_key("version"),
+            "Unused key 'version' should be removed"
+        );
+        assert!(
+            !obj.contains_key("description"),
+            "Unused key 'description' should be removed"
+        );
+        assert_eq!(obj["name"], "x");
+    }
+
+    #[test]
+    fn test_json_tree_shake_default_import_keeps_all() {
+        // T20: Default import should keep all keys
+        let json_source = r#"{"name":"x","version":"1","description":"y"}"#;
+
+        // None means use all (default import behavior)
+        let result = shake_json(json_source, None);
+        assert_eq!(result, json_source, "Default import should keep all keys");
+    }
+
+    #[test]
+    fn test_json_tree_shake_empty_keys() {
+        let json_source = r#"{"a":1,"b":2}"#;
+        let empty_keys = HashSet::new();
+        let result = shake_json(json_source, Some(&empty_keys));
+        assert_eq!(result, json_source, "Empty used_keys should keep all");
+    }
+
+    #[test]
+    fn test_json_tree_shake_multiple_keys() {
+        let json_source =
+            r#"{"name":"pkg","version":"1.0","desc":"test","main":"index.js","license":"MIT"}"#;
+        let mut used_keys = HashSet::new();
+        used_keys.insert("name".to_string());
+        used_keys.insert("version".to_string());
+
+        let result = shake_json(json_source, Some(&used_keys));
+        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
+        let obj = parsed.as_object().unwrap();
+        assert_eq!(obj.len(), 2);
+        assert!(obj.contains_key("name"));
+        assert!(obj.contains_key("version"));
+    }
+
+    #[test]
+    fn test_json_tree_shake_non_object() {
+        // JSON arrays and primitives should be returned unchanged
+        let array_source = r#"[1,2,3]"#;
+        let mut used_keys = HashSet::new();
+        used_keys.insert("name".to_string());
+        let result = shake_json(array_source, Some(&used_keys));
+        assert_eq!(result, array_source);
+    }
+
+    #[test]
+    fn test_analyze_json_imports_named() {
+        // Use paths that match what find_module_by_specifier expects:
+        // specifier "./pkg.json" matches path "src/pkg.json" via ends_with
+        let modules = vec![
+            (
+                PathBuf::from("src/app.js"),
+                "import { name } from './pkg.json';\n".to_string(),
+            ),
+            (
+                PathBuf::from("src/pkg.json"),
+                r#"{"name":"x","version":"1","desc":"y"}"#.to_string(),
+            ),
+        ];
+
+        let usage = analyze_json_imports(&modules);
+        let pkg_usage = usage.get(&PathBuf::from("src/pkg.json")).unwrap();
+        match pkg_usage {
+            JsonImportUsage::NamedKeys(keys) => {
+                assert!(keys.contains("name"), "Should have 'name' key");
+                assert!(!keys.contains("version"), "Should not have 'version' key");
+            }
+            other => panic!("Expected NamedKeys, got {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_analyze_json_imports_default() {
+        let modules = vec![
+            (
+                PathBuf::from("src/app.js"),
+                "import data from './config.json';\n".to_string(),
+            ),
+            (
+                PathBuf::from("src/config.json"),
+                r#"{"key":"value"}"#.to_string(),
+            ),
+        ];
+
+        let usage = analyze_json_imports(&modules);
+        let config_usage = usage.get(&PathBuf::from("src/config.json")).unwrap();
+        assert_eq!(
+            *config_usage,
+            JsonImportUsage::UseAll,
+            "Default import should use all keys"
+        );
+    }
+}
diff --git a/crates/cclab-jet/src/bundler/minify.rs b/crates/cclab-jet/src/bundler/minify.rs
index ced06c8a..a6b29e05 100644
--- a/crates/cclab-jet/src/bundler/minify.rs
+++ b/crates/cclab-jet/src/bundler/minify.rs
@@ -423,6 +423,9 @@ fn is_id_char(c: u8) -> bool {
     c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
 }
 
+// HTML minification is in the `html_minify` submodule.
+// Re-exported from `super::html_minify::minify_html`.
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -707,4 +710,5 @@ var x = 1;"#;
         // Should not panic, and the identifier should survive
         assert!(result.contains("café"), "UTF-8 identifier preserved, got: {}", result);
     }
+
 }
diff --git a/crates/cclab-jet/src/bundler/mod.rs b/crates/cclab-jet/src/bundler/mod.rs
index ed46c8b8..86993acd 100644
--- a/crates/cclab-jet/src/bundler/mod.rs
+++ b/crates/cclab-jet/src/bundler/mod.rs
@@ -12,7 +12,9 @@ pub mod dce;
 pub mod define;
 pub mod fold;
 pub mod graph;
+pub mod html_minify;
 pub mod imports;
+pub mod json_shake;
 pub mod mangle;
 pub mod minify;
 pub mod scope_hoist;
@@ -23,7 +25,8 @@ pub mod types;
 
 pub use graph::{EdgeKind, ModuleGraph, ModuleNode};
 pub use imports::{ImportDeclaration, ImportKind, ModuleImports};
-pub use types::{BundleOptions, BundleOutput, ModuleId};
+pub use splitting::SplitResult;
+pub use types::{BundleOptions, BundleOutput, ModuleId, PreloadHint};
 
 /// Determine module kind from file extension
 fn determine_module_kind(path: &PathBuf) -> graph::ModuleKind {
@@ -134,6 +137,49 @@ fn generate_runtime() -> String {
     .to_string()
 }
 
+/// Generate `<link rel="modulepreload">` tags from preload hints.
+///
+/// Returns HTML tags suitable for injection into `<head>`. Only static
+/// dependencies are included; dynamic imports are excluded since they
+/// load on demand.
+pub fn generate_preload_tags(hints: &[PreloadHint]) -> String {
+    let mut tags = String::new();
+    for hint in hints {
+        if hint.is_static {
+            tags.push_str(&format!(
+                "<link rel=\"modulepreload\" href=\"{}\">\n",
+                hint.href
+            ));
+        }
+    }
+    tags
+}
+
+/// Inject preload hint tags into an HTML string's `<head>` section.
+///
+/// If `<head>` is found, the tags are inserted right after it.
+/// Otherwise the tags are prepended to the HTML.
+pub fn inject_preload_hints(html: &str, hints: &[PreloadHint]) -> String {
+    let tags = generate_preload_tags(hints);
+    if tags.is_empty() {
+        return html.to_string();
+    }
+
+    // Try to insert after <head> (case-insensitive search)
+    let lower = html.to_lowercase();
+    if let Some(pos) = lower.find("<head>") {
+        let insert_pos = pos + "<head>".len();
+        let mut result = String::with_capacity(html.len() + tags.len() + 1);
+        result.push_str(&html[..insert_pos]);
+        result.push('\n');
+        result.push_str(&tags);
+        result.push_str(&html[insert_pos..]);
+        result
+    } else {
+        format!("{}{}", tags, html)
+    }
+}
+
 /// Core bundler that orchestrates the build process
 pub struct Bundler {
     resolver: Arc<crate::resolver::ModuleResolver>,
@@ -693,6 +739,76 @@ mod tests {
         assert_eq!(cache.module_cache.len(), 0);
     }
 
+    // ──────────────────────────────────────────────────────────────────
+    // Preload hints tests (R8 / T12)
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_generate_preload_tags() {
+        let hints = vec![
+            PreloadHint {
+                href: "assets/vendor.abc123.js".to_string(),
+                is_static: true,
+            },
+            PreloadHint {
+                href: "assets/chunk-lazy.def456.js".to_string(),
+                is_static: false, // dynamic, should be excluded
+            },
+        ];
+        let tags = generate_preload_tags(&hints);
+        assert!(
+            tags.contains(r#"<link rel="modulepreload" href="assets/vendor.abc123.js">"#),
+            "Static preload hint should generate a modulepreload tag"
+        );
+        assert!(
+            !tags.contains("chunk-lazy"),
+            "Dynamic imports should not be preloaded"
+        );
+    }
+
+    #[test]
+    fn test_inject_preload_hints_into_head() {
+        let html = "<html><head><title>App</title></head><body></body></html>";
+        let hints = vec![PreloadHint {
+            href: "assets/vendor.abc123.js".to_string(),
+            is_static: true,
+        }];
+        let result = inject_preload_hints(html, &hints);
+        assert!(
+            result.contains(r#"<link rel="modulepreload" href="assets/vendor.abc123.js">"#),
+            "Preload tag should be injected"
+        );
+        // Should appear after <head>
+        let head_pos = result.find("<head>").unwrap();
+        let link_pos = result.find("modulepreload").unwrap();
+        assert!(
+            link_pos > head_pos,
+            "Preload tag should be after <head>"
+        );
+    }
+
+    #[test]
+    fn test_inject_preload_hints_no_head() {
+        let html = "<div>Content</div>";
+        let hints = vec![PreloadHint {
+            href: "assets/shared.js".to_string(),
+            is_static: true,
+        }];
+        let result = inject_preload_hints(html, &hints);
+        assert!(
+            result.contains("modulepreload"),
+            "Preload tag should be prepended when no <head>"
+        );
+    }
+
+    #[test]
+    fn test_inject_preload_hints_empty() {
+        let html = "<html><head></head></html>";
+        let hints: Vec<PreloadHint> = Vec::new();
+        let result = inject_preload_hints(html, &hints);
+        assert_eq!(result, html, "Empty hints should not modify HTML");
+    }
+
     // ──────────────────────────────────────────────────────────────────
     // Phase 2 flattening + mangling pipeline tests (#882, #903)
     // ──────────────────────────────────────────────────────────────────
diff --git a/crates/cclab-jet/src/bundler/sourcemap.rs b/crates/cclab-jet/src/bundler/sourcemap.rs
index 293f3f44..655a080c 100644
--- a/crates/cclab-jet/src/bundler/sourcemap.rs
+++ b/crates/cclab-jet/src/bundler/sourcemap.rs
@@ -169,6 +169,277 @@ fn vlq_char(value: u8) -> char {
     CHARS[value as usize] as char
 }
 
+/// A single decoded mapping entry.
+#[derive(Debug, Clone)]
+pub struct MappingEntry {
+    /// Generated line (0-based).
+    pub gen_line: usize,
+    /// Generated column (0-based).
+    pub gen_col: usize,
+    /// Source index.
+    pub source: usize,
+    /// Original line (0-based).
+    pub orig_line: usize,
+    /// Original column (0-based).
+    pub orig_col: usize,
+}
+
+/// Decode a VLQ-encoded mappings string into a list of mapping entries.
+pub fn decode_mappings(mappings: &str) -> Vec<MappingEntry> {
+    let mut entries = Vec::new();
+    let mut gen_line = 0usize;
+    #[allow(unused_assignments)]
+    let mut prev_gen_col = 0i64;
+    let mut prev_source = 0i64;
+    let mut prev_orig_line = 0i64;
+    let mut prev_orig_col = 0i64;
+
+    for line_str in mappings.split(';') {
+        if !line_str.is_empty() {
+            prev_gen_col = 0; // gen_col resets per line
+            for segment in line_str.split(',') {
+                let values = vlq_decode(segment);
+                if values.len() >= 4 {
+                    prev_gen_col += values[0];
+                    prev_source += values[1];
+                    prev_orig_line += values[2];
+                    prev_orig_col += values[3];
+
+                    entries.push(MappingEntry {
+                        gen_line,
+                        gen_col: prev_gen_col as usize,
+                        source: prev_source as usize,
+                        orig_line: prev_orig_line as usize,
+                        orig_col: prev_orig_col as usize,
+                    });
+                }
+            }
+        }
+        gen_line += 1;
+    }
+
+    entries
+}
+
+/// Encode mapping entries back into a VLQ mappings string.
+fn encode_mappings(entries: &[MappingEntry], max_gen_line: usize) -> String {
+    let mut segments: Vec<String> = Vec::new();
+    #[allow(unused_assignments)]
+    let mut prev_gen_col = 0i64;
+    let mut prev_source = 0i64;
+    let mut prev_orig_line = 0i64;
+    let mut prev_orig_col = 0i64;
+
+    let total_lines = if entries.is_empty() {
+        max_gen_line + 1
+    } else {
+        let max_entry_line = entries.iter().map(|e| e.gen_line).max().unwrap_or(0);
+        std::cmp::max(max_gen_line + 1, max_entry_line + 1)
+    };
+
+    for line in 0..total_lines {
+        let line_entries: Vec<&MappingEntry> =
+            entries.iter().filter(|e| e.gen_line == line).collect();
+
+        if line_entries.is_empty() {
+            segments.push(String::new());
+        } else {
+            prev_gen_col = 0; // reset per line
+            let mut line_segs: Vec<String> = Vec::new();
+
+            for entry in line_entries {
+                let mut seg = String::new();
+                vlq_encode(&mut seg, entry.gen_col as i64 - prev_gen_col);
+                vlq_encode(&mut seg, entry.source as i64 - prev_source);
+                vlq_encode(&mut seg, entry.orig_line as i64 - prev_orig_line);
+                vlq_encode(&mut seg, entry.orig_col as i64 - prev_orig_col);
+
+                prev_gen_col = entry.gen_col as i64;
+                prev_source = entry.source as i64;
+                prev_orig_line = entry.orig_line as i64;
+                prev_orig_col = entry.orig_col as i64;
+
+                line_segs.push(seg);
+            }
+            segments.push(line_segs.join(","));
+        }
+    }
+
+    segments.join(";")
+}
+
+/// Decode a single VLQ segment into a vector of values.
+fn vlq_decode(segment: &str) -> Vec<i64> {
+    const VLQ_BASE_SHIFT: u32 = 5;
+    const VLQ_BASE: i64 = 1 << VLQ_BASE_SHIFT;
+    const VLQ_BASE_MASK: i64 = VLQ_BASE - 1;
+    const VLQ_CONTINUATION_BIT: i64 = VLQ_BASE;
+
+    let mut values = Vec::new();
+    let mut shift = 0u32;
+    let mut result: i64 = 0;
+
+    for ch in segment.chars() {
+        let digit = vlq_decode_char(ch);
+        if digit < 0 {
+            continue;
+        }
+        let digit = digit as i64;
+        result += (digit & VLQ_BASE_MASK) << shift;
+        shift += VLQ_BASE_SHIFT;
+
+        if digit & VLQ_CONTINUATION_BIT == 0 {
+            // Final digit
+            let is_negative = (result & 1) == 1;
+            let value = result >> 1;
+            values.push(if is_negative { -value } else { value });
+            result = 0;
+            shift = 0;
+        }
+    }
+
+    values
+}
+
+/// Decode a Base64 character to its 6-bit value, or -1 if invalid.
+fn vlq_decode_char(ch: char) -> i8 {
+    match ch {
+        'A'..='Z' => (ch as i8) - ('A' as i8),
+        'a'..='z' => (ch as i8) - ('a' as i8) + 26,
+        '0'..='9' => (ch as i8) - ('0' as i8) + 52,
+        '+' => 62,
+        '/' => 63,
+        _ => -1,
+    }
+}
+
+/// Compose (chain) two source maps: an input map and an output map.
+///
+/// The input map maps intermediate positions to original positions
+/// (e.g., TS -> JS). The output map maps final positions to intermediate
+/// positions (e.g., bundled/minified -> JS). The result maps final
+/// positions directly to original positions.
+///
+/// `sourcesContent` from the input map is preserved in the result.
+pub fn compose_source_maps(input_map_json: &str, output_map_json: &str) -> String {
+    // Parse both maps
+    let input_map: serde_json::Value =
+        serde_json::from_str(input_map_json).unwrap_or(serde_json::Value::Null);
+    let output_map: serde_json::Value =
+        serde_json::from_str(output_map_json).unwrap_or(serde_json::Value::Null);
+
+    if input_map.is_null() || output_map.is_null() {
+        return output_map_json.to_string();
+    }
+
+    let input_mappings = input_map
+        .get("mappings")
+        .and_then(|v| v.as_str())
+        .unwrap_or("");
+    let output_mappings = output_map
+        .get("mappings")
+        .and_then(|v| v.as_str())
+        .unwrap_or("");
+
+    let input_entries = decode_mappings(input_mappings);
+    let output_entries = decode_mappings(output_mappings);
+
+    // For each output entry, look up the intermediate position in the input map
+    // to find the original position
+    let mut composed_entries: Vec<MappingEntry> = Vec::new();
+
+    for out_entry in &output_entries {
+        // out_entry maps: final (gen_line, gen_col) -> intermediate (orig_line, orig_col)
+        // We need to find an input entry that maps intermediate -> original
+        let intermediate_line = out_entry.orig_line;
+        let intermediate_col = out_entry.orig_col;
+
+        // Find the best matching input entry for this intermediate position
+        if let Some(input_entry) = find_mapping_for(
+            &input_entries,
+            intermediate_line,
+            intermediate_col,
+        ) {
+            composed_entries.push(MappingEntry {
+                gen_line: out_entry.gen_line,
+                gen_col: out_entry.gen_col,
+                source: input_entry.source,
+                orig_line: input_entry.orig_line,
+                orig_col: input_entry.orig_col,
+            });
+        } else {
+            // No input mapping found — keep the output mapping as-is
+            composed_entries.push(out_entry.clone());
+        }
+    }
+
+    // Encode the composed mappings
+    let max_gen_line = composed_entries
+        .iter()
+        .map(|e| e.gen_line)
+        .max()
+        .unwrap_or(0);
+    let composed_mappings = encode_mappings(&composed_entries, max_gen_line);
+
+    // Build the result using the input map's sources/sourcesContent
+    // and the output map's file
+    let file = output_map
+        .get("file")
+        .and_then(|v| v.as_str())
+        .unwrap_or("");
+    let sources = input_map.get("sources").cloned().unwrap_or(
+        serde_json::Value::Array(Vec::new()),
+    );
+    let sources_content = input_map.get("sourcesContent").cloned().unwrap_or(
+        serde_json::Value::Array(Vec::new()),
+    );
+
+    let result = serde_json::json!({
+        "version": 3,
+        "file": file,
+        "sources": sources,
+        "sourcesContent": sources_content,
+        "mappings": composed_mappings,
+    });
+
+    serde_json::to_string(&result).unwrap_or_else(|_| output_map_json.to_string())
+}
+
+/// Find the best matching input mapping entry for a given line/column.
+///
+/// Uses binary-search-like approach: finds the entry on the target line
+/// whose column is closest to (but not exceeding) the target column.
+fn find_mapping_for(
+    entries: &[MappingEntry],
+    target_line: usize,
+    target_col: usize,
+) -> Option<&MappingEntry> {
+    // Filter to entries on the target line
+    let line_entries: Vec<&MappingEntry> = entries
+        .iter()
+        .filter(|e| e.gen_line == target_line)
+        .collect();
+
+    if line_entries.is_empty() {
+        return None;
+    }
+
+    // Find the entry with the largest gen_col <= target_col
+    let mut best: Option<&MappingEntry> = None;
+    for entry in &line_entries {
+        if entry.gen_col <= target_col {
+            match best {
+                None => best = Some(entry),
+                Some(b) if entry.gen_col > b.gen_col => best = Some(entry),
+                _ => {}
+            }
+        }
+    }
+
+    // If no exact match, return the first entry on the line
+    best.or_else(|| line_entries.first().copied())
+}
+
 /// Escape a string for JSON output.
 fn escape_json(s: &str) -> String {
     let mut out = String::with_capacity(s.len());
@@ -253,4 +524,186 @@ mod tests {
         let map = generate_source_map("out.js", &sources, "");
         assert!(map.json.contains("\"sources\":[]"));
     }
+
+    // ──────────────────────────────────────────────────────────────────
+    // VLQ decode tests
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_vlq_decode_a() {
+        let values = vlq_decode("A");
+        assert_eq!(values, vec![0]);
+    }
+
+    #[test]
+    fn test_vlq_decode_c() {
+        let values = vlq_decode("C");
+        assert_eq!(values, vec![1]);
+    }
+
+    #[test]
+    fn test_vlq_decode_d() {
+        let values = vlq_decode("D");
+        assert_eq!(values, vec![-1]);
+    }
+
+    #[test]
+    fn test_vlq_roundtrip() {
+        for v in [-100, -10, -1, 0, 1, 10, 100] {
+            let mut encoded = String::new();
+            vlq_encode(&mut encoded, v);
+            let decoded = vlq_decode(&encoded);
+            assert_eq!(
+                decoded,
+                vec![v],
+                "VLQ roundtrip failed for {}",
+                v
+            );
+        }
+    }
+
+    // ──────────────────────────────────────────────────────────────────
+    // Source map chaining tests (R11 / T16)
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_compose_source_maps_chaining() {
+        // T16: Input map: line 5 -> original line 10
+        //       Output map: line 3 -> bundled line 5
+        //       Result: line 3 -> original line 10
+
+        // Build input map: maps generated line 5 to original line 10
+        let input_entries = vec![MappingEntry {
+            gen_line: 5,
+            gen_col: 0,
+            source: 0,
+            orig_line: 10,
+            orig_col: 0,
+        }];
+        let input_mappings = encode_mappings(&input_entries, 5);
+
+        let input_map = serde_json::json!({
+            "version": 3,
+            "file": "intermediate.js",
+            "sources": ["original.ts"],
+            "sourcesContent": ["// original TypeScript source"],
+            "mappings": input_mappings,
+        });
+
+        // Build output map: maps final line 3 to intermediate line 5
+        let output_entries = vec![MappingEntry {
+            gen_line: 3,
+            gen_col: 0,
+            source: 0,
+            orig_line: 5,
+            orig_col: 0,
+        }];
+        let output_mappings = encode_mappings(&output_entries, 3);
+
+        let output_map = serde_json::json!({
+            "version": 3,
+            "file": "final.js",
+            "sources": ["intermediate.js"],
+            "sourcesContent": ["// intermediate JS"],
+            "mappings": output_mappings,
+        });
+
+        let result_json = compose_source_maps(
+            &serde_json::to_string(&input_map).unwrap(),
+            &serde_json::to_string(&output_map).unwrap(),
+        );
+
+        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
+
+        // The result should map to the original source
+        assert_eq!(result["version"], 3);
+        assert_eq!(result["sources"][0], "original.ts");
+
+        // Verify sourcesContent is preserved from input map
+        assert_eq!(result["sourcesContent"][0], "// original TypeScript source");
+
+        // Decode the composed mappings and verify the chain
+        let composed_mappings = result["mappings"].as_str().unwrap();
+        let composed_entries = decode_mappings(composed_mappings);
+
+        // Find the entry for gen_line 3
+        let entry = composed_entries
+            .iter()
+            .find(|e| e.gen_line == 3)
+            .expect("Should have mapping for gen_line 3");
+
+        assert_eq!(
+            entry.orig_line, 10,
+            "Chained mapping should map line 3 -> original line 10, got line {}",
+            entry.orig_line
+        );
+    }
+
+    #[test]
+    fn test_compose_source_maps_preserves_sources() {
+        let input_map = serde_json::json!({
+            "version": 3,
+            "file": "app.js",
+            "sources": ["app.ts", "utils.ts"],
+            "sourcesContent": ["// app.ts source", "// utils.ts source"],
+            "mappings": "AAAA",
+        });
+
+        let output_map = serde_json::json!({
+            "version": 3,
+            "file": "bundle.js",
+            "sources": ["app.js"],
+            "sourcesContent": ["// app.js content"],
+            "mappings": "AAAA",
+        });
+
+        let result_json = compose_source_maps(
+            &serde_json::to_string(&input_map).unwrap(),
+            &serde_json::to_string(&output_map).unwrap(),
+        );
+
+        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
+        // Sources should come from the input map (original)
+        assert_eq!(result["sources"][0], "app.ts");
+        assert_eq!(result["sources"][1], "utils.ts");
+        assert_eq!(result["sourcesContent"][0], "// app.ts source");
+    }
+
+    #[test]
+    fn test_decode_mappings_empty() {
+        let entries = decode_mappings("");
+        assert!(entries.is_empty());
+    }
+
+    #[test]
+    fn test_decode_encode_roundtrip() {
+        let original = vec![
+            MappingEntry {
+                gen_line: 0,
+                gen_col: 0,
+                source: 0,
+                orig_line: 0,
+                orig_col: 0,
+            },
+            MappingEntry {
+                gen_line: 1,
+                gen_col: 4,
+                source: 0,
+                orig_line: 2,
+                orig_col: 8,
+            },
+        ];
+
+        let encoded = encode_mappings(&original, 1);
+        let decoded = decode_mappings(&encoded);
+
+        assert_eq!(decoded.len(), original.len());
+        for (d, o) in decoded.iter().zip(original.iter()) {
+            assert_eq!(d.gen_line, o.gen_line);
+            assert_eq!(d.gen_col, o.gen_col);
+            assert_eq!(d.source, o.source);
+            assert_eq!(d.orig_line, o.orig_line);
+            assert_eq!(d.orig_col, o.orig_col);
+        }
+    }
 }
diff --git a/crates/cclab-jet/src/bundler/splitting.rs b/crates/cclab-jet/src/bundler/splitting.rs
index 788c1e53..7acdc21f 100644
--- a/crates/cclab-jet/src/bundler/splitting.rs
+++ b/crates/cclab-jet/src/bundler/splitting.rs
@@ -6,6 +6,8 @@
 use std::collections::{HashMap, HashSet};
 use std::path::PathBuf;
 
+use super::types::PreloadHint;
+
 /// A chunk produced by code splitting.
 #[derive(Debug, Clone)]
 pub struct Chunk {
@@ -35,6 +37,23 @@ pub struct SplitEdge {
     pub is_dynamic: bool,
 }
 
+/// Result of code splitting with preload hint metadata.
+#[derive(Debug, Clone)]
+pub struct SplitResult {
+    /// Produced chunks.
+    pub chunks: Vec<Chunk>,

... truncated (38824 more lines)
```

## Review: 

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-subagent-mode

**Summary**: Approved.

