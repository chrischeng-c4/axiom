---
id: implementation
type: change_implementation
change_id: mangle-module-scope
---

# Implementation

## Summary

*(auto-generated baseline from git diff)*

## Changed Files

```
M	.claude/settings.json
D	.claude/skills/cclab-sdd-merge/SKILL.md
D	.claude/skills/cclab-sdd-revise-artifact/SKILL.md
M	.codex/config.toml
M	.gemini/policies/sdd-agent.toml
M	.gemini/settings.json
M	Cargo.lock
M	Cargo.toml
D	cclab/archive/20260318-sdd-frontend-doc-support/STATE.yaml
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/pre_clarifications.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/analyze_spec_sdd-frontend-doc-support-spec.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/begin_implementation.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/create_pre_clarifications.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/fill_spec_sdd-frontend-doc-support-spec_overview.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/fill_spec_sdd-frontend-doc-support-spec_requirements.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/fill_spec_sdd-frontend-doc-support-spec_scenarios.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/review_impl_sdd-frontend-doc-support-spec.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/revise_change_implementation.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/reference_context.md
D	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/requirements.md
D	cclab/archive/20260318-sdd-frontend-doc-support/implementation.md
D	cclab/archive/20260318-sdd-frontend-doc-support/issues/issue_897_feat-sdd-add-wireframe-yaml-dsl-for-frontend-inter.md
D	cclab/archive/20260318-sdd-frontend-doc-support/issues/issue_898_feat-sdd-support-user-facing-doc-as-change-artifac.md
D	cclab/archive/20260318-sdd-frontend-doc-support/payloads/create-change-implementation.json
D	cclab/archive/20260318-sdd-frontend-doc-support/payloads/create-post-clarifications.json
D	cclab/archive/20260318-sdd-frontend-doc-support/payloads/create-pre-clarifications.json
D	cclab/archive/20260318-sdd-frontend-doc-support/payloads/create-reference-context.json
D	cclab/archive/20260318-sdd-frontend-doc-support/payloads/restructure-input.json
D	cclab/archive/20260318-sdd-frontend-doc-support/payloads/review-change-implementation.json
D	cclab/archive/20260318-sdd-frontend-doc-support/payloads/review-reference-context.json
D	cclab/archive/20260318-sdd-frontend-doc-support/payloads/revise-reference-context.json
D	cclab/archive/20260318-sdd-frontend-doc-support/prompts/create_change_merge.md
D	cclab/archive/20260318-sdd-frontend-doc-support/specs/sdd-frontend-doc-support-spec.md
D	cclab/archive/20260318-sdd-frontend-doc-support/user_input.md
D	cclab/archive/20260318-sdd-workflow-cleanup/STATE.yaml
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/config-unification/pre_clarifications.md
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/config-unification/prompts/create_pre_clarifications.md
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/config-unification/requirements.md
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/spec-plan/pre_clarifications.md
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/spec-plan/prompts/create_pre_clarifications.md
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/spec-plan/requirements.md
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/tools-cleanup/pre_clarifications.md
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/tools-cleanup/prompts/create_pre_clarifications.md
D	cclab/archive/20260318-sdd-workflow-cleanup/groups/tools-cleanup/requirements.md
D	cclab/archive/20260318-sdd-workflow-cleanup/issues/issue_644_refactor-sdd-remove-sdd-read-artifact-and-sdd-writ.md
D	cclab/archive/20260318-sdd-workflow-cleanup/issues/issue_884_unify-workflow-config-merge-agent-label-settings-p.md
D	cclab/archive/20260318-sdd-workflow-cleanup/issues/issue_886_spec-plan-in-reference-context-auto-determine-main.md
D	cclab/archive/20260318-sdd-workflow-cleanup/payloads/pre-clarifications-config-unification.json
D	cclab/archive/20260318-sdd-workflow-cleanup/payloads/pre-clarifications-spec-plan.json
D	cclab/archive/20260318-sdd-workflow-cleanup/payloads/pre-clarifications-tools-cleanup.json
D	cclab/archive/20260318-sdd-workflow-cleanup/user_input.md
A	cclab/changes/all-jet-issues/STATE.yaml
R069	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/post_clarifications.md	cclab/changes/all-jet-issues/groups/jet-build-aot/post_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-build-aot/pre_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-build-aot/prompts/create_post_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-build-aot/prompts/create_pre_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-build-aot/prompts/create_reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-build-aot/prompts/review_reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-build-aot/prompts/revise_reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-build-aot/reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-build-aot/requirements.md
A	cclab/changes/all-jet-issues/groups/jet-infra-codesign/post_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-infra-codesign/pre_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-infra-codesign/prompts/create_post_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-infra-codesign/prompts/create_pre_clarifications.md
R054	cclab/archive/20260318-sdd-workflow-cleanup/groups/config-unification/prompts/create_reference_context.md	cclab/changes/all-jet-issues/groups/jet-infra-codesign/prompts/create_reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-infra-codesign/prompts/review_reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-infra-codesign/reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-infra-codesign/requirements.md
A	cclab/changes/all-jet-issues/groups/jet-install-optimizations/post_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-install-optimizations/pre_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-install-optimizations/prompts/create_post_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-install-optimizations/prompts/create_pre_clarifications.md
A	cclab/changes/all-jet-issues/groups/jet-install-optimizations/prompts/review_reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-install-optimizations/prompts/revise_reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-install-optimizations/reference_context.md
A	cclab/changes/all-jet-issues/groups/jet-install-optimizations/requirements.md
A	cclab/changes/all-jet-issues/implementation.md
A	cclab/changes/all-jet-issues/issues/issue_765_feat-jet-aot-production-build-tree-shaking-code-sp.md
A	cclab/changes/all-jet-issues/issues/issue_797_jet-build-validate-against-real-world-open-source-.md
A	cclab/changes/all-jet-issues/issues/issue_881_jet-install-cold-install-4-9s-vs-pnpm-3-4s-optimiz.md
A	cclab/changes/all-jet-issues/issues/issue_882_jet-build-bundle-size-215kb-vs-webpack-192kb-imple.md
A	cclab/changes/all-jet-issues/issues/issue_883_jet-install-resolver-bugs-fixed-version-conflicts-.md
A	cclab/changes/all-jet-issues/issues/issue_903_jet-build-scope-hoisting-phase-2-true-module-flatt.md
A	cclab/changes/all-jet-issues/issues/issue_904_jet-build-dce-and-minifier-assume-ascii-only-sourc.md
A	cclab/changes/all-jet-issues/issues/issue_905_jet-install-validate-disk-cache-http-2-performance.md
A	cclab/changes/all-jet-issues/issues/issue_906_chore-jet-codesign-release-binary-macos-kills-unsi.md
A	cclab/changes/all-jet-issues/payloads/create-change-implementation.json
A	cclab/changes/all-jet-issues/payloads/create-post-clarifications.json
A	cclab/changes/all-jet-issues/payloads/create-pre-clarifications.json
A	cclab/changes/all-jet-issues/payloads/review-reference-context.json
A	cclab/changes/all-jet-issues/payloads/spec-changes.json
A	cclab/changes/all-jet-issues/prompts/analyze_spec_all-jet-issues-spec.md
A	cclab/changes/all-jet-issues/prompts/begin_implementation.md
A	cclab/changes/all-jet-issues/prompts/fill_spec_all-jet-issues-spec_overview.md
A	cclab/changes/all-jet-issues/prompts/fill_spec_all-jet-issues-spec_requirements.md
A	cclab/changes/all-jet-issues/prompts/fill_spec_all-jet-issues-spec_scenarios.md
R081	cclab/archive/20260318-sdd-workflow-cleanup/prompts/restructure_input.md	cclab/changes/all-jet-issues/prompts/restructure_input.md
R059	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/write_implementation_diff.md	cclab/changes/all-jet-issues/prompts/write_implementation_diff.md
A	cclab/changes/all-jet-issues/specs/all-jet-issues-spec.md
A	cclab/changes/all-jet-issues/user_input.md
A	cclab/changes/all-open-jet-issues/STATE.yaml
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/post_clarifications.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/pre_clarifications.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/prompts/create_post_clarifications.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/prompts/create_pre_clarifications.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/prompts/create_reference_context.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/prompts/review_reference_context.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/prompts/revise_reference_context.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/reference_context.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/requirements.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-real-world-validation/post_clarifications.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-real-world-validation/pre_clarifications.md
R062	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/create_post_clarifications.md	cclab/changes/all-open-jet-issues/groups/jet-build-real-world-validation/prompts/create_post_clarifications.md
R051	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/create_reference_context.md	cclab/changes/all-open-jet-issues/groups/jet-build-real-world-validation/prompts/create_reference_context.md
R062	cclab/archive/20260318-sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/prompts/review_reference_context.md	cclab/changes/all-open-jet-issues/groups/jet-build-real-world-validation/prompts/review_reference_context.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-real-world-validation/reference_context.md
A	cclab/changes/all-open-jet-issues/groups/jet-build-real-world-validation/requirements.md
A	cclab/changes/all-open-jet-issues/groups/jet-install-optimizations/post_clarifications.md
A	cclab/changes/all-open-jet-issues/groups/jet-install-optimizations/pre_clarifications.md
A	cclab/changes/all-open-jet-issues/groups/jet-install-optimizations/prompts/create_post_clarifications.md
A	cclab/changes/all-open-jet-issues/groups/jet-install-optimizations/prompts/review_reference_context.md
A	cclab/changes/all-open-jet-issues/groups/jet-install-optimizations/reference_context.md
A	cclab/changes/all-open-jet-issues/groups/jet-install-optimizations/requirements.md
A	cclab/changes/all-open-jet-issues/implementation.md
A	cclab/changes/all-open-jet-issues/issues/issue_765_feat-jet-aot-production-build-tree-shaking-code-sp.md
A	cclab/changes/all-open-jet-issues/issues/issue_797_jet-build-validate-against-real-world-open-source-.md
A	cclab/changes/all-open-jet-issues/issues/issue_881_jet-install-cold-install-4-9s-vs-pnpm-3-4s-optimiz.md
A	cclab/changes/all-open-jet-issues/issues/issue_882_jet-build-bundle-size-215kb-vs-webpack-192kb-imple.md
A	cclab/changes/all-open-jet-issues/issues/issue_883_jet-install-resolver-bugs-fixed-version-conflicts-.md
A	cclab/changes/all-open-jet-issues/issues/issue_903_jet-build-scope-hoisting-phase-2-true-module-flatt.md
A	cclab/changes/all-open-jet-issues/payloads/create-change-implementation.json
A	cclab/changes/all-open-jet-issues/payloads/create-ref-ctx-install.json
A	cclab/changes/all-open-jet-issues/payloads/create-ref-ctx-validation.json
A	cclab/changes/all-open-jet-issues/payloads/post-clarifications.json
A	cclab/changes/all-open-jet-issues/payloads/review-change-implementation.json
A	cclab/changes/all-open-jet-issues/payloads/review-reference-context-install.json
A	cclab/changes/all-open-jet-issues/payloads/review-reference-context.json
A	cclab/changes/all-open-jet-issues/payloads/revise-ref-ctx-install.json
A	cclab/changes/all-open-jet-issues/prompts/analyze_spec_all-open-jet-issues-spec.md
A	cclab/changes/all-open-jet-issues/prompts/begin_implementation.md
R080	cclab/archive/20260318-sdd-frontend-doc-support/prompts/restructure_input.md	cclab/changes/all-open-jet-issues/prompts/restructure_input.md
A	cclab/changes/all-open-jet-issues/prompts/review_impl_all-open-jet-issues-spec.md
A	cclab/changes/all-open-jet-issues/prompts/write_implementation_diff.md
A	cclab/changes/all-open-jet-issues/specs/all-open-jet-issues-spec.md
A	cclab/changes/all-open-jet-issues/user_input.md
M	cclab/specs/cclab-jet/pkg-manager.md
A	cclab/specs/cclab-jet/scope-hoisting.md
M	cclab/specs/cclab-jet/tree-shaking.md
M	cclab/specs/cclab-jet/variable-mangling.md
M	cclab/specs/cclab-sdd/config/agents.md
M	cclab/specs/cclab-sdd/config/platform.md
M	cclab/specs/cclab-sdd/generate/architecture.md
M	cclab/specs/cclab-sdd/generate/mermaid-plus-conversion.md
M	cclab/specs/cclab-sdd/generate/spec-model.md
M	cclab/specs/cclab-sdd/generate/template-mcp-configs.md
M	cclab/specs/cclab-sdd/interfaces/cli/commands.md
M	cclab/specs/cclab-sdd/logic/change-merge.md
M	cclab/specs/cclab-sdd/logic/change-spec.md
M	cclab/specs/cclab-sdd/logic/executor-resolution.md
M	cclab/specs/cclab-sdd/logic/implement-task.md
M	cclab/specs/cclab-sdd/logic/reference-context.md
M	cclab/specs/cclab-sdd/tools/utils/platform-sync.md
M	crates/cclab-jet/src/bundler/mangle.rs
M	crates/cclab-jet/src/bundler/minify.rs
M	crates/cclab-jet/src/bundler/mod.rs
M	crates/cclab-jet/src/bundler/scope_hoist.rs
M	crates/cclab-jet/src/bundler/tree_shake.rs
M	crates/cclab-jet/src/cli.rs
M	crates/cclab-jet/src/pkg_manager/mod.rs
M	crates/cclab-jet/src/pkg_manager/registry.rs
M	crates/cclab-jet/src/pkg_manager/resolver.rs
M	crates/cclab-sdd-cli/src/init.rs
M	crates/cclab-sdd/src/cli/spec.rs
D	crates/cclab-sdd/src/generators/async_api.rs
D	crates/cclab-sdd/src/generators/changes.rs
D	crates/cclab-sdd/src/generators/db_model.rs
D	crates/cclab-sdd/src/generators/dependency.rs
D	crates/cclab-sdd/src/generators/doc.rs
D	crates/cclab-sdd/src/generators/flowchart.rs
D	crates/cclab-sdd/src/generators/frontend.rs
D	crates/cclab-sdd/src/generators/mindmap.rs
D	crates/cclab-sdd/src/generators/mod.rs
D	crates/cclab-sdd/src/generators/overview.rs
D	crates/cclab-sdd/src/generators/requirements.rs
D	crates/cclab-sdd/src/generators/rest_api.rs
D	crates/cclab-sdd/src/generators/rpc_api.rs
D	crates/cclab-sdd/src/generators/scenarios.rs
D	crates/cclab-sdd/src/generators/sequence.rs
D	crates/cclab-sdd/src/generators/serverless_workflow.rs
D	crates/cclab-sdd/src/generators/state_machine.rs
D	crates/cclab-sdd/src/generators/test_plan.rs
M	crates/cclab-sdd/src/lib.rs
M	crates/cclab-sdd/src/models/mod.rs
D	crates/cclab-sdd/src/models/section.rs
M	crates/cclab-sdd/src/models/spec_rules.rs
D	crates/cclab-sdd/src/prompts/section_prompts.yaml
M	crates/cclab-sdd/src/services/mod.rs
M	crates/cclab-sdd/src/services/spec_service.rs
M	crates/cclab-sdd/src/tools/agent.rs
M	crates/cclab-sdd/src/tools/clarifications.rs
M	crates/cclab-sdd/src/tools/common_change_impl.rs
M	crates/cclab-sdd/src/tools/common_change_spec.rs
M	crates/cclab-sdd/src/tools/common_reference_context.rs
M	crates/cclab-sdd/src/tools/create_change_impl.rs
M	crates/cclab-sdd/src/tools/create_change_merge.rs
M	crates/cclab-sdd/src/tools/create_change_spec.rs
M	crates/cclab-sdd/src/tools/create_post_clarifications.rs
M	crates/cclab-sdd/src/tools/create_pre_clarifications.rs
M	crates/cclab-sdd/src/tools/create_reference_context.rs
M	crates/cclab-sdd/src/tools/mod.rs
M	crates/cclab-sdd/src/tools/review_change_impl.rs
M	crates/cclab-sdd/src/tools/review_change_spec.rs
M	crates/cclab-sdd/src/tools/review_reference_context.rs
M	crates/cclab-sdd/src/tools/revise_change_spec.rs
M	crates/cclab-sdd/src/tools/revise_reference_context.rs
M	crates/cclab-sdd/src/tools/spec.rs
D	crates/cclab-sdd/src/tools/spec_plan.rs
M	crates/cclab-sdd/src/tools/validate_spec.rs
M	crates/cclab-sdd/src/validator/semantic.rs
M	crates/cclab-sdd/src/workflow/helpers.rs
```

## Diff Statistics

```
.claude/settings.json                              |    2 -
 .claude/skills/cclab-sdd-merge/SKILL.md            |   33 -
 .claude/skills/cclab-sdd-revise-artifact/SKILL.md  |   34 -
 .codex/config.toml                                 |    4 +
 .gemini/policies/sdd-agent.toml                    |   25 +-
 .gemini/settings.json                              |   12 +-
 Cargo.lock                                         |  126 +--
 Cargo.toml                                         |    2 +-
 .../20260318-sdd-frontend-doc-support/STATE.yaml   |   88 --
 .../pre_clarifications.md                          |   30 -
 .../analyze_spec_sdd-frontend-doc-support-spec.md  |   13 -
 .../prompts/begin_implementation.md                |   19 -
 .../prompts/create_pre_clarifications.md           |   29 -
 ..._spec_sdd-frontend-doc-support-spec_overview.md |    5 -
 ...c_sdd-frontend-doc-support-spec_requirements.md |    9 -
 ...spec_sdd-frontend-doc-support-spec_scenarios.md |    5 -
 .../review_impl_sdd-frontend-doc-support-spec.md   |   29 -
 .../prompts/revise_change_implementation.md        |   19 -
 .../reference_context.md                           |   18 -
 .../sdd-frontend-doc-artifacts/requirements.md     |   28 -
 .../implementation.md                              | 1191 --------------------
 ...dd-add-wireframe-yaml-dsl-for-frontend-inter.md |   47 -
 ...dd-support-user-facing-doc-as-change-artifac.md |   42 -
 .../payloads/create-change-implementation.json     |    6 -
 .../payloads/create-post-clarifications.json       |    6 -
 .../payloads/create-pre-clarifications.json        |   34 -
 .../payloads/create-reference-context.json         |   70 --
 .../payloads/restructure-input.json                |   39 -
 .../payloads/review-change-implementation.json     |   51 -
 .../payloads/review-reference-context.json         |   51 -
 .../payloads/revise-reference-context.json         |   74 --
 .../prompts/create_change_merge.md                 |   11 -
 .../specs/sdd-frontend-doc-support-spec.md         |   94 --
 .../user_input.md                                  |    1 -
 .../20260318-sdd-workflow-cleanup/STATE.yaml       |   67 --
 .../config-unification/pre_clarifications.md       |   17 -
 .../prompts/create_pre_clarifications.md           |   29 -
 .../groups/config-unification/requirements.md      |    9 -
 .../groups/spec-plan/pre_clarifications.md         |   17 -
 .../spec-plan/prompts/create_pre_clarifications.md |   29 -
 .../groups/spec-plan/requirements.md               |    9 -
 .../groups/tools-cleanup/pre_clarifications.md     |   17 -
 .../prompts/create_pre_clarifications.md           |   29 -
 .../groups/tools-cleanup/requirements.md           |    9 -
 ...or-sdd-remove-sdd-read-artifact-and-sdd-writ.md |   27 -
 ...workflow-config-merge-agent-label-settings-p.md |   61 -
 ...lan-in-reference-context-auto-determine-main.md |   61 -
 .../pre-clarifications-config-unification.json     |   14 -
 .../payloads/pre-clarifications-spec-plan.json     |   14 -
 .../payloads/pre-clarifications-tools-cleanup.json |   14 -
 .../20260318-sdd-workflow-cleanup/user_input.md    |    1 -
 cclab/changes/all-jet-issues/STATE.yaml            |  161 +++
 .../groups/jet-build-aot}/post_clarifications.md   |    4 +-
 .../groups/jet-build-aot/pre_clarifications.md     |   18 +
 .../prompts/create_post_clarifications.md          |   48 +
 .../prompts/create_pre_clarifications.md           |   29 +
 .../prompts/create_reference_context.md            |   38 +
 .../prompts/review_reference_context.md            |   24 +
 .../prompts/revise_reference_context.md            |   23 +
 .../groups/jet-build-aot/reference_context.md      |   16 +
 .../groups/jet-build-aot/requirements.md           |    9 +
 .../jet-infra-codesign/post_clarifications.md      |   10 +
 .../jet-infra-codesign/pre_clarifications.md       |   12 +
 .../prompts/create_post_clarifications.md          |   48 +
 .../prompts/create_pre_clarifications.md           |   29 +
 .../prompts/create_reference_context.md            |   20 +-
 .../prompts/review_reference_context.md            |   24 +
 .../groups/jet-infra-codesign/reference_context.md |   13 +
 .../groups/jet-infra-codesign/requirements.md      |    9 +
 .../post_clarifications.md                         |   10 +
 .../pre_clarifications.md                          |   15 +
 .../prompts/create_post_clarifications.md          |   48 +
 .../prompts/create_pre_clarifications.md           |   29 +
 .../prompts/review_reference_context.md            |   24 +
 .../prompts/revise_reference_context.md            |   23 +
 .../jet-install-optimizations/reference_context.md |   14 +
 .../jet-install-optimizations/requirements.md      |    9 +
 cclab/changes/all-jet-issues/implementation.md     |   71 ++
 ...et-aot-production-build-tree-shaking-code-sp.md |  140 +++
 ...ild-validate-against-real-world-open-source-.md |   38 +
 ...stall-cold-install-4-9s-vs-pnpm-3-4s-optimiz.md |   44 +
 ...ild-bundle-size-215kb-vs-webpack-192kb-imple.md |   54 +
 ...stall-resolver-bugs-fixed-version-conflicts-.md |   47 +
 ...ild-scope-hoisting-phase-2-true-module-flatt.md |   47 +
 ...ild-dce-and-minifier-assume-ascii-only-sourc.md |   46 +
 ...stall-validate-disk-cache-http-2-performance.md |   40 +
 ...jet-codesign-release-binary-macos-kills-unsi.md |   29 +
 .../payloads/create-change-implementation.json     |    4 +
 .../payloads/create-post-clarifications.json       |    5 +
 .../payloads/create-pre-clarifications.json        |    9 +
 .../payloads/review-reference-context.json         |   59 +
 .../all-jet-issues/payloads/spec-changes.json      |    5 +
 .../prompts/analyze_spec_all-jet-issues-spec.md    |   13 +
 .../all-jet-issues/prompts/begin_implementation.md |   19 +
 .../fill_spec_all-jet-issues-spec_overview.md      |    5 +
 .../fill_spec_all-jet-issues-spec_requirements.md  |    9 +
 .../fill_spec_all-jet-issues-spec_scenarios.md     |    5 +
 .../all-jet-issues}/prompts/restructure_input.md   |    8 +-
 .../prompts/write_implementation_diff.md           |    4 +-
 .../all-jet-issues/specs/all-jet-issues-spec.md    |   15 +
 cclab/changes/all-jet-issues/user_input.md         |    1 +
 cclab/changes/all-open-jet-issues/STATE.yaml       |  143 +++
 .../post_clarifications.md                         |   10 +
 .../jet-build-aot-production/pre_clarifications.md |   21 +
 .../prompts/create_post_clarifications.md          |   48 +
 .../prompts/create_pre_clarifications.md           |   29 +
 .../prompts/create_reference_context.md            |   39 +
 .../prompts/review_reference_context.md            |   24 +
 .../prompts/revise_reference_context.md            |   23 +
 .../jet-build-aot-production/reference_context.md  |   20 +
 .../jet-build-aot-production/requirements.md       |    9 +
 .../post_clarifications.md                         |   10 +
 .../pre_clarifications.md                          |   12 +
 .../prompts/create_post_clarifications.md          |   16 +-
 .../prompts/create_reference_context.md            |   21 +-
 .../prompts/review_reference_context.md            |   10 +-
 .../reference_context.md                           |   15 +
 .../requirements.md                                |    9 +
 .../post_clarifications.md                         |   10 +
 .../pre_clarifications.md                          |   15 +
 .../prompts/create_post_clarifications.md          |   48 +
 .../prompts/review_reference_context.md            |   24 +
 .../jet-install-optimizations/reference_context.md |   14 +
 .../jet-install-optimizations/requirements.md      |    9 +
 .../changes/all-open-jet-issues/implementation.md  |   80 ++
 ...et-aot-production-build-tree-shaking-code-sp.md |  140 +++
 ...ild-validate-against-real-world-open-source-.md |   38 +
 ...stall-cold-install-4-9s-vs-pnpm-3-4s-optimiz.md |   44 +
 ...ild-bundle-size-215kb-vs-webpack-192kb-imple.md |   54 +
 ...stall-resolver-bugs-fixed-version-conflicts-.md |   47 +
 ...ild-scope-hoisting-phase-2-true-module-flatt.md |   47 +
 .../payloads/create-change-implementation.json     |    4 +
 .../payloads/create-ref-ctx-install.json           |    7 +
 .../payloads/create-ref-ctx-validation.json        |    8 +
 .../payloads/post-clarifications.json              |    5 +
 .../payloads/review-change-implementation.json     |   23 +
 .../payloads/review-reference-context-install.json |   44 +
 .../payloads/review-reference-context.json         |   49 +
 .../payloads/revise-ref-ctx-install.json           |    7 +
 .../analyze_spec_all-open-jet-issues-spec.md       |   13 +
 .../prompts/begin_implementation.md                |   19 +
 .../prompts/restructure_input.md                   |    8 +-
 .../review_impl_all-open-jet-issues-spec.md        |   29 +
 .../prompts/write_implementation_diff.md           |   14 +
 .../specs/all-open-jet-issues-spec.md              |  153 +++
 cclab/changes/all-open-jet-issues/user_input.md    |    1 +
 cclab/specs/cclab-jet/pkg-manager.md               |    4 +-
 cclab/specs/cclab-jet/scope-hoisting.md            |   97 ++
 cclab/specs/cclab-jet/tree-shaking.md              |   15 +-
 cclab/specs/cclab-jet/variable-mangling.md         |   12 +-
 cclab/specs/cclab-sdd/config/agents.md             |   20 +-
 cclab/specs/cclab-sdd/config/platform.md           |   60 +-
 cclab/specs/cclab-sdd/generate/architecture.md     |    2 +-
 .../cclab-sdd/generate/mermaid-plus-conversion.md  |   35 +-
 cclab/specs/cclab-sdd/generate/spec-model.md       |   55 +-
 .../cclab-sdd/generate/template-mcp-configs.md     |   11 +-
 cclab/specs/cclab-sdd/interfaces/cli/commands.md   |   21 +-
 cclab/specs/cclab-sdd/logic/change-merge.md        |   11 +-
 cclab/specs/cclab-sdd/logic/change-spec.md         |  422 ++-----
 cclab/specs/cclab-sdd/logic/executor-resolution.md |   29 +-
 cclab/specs/cclab-sdd/logic/implement-task.md      |   16 +-
 cclab/specs/cclab-sdd/logic/reference-context.md   |   98 +-
 cclab/specs/cclab-sdd/tools/utils/platform-sync.md |   43 +-
 crates/cclab-jet/src/bundler/mangle.rs             |  110 +-
 crates/cclab-jet/src/bundler/minify.rs             |   73 ++
 crates/cclab-jet/src/bundler/mod.rs                |   16 +-
 crates/cclab-jet/src/bundler/scope_hoist.rs        | 1005 ++++++++++++++++-
 crates/cclab-jet/src/bundler/tree_shake.rs         |  285 +++++
 crates/cclab-jet/src/cli.rs                        |   28 +-
 crates/cclab-jet/src/pkg_manager/mod.rs            |  111 +-
 crates/cclab-jet/src/pkg_manager/registry.rs       |  141 ++-
 crates/cclab-jet/src/pkg_manager/resolver.rs       |   74 +-
 crates/cclab-sdd-cli/src/init.rs                   |  111 +-
 crates/cclab-sdd/src/cli/spec.rs                   |   63 +-
 crates/cclab-sdd/src/generators/async_api.rs       |  105 --
 crates/cclab-sdd/src/generators/changes.rs         |  105 --
 crates/cclab-sdd/src/generators/db_model.rs        |   89 --
 crates/cclab-sdd/src/generators/dependency.rs      |   87 --
 crates/cclab-sdd/src/generators/doc.rs             |   82 --
 crates/cclab-sdd/src/generators/flowchart.rs       |   83 --
 crates/cclab-sdd/src/generators/frontend.rs        |  103 --
 crates/cclab-sdd/src/generators/mindmap.rs         |   85 --
 crates/cclab-sdd/src/generators/mod.rs             |  347 ------
 crates/cclab-sdd/src/generators/overview.rs        |   81 --
 crates/cclab-sdd/src/generators/requirements.rs    |   90 --
 crates/cclab-sdd/src/generators/rest_api.rs        |  123 --
 crates/cclab-sdd/src/generators/rpc_api.rs         |  107 --
 crates/cclab-sdd/src/generators/scenarios.rs       |   88 --
 crates/cclab-sdd/src/generators/sequence.rs        |   82 --
 .../src/generators/serverless_workflow.rs          |  109 --
 crates/cclab-sdd/src/generators/state_machine.rs   |   84 --
 crates/cclab-sdd/src/generators/test_plan.rs       |  100 --
 crates/cclab-sdd/src/lib.rs                        |    1 -
 crates/cclab-sdd/src/models/mod.rs                 |    4 +-
 crates/cclab-sdd/src/models/section.rs             |  226 ----
 crates/cclab-sdd/src/models/spec_rules.rs          |  276 -----
 crates/cclab-sdd/src/prompts/section_prompts.yaml  |  348 ------
 crates/cclab-sdd/src/services/mod.rs               |    1 -
 crates/cclab-sdd/src/services/spec_service.rs      |    1 -
 crates/cclab-sdd/src/tools/agent.rs                |    3 +-
 crates/cclab-sdd/src/tools/clarifications.rs       |    5 +-
 crates/cclab-sdd/src/tools/common_change_impl.rs   |  111 +-
 crates/cclab-sdd/src/tools/common_change_spec.rs   |  237 +---
 .../src/tools/common_reference_context.rs          |  219 ----
 crates/cclab-sdd/src/tools/create_change_impl.rs   |    7 +-
 crates/cclab-sdd/src/tools/create_change_merge.rs  |   20 +-
 crates/cclab-sdd/src/tools/create_change_spec.rs   |  238 +---
 .../src/tools/create_post_clarifications.rs        |    9 +-
 .../src/tools/create_pre_clarifications.rs         |    9 +-
 .../src/tools/create_reference_context.rs          |  138 +--
 crates/cclab-sdd/src/tools/mod.rs                  |    1 -
 crates/cclab-sdd/src/tools/review_change_impl.rs   |   10 +-
 crates/cclab-sdd/src/tools/review_change_spec.rs   |   17 +-
 .../src/tools/review_reference_context.rs          |   10 +-
 crates/cclab-sdd/src/tools/revise_change_spec.rs   |    5 +-
 .../src/tools/revise_reference_context.rs          |   10 +-
 crates/cclab-sdd/src/tools/spec.rs                 |    1 -
 crates/cclab-sdd/src/tools/spec_plan.rs            |  672 -----------
 crates/cclab-sdd/src/tools/validate_spec.rs        |    1 -
 crates/cclab-sdd/src/validator/semantic.rs         |    1 -
 crates/cclab-sdd/src/workflow/helpers.rs           |   56 +-
 221 files changed, 5223 insertions(+), 7840 deletions(-)
```

## Diff

```diff
diff --git a/.claude/settings.json b/.claude/settings.json
index 060b9c03..60eb046b 100644
--- a/.claude/settings.json
+++ b/.claude/settings.json
@@ -8,8 +8,6 @@
       "Bash",
       "Read",
       "Agent",
-      "WebSearch",
-      "WebFetch",
       "Bash(git status*)",
       "Bash(git diff*)",
       "Bash(git log*)",
diff --git a/.claude/skills/cclab-sdd-merge/SKILL.md b/.claude/skills/cclab-sdd-merge/SKILL.md
deleted file mode 100644
index e5830f89..00000000
--- a/.claude/skills/cclab-sdd-merge/SKILL.md
+++ /dev/null
@@ -1,33 +0,0 @@
----
-name: cclab:sdd:merge
-description: Merge completed change — archive specs and implementation
-user-invocable: true
-auto-invoke: false
----
-
-# /cclab:sdd:merge
-
-Merges a completed SDD change: copies specs to `cclab/specs/`, archives the change directory, and advances STATE to `change_archived`.
-
-**This skill must be invoked by the user.** Do NOT invoke it automatically.
-
-## Change-ID Resolution
-
-If a change-id is provided, use it. Otherwise:
-
-- Check `cclab/changes/` for existing change directories whose `STATE.yaml` has `branch` matching the current git branch AND `phase` is not terminal (`archived` or `rejected`).
-  - Found → use that change's `change_id`.
-  - Not found → ask the user.
-
-## Instructions
-
-1. Resolve change-id
-2. Run: `cclab sdd workflow create-change-merge <change-id>`
-3. Report the result (specs merged, archive path)
-
-## Usage
-
-```
-/cclab:sdd:merge <change-id>
-/cclab:sdd:merge
-```
diff --git a/.claude/skills/cclab-sdd-revise-artifact/SKILL.md b/.claude/skills/cclab-sdd-revise-artifact/SKILL.md
deleted file mode 100644
index 3fc95111..00000000
--- a/.claude/skills/cclab-sdd-revise-artifact/SKILL.md
+++ /dev/null
@@ -1,34 +0,0 @@
----
-name: cclab:sdd:revise-artifact
-description: Revise change-spec and re-implement — fix design issues after review
-user-invocable: true
-auto-invoke: false
----
-
-# /cclab:sdd:revise-artifact
-
-Revises the change-spec and re-runs implementation when design issues are found after implementation review. Resets the workflow phase to re-enter the spec → implementation cycle.
-
-**This skill must be invoked by the user.** Do NOT invoke it automatically.
-
-## Change-ID Resolution
-
-If a change-id is provided, use it. Otherwise:
-
-- Check `cclab/changes/` for existing change directories whose `STATE.yaml` has `branch` matching the current git branch AND `phase` is not terminal (`archived` or `rejected`).
-  - Found → use that change's `change_id`.
-  - Not found → ask the user.
-
-## Instructions
-
-1. Resolve change-id
-2. Run: `cclab sdd revise-artifact <change-id> --description "<what needs to change>"`
-   - This resets the phase to `post_clarifications_created` and clears spec/impl artifacts
-3. Then run `/cclab:sdd:run-change <change-id>` to continue the workflow from spec creation
-
-## Usage
-
-```
-/cclab:sdd:revise-artifact <change-id> "<description of design changes needed>"
-/cclab:sdd:revise-artifact "<description of design changes needed>"
-```
diff --git a/.codex/config.toml b/.codex/config.toml
index e69de29b..3347822f 100644
--- a/.codex/config.toml
+++ b/.codex/config.toml
@@ -0,0 +1,4 @@
+[mcp_servers.cclab-mcp]
+disabled_tools = ["sdd_delegate_agent"]
+type = "http"
+url = "http://localhost:3456/mcp"
diff --git a/.gemini/policies/sdd-agent.toml b/.gemini/policies/sdd-agent.toml
index ce7204ea..fafe6dd4 100644
--- a/.gemini/policies/sdd-agent.toml
+++ b/.gemini/policies/sdd-agent.toml
@@ -1,28 +1,13 @@
-# SDD agent policy: allow only cclab CLI and payload writes
+# SDD agent policy: block local file/shell tools (agents use MCP tools only)
 
-# Allow cclab sdd artifact CLI commands
 [[rule]]
-toolName = "run_shell_command"
-commandPrefix = "cclab sdd artifact"
-decision = "allow"
-priority = 200
-
-# Block all other shell commands
-[[rule]]
-toolName = "run_shell_command"
+toolName = ["write_file", "edit", "replace"]
 decision = "deny"
 priority = 100
-deny_message = "Only 'cclab sdd artifact' commands are allowed"
+deny_message = "File writing blocked - use MCP sdd_write_artifact instead"
 
-# Allow writing payload JSON files only
 [[rule]]
-toolName = "write_file"
-decision = "allow"
-priority = 200
-
-# Block direct file editing (agents write via CLI payload + artifact command)
-[[rule]]
-toolName = ["edit", "replace"]
+toolName = "run_shell_command"
 decision = "deny"
 priority = 100
-deny_message = "Use cclab sdd artifact CLI instead of direct editing"
+deny_message = "Shell commands blocked - use MCP tools instead"
diff --git a/.gemini/settings.json b/.gemini/settings.json
index b8ac8fe8..9fa516e9 100644
--- a/.gemini/settings.json
+++ b/.gemini/settings.json
@@ -1,6 +1,16 @@
 {
   "mcp": {
+    "allowed": [
+      "cclab-mcp"
+    ]
   },
   "mcpServers": {
+    "cclab-mcp": {
+      "excludeTools": [
+        "sdd_delegate_agent"
+      ],
+      "type": "http",
+      "url": "http://localhost:3456/mcp"
+    }
   }
-}
+}
\ No newline at end of file
diff --git a/Cargo.lock b/Cargo.lock
index abc5d913..5dbd1b4d 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1167,7 +1167,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-agent-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "async-trait",
  "cclab-agent",
@@ -1233,7 +1233,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-api-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "async-trait",
  "cclab-api",
@@ -1247,7 +1247,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "rayon",
  "serde",
@@ -1257,7 +1257,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-array-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-array",
  "pyo3",
@@ -1265,7 +1265,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "cclab-api",
@@ -1302,7 +1302,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cli-registry"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "clap",
@@ -1311,7 +1311,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "thiserror 2.0.18",
@@ -1319,7 +1319,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-cmd-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-cmd",
  "pyo3",
@@ -1327,7 +1327,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-core"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "bson",
@@ -1345,7 +1345,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "aes-gcm",
  "argon2",
@@ -1371,7 +1371,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-crypto-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-crypto",
  "pyo3",
@@ -1379,7 +1379,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1404,7 +1404,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-fetch-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-core",
  "cclab-fetch",
@@ -1417,7 +1417,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-array",
  "rayon",
@@ -1429,7 +1429,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-frame-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-frame",
  "pyo3",
@@ -1437,7 +1437,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-core"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "bitvec",
  "regex",
@@ -1464,7 +1464,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-formula"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-grid-core",
  "nom 7.1.3",
@@ -1474,7 +1474,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-history"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1482,7 +1482,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-server"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "axum 0.7.9",
@@ -1506,7 +1506,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-grid-wasm"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-grid-core",
  "cclab-grid-formula",
@@ -1524,7 +1524,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-hive"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "bytes",
@@ -1540,7 +1540,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-hive-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-hive",
  "pyo3",
@@ -1551,7 +1551,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1592,7 +1592,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-jet-cli"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "cclab-cli-registry",
@@ -1604,7 +1604,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "async-trait",
  "bincode",
@@ -1631,7 +1631,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-kv-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-kv",
  "pyo3",
@@ -1642,7 +1642,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-array",
  "rayon",
@@ -1653,7 +1653,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-learn-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-learn",
  "pyo3",
@@ -1661,7 +1661,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-lens"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1698,7 +1698,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-log"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "chrono",
@@ -1714,7 +1714,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-log-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-log",
  "pyo3",
@@ -1722,7 +1722,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mamba"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "base64 0.22.1",
@@ -1751,7 +1751,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "image",
  "serde",
@@ -1761,7 +1761,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-media-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-media",
  "pyo3",
@@ -1769,7 +1769,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -1789,7 +1789,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-mongo-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "bson",
  "cclab-core",
@@ -1803,7 +1803,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1831,7 +1831,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-pg-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-pg",
  "parking_lot",
@@ -1843,7 +1843,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "serde",
  "tempfile",
@@ -1852,7 +1852,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-plot-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-plot",
  "pyo3",
@@ -1860,7 +1860,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
@@ -1890,7 +1890,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-qc-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "cclab-qc",
@@ -1902,7 +1902,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "async-nats",
  "async-trait",
@@ -1941,7 +1941,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-queue-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-queue",
  "chrono",
@@ -1954,7 +1954,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-razer"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "hidapi",
@@ -1963,7 +1963,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "async-trait",
  "cclab-core",
@@ -1988,7 +1988,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-runtime-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-runtime",
  "pyo3",
@@ -1996,7 +1996,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "bson",
  "dotenvy",
@@ -2010,7 +2010,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-schema-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-schema",
  "pyo3",
@@ -2018,7 +2018,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-array",
  "cclab-frame",
@@ -2030,7 +2030,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sci-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-sci",
  "pyo3",
@@ -2038,7 +2038,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "async-trait",
@@ -2086,7 +2086,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-sdd-cli"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "cclab-cli-registry",
@@ -2112,7 +2112,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-server"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "async-stream",
@@ -2137,7 +2137,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "rayon",
  "serde",
@@ -2148,7 +2148,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-text-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-text",
  "pyo3",
@@ -2156,7 +2156,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-tqdm"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "indicatif",
@@ -2166,7 +2166,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-tqdm-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-tqdm",
  "parking_lot",
@@ -2175,7 +2175,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-typer"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "anyhow",
  "clap",
@@ -2185,7 +2185,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-typer-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-typer",
  "parking_lot",
@@ -2194,11 +2194,11 @@ dependencies = [
 
 [[package]]
 name = "cclab-util"
-version = "0.3.43"
+version = "0.3.41"
 
 [[package]]
 name = "cclab-util-pyo3"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "cclab-util",
  "pyo3",
@@ -2206,7 +2206,7 @@ dependencies = [
 
 [[package]]
 name = "cclab-vortex"
-version = "0.3.43"
+version = "0.3.41"
 dependencies = [
  "bytemuck",
  "env_logger",
diff --git a/Cargo.toml b/Cargo.toml
index ebfb3573..40ab040e 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -71,7 +71,7 @@ members = [
 resolver = "2"
 
 [workspace.package]
-version = "0.3.43"
+version = "0.3.41"
 authors = ["Chris Cheng <chris.cheng.c4@gmail.com>"]
 edition = "2021"
 license = "MIT"
diff --git a/crates/cclab-jet/src/bundler/mangle.rs b/crates/cclab-jet/src/bundler/mangle.rs
index 104aec81..37571783 100644
--- a/crates/cclab-jet/src/bundler/mangle.rs
+++ b/crates/cclab-jet/src/bundler/mangle.rs
@@ -529,8 +529,13 @@ const RESERVED: &[&str] = &[
     "protected", "public", "static", "yield", "async", "await", "of",
     // Literals
     "null", "undefined", "true", "false", "NaN", "Infinity",
-    // Module system
-    "require", "module", "exports", "arguments",
+    // Module system — `require` kept reserved as it appears as a global in
+    // many environments; `module` and `exports` are intentionally NOT
+    // reserved so the mangler can rename them when they appear as function
+    // parameters inside per-module IIFE wrappers (e.g., the scope-hoisted
+    // `!function(module,exports,require){...}()` format), turning the
+    // 7-byte `exports` parameter into a 1-byte short name.
+    "require", "arguments",
     // Common globals (safety net)
     "window", "document", "console", "navigator", "location", "history",
     "setTimeout", "setInterval", "clearTimeout", "clearInterval",
@@ -638,6 +643,42 @@ mod tests {
         assert!(!out.contains("longVarName"), "wrapper var should be mangled, got: {}", out);
         assert!(!out.contains("innerLong"), "inner var should be mangled, got: {}", out);
         assert!(out.contains("require"), "require preserved, got: {}", out);
+        // `module` and `exports` are no longer reserved — they should be mangled
+        // to short names when used as function parameters.
+        // The function signature must not contain the original long names as params.
+        assert!(!out.contains(",module,"), "module param should be mangled, got: {}", out);
+        // `exports` as a param appears as `,exports)` in the original; it should be renamed.
+        assert!(!out.contains(",exports)"), "exports param should be mangled, got: {}", out);
+    }
+
+    #[test]
+    fn test_scope_hoisted_module_mangling() {
+        // Simulate a scope-hoisted Phase-1 module IIFE: exports/module are params
+        // and should be mangled to short names.
+        let src = "!function(module,exports,require){var workInProgress=null;exports.render=workInProgress;}(_m0,_m0.exports,_r)";
+        let out = mangle_variables(src);
+        eprintln!("scope-hoisted output: {}", out);
+        // workInProgress is a local var — must be mangled
+        assert!(!out.contains("workInProgress"), "workInProgress should be mangled, got: {}", out);
+        // `exports` as a function parameter must be renamed — the signature
+        // `(module,exports,require)` must be gone. Note that `_m0.exports`
+        // (property access) is NOT renamed, so we check the param list, not
+        // the whole output.
+        assert!(
+            !out.contains(",exports,") && !out.contains("(exports,") && !out.contains(",exports)"),
+            "exports param should be mangled (only .exports property access remains), got: {}",
+            out
+        );
+        // module is a parameter — must be mangled (no longer appears as standalone `module,`)
+        assert!(
+            !out.contains("(module,") && !out.contains(",module,") && !out.contains(",module)"),
+            "module param should be mangled, got: {}",
+            out
+        );
+        // require is still reserved — must NOT be mangled
+        assert!(out.contains("require"), "require should be preserved, got: {}", out);
+        // The property access `.exports` (in the argument `_m0.exports`) must be preserved
+        assert!(out.contains(".exports"), ".exports property access preserved, got: {}", out);
     }
 
     #[test]
@@ -662,4 +703,69 @@ mod tests {
         assert!(!names.contains(&"do".to_string()), "should not generate keyword 'do'");
         assert!(!names.contains(&"if".to_string()), "should not generate keyword 'if'");
     }
+
+    // ──────────────────────────────────────────────────────────────────
+    // UTF-8 multi-byte safety tests (issue #904)
+    //
+    // The tokenizer operates on raw bytes (`source.as_bytes()`) and only
+    // matches ASCII identifier characters, so multi-byte UTF-8 sequences
+    // pass through as opaque Punct tokens.  `apply_renames` reconstructs
+    // the output via `result.splice(byte_start..byte_end, ...)` where
+    // offsets come from the byte-level scan — no char-index-as-byte-offset
+    // bug is possible.  These tests verify that.
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_utf8_string_content_unchanged() {
+        // ✓ is 3-byte UTF-8; the string literal must survive intact.
+        // Use a multi-char name so the mangler actually renames it.
+        let src = r#"function f() { var checkResult = "✓ ok"; return checkResult; }"#;
+        let out = mangle_variables(src);
+        assert!(
+            out.contains("\"✓ ok\""),
+            "UTF-8 string must be unchanged, got: {}",
+            out
+        );
+        // Local `checkResult` should be mangled to a shorter name
+        assert!(!out.contains("checkResult"), "local checkResult should be mangled, got: {}", out);
+    }
+
+    #[test]
+    fn test_utf8_emoji_in_string_preserved() {
+        // 🎉 is 4-byte UTF-8
+        let src = r#"function f() { var msg = "Hello 🎉"; return msg; }"#;
+        let out = mangle_variables(src);
+        assert!(
+            out.contains("\"Hello 🎉\""),
+            "emoji string must be preserved, got: {}",
+            out
+        );
+    }
+
+    #[test]
+    fn test_utf8_cjk_in_string_preserved() {
+        // Use a multi-char name so the mangler actually renames it.
+        let src = "function f() { var strVal = '日本語テスト'; return strVal; }";
+        let out = mangle_variables(src);
+        assert!(
+            out.contains("'日本語テスト'"),
+            "CJK string must be preserved, got: {}",
+            out
+        );
+        // `strVal` should be mangled to a shorter name
+        assert!(!out.contains("strVal"), "local strVal should be mangled, got: {}", out);
+    }
+
+    #[test]
+    fn test_utf8_mixed_code_and_strings() {
+        // Multi-byte chars before and after identifiers that get renamed
+        let src = "function f() { var longName = '✓'; var other = longName + '日'; return other; }";
+        let out = mangle_variables(src);
+        // Strings preserved
+        assert!(out.contains("'✓'"), "✓ string preserved, got: {}", out);
+        assert!(out.contains("'日'"), "日 string preserved, got: {}", out);
+        // Identifiers mangled
+        assert!(!out.contains("longName"), "longName should be mangled, got: {}", out);
+        assert!(!out.contains("other"), "other should be mangled, got: {}", out);
+    }
 }
diff --git a/crates/cclab-jet/src/bundler/minify.rs b/crates/cclab-jet/src/bundler/minify.rs
index 240014e5..ced06c8a 100644
--- a/crates/cclab-jet/src/bundler/minify.rs
+++ b/crates/cclab-jet/src/bundler/minify.rs
@@ -634,4 +634,77 @@ var x = 1;"#;
         assert!(!result.contains('\n'));
         assert!(result.contains("color:red"));
     }
+
+    // ──────────────────────────────────────────────────────────────────
+    // UTF-8 multi-byte safety tests (issue #904)
+    //
+    // The minifier iterates with `chars().collect::<Vec<char>>()` and
+    // pushes chars directly to the result — it never slices `source` by
+    // char index.  These tests confirm that multi-byte characters are
+    // handled correctly end-to-end.
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_minify_utf8_multibyte_in_string() {
+        // ✓ is 3 bytes (E2 9C 93); should survive unchanged inside string
+        let source = r#"var x = "✓ passed";"#;
+        let result = minify_js(source, &[]);
+        assert!(
+            result.contains("\"✓ passed\""),
+            "UTF-8 string preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_minify_utf8_emoji_in_string() {
+        // 🎉 is 4 bytes (F0 9F 8E 89)
+        let source = r#"console.log("Hello 🎉");  const x = 1;"#;
+        let result = minify_js(source, &[]);
+        assert!(
+            result.contains("\"Hello 🎉\""),
+            "emoji in string preserved, got: {}",
+            result
+        );
+        assert!(
+            result.contains("const x"),
+            "code after emoji string intact, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_minify_utf8_cjk_in_string() {
+        // CJK characters: 日本語 (3 bytes each)
+        let source = "var label = '日本語テスト';  var x = 1;";
+        let result = minify_js(source, &[]);
+        assert!(
+            result.contains("'日本語テスト'"),
+            "CJK string preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_strip_comments_utf8_preserved() {
+        let source = "// comment\nvar x = '日本語'; /* block */ var y = 1;";
+        let result = strip_comments(source);
+        assert!(!result.contains("comment"), "comment stripped");
+        assert!(!result.contains("block"), "block comment stripped");
+        assert!(
+            result.contains("'日本語'"),
+            "CJK string preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_minify_utf8_outside_string() {
+        // UTF-8 identifier characters (JS allows them, though uncommon in practice).
+        // The minifier should pass them through without panic.
+        let source = "var café = 1; /* strip */ var x = café;";
+        let result = minify_js(source, &[]);
+        // Should not panic, and the identifier should survive
+        assert!(result.contains("café"), "UTF-8 identifier preserved, got: {}", result);
+    }
 }
diff --git a/crates/cclab-jet/src/bundler/mod.rs b/crates/cclab-jet/src/bundler/mod.rs
index 972dcafc..9ad7575b 100644
--- a/crates/cclab-jet/src/bundler/mod.rs
+++ b/crates/cclab-jet/src/bundler/mod.rs
@@ -477,9 +477,21 @@ impl Bundler {
         // Use scope hoisting when there are no dynamic imports.
         // This eliminates the full module runtime and gives minifiers
         // cross-module visibility for better DCE and constant folding.
+        //
+        // Phase 2 (true flattening): `generate_flattened_bundle` inlines
+        // all module bodies into a single flat scope.  It renames each
+        // module's top-level variables with a collision-avoiding `_m{n}_`
+        // prefix and replaces CJS globals (`exports` → `_m{n}e`,
+        // `module` → `_m{n}`, `require` → `_r`).  The whole-bundle
+        // `mangle_variables` pass then compresses all of these prefixed
+        // names to 1-2 byte identifiers in a single unified scope, giving
+        // tighter minification than per-module IIFE renaming.
+        //
+        // Falls back to Phase 1 (per-module IIFE wrappers) internally when
+        // `is_flatten_safe` returns false (eval/with/arguments[ detected).
         let bundle = if scope_hoist::is_scope_hoist_safe(&modules) {
-            tracing::debug!("Using scope hoisting (no dynamic imports)");
-            scope_hoist::generate_scope_hoisted_bundle(&modules)
+            tracing::debug!("Using Phase 2 flat scope hoisting (no dynamic imports)");
+            scope_hoist::generate_flattened_bundle(&modules)
         } else {
             tracing::debug!("Falling back to runtime module system (dynamic imports present)");
             generate_bundle_with_runtime(&modules)
diff --git a/crates/cclab-jet/src/bundler/scope_hoist.rs b/crates/cclab-jet/src/bundler/scope_hoist.rs
index e6d467e9..12a7978e 100644
--- a/crates/cclab-jet/src/bundler/scope_hoist.rs
+++ b/crates/cclab-jet/src/bundler/scope_hoist.rs
@@ -2,7 +2,7 @@
 //!
 //! Instead of the `__jet__.define` / `__jet__.require` module registry,
 //! all modules are inlined into a single IIFE with a lightweight
-//! `__jet_require` function. This gives minifiers full cross-module
+//! `_r` function. This gives minifiers full cross-module
 //! visibility for dead-code elimination and constant folding, which
 //! reduces bundle size to match Webpack / Vite output.
 //!
@@ -16,15 +16,15 @@
 //! The scope-hoisted format flattens all modules into one IIFE:
 //! ```js
 //! (function() {
-//!   var __jet_m0 = {exports: {}};
+//!   var _m0 = {exports: {}};
 //!   // ...
-//!   function __jet_require(id) { ... }
+//!   function _r(id) { ... }
 //!
 //!   // Execute in dependency order (leaf modules first)
 //!   (function(module, exports, require) { /* dep */ })
-//!     (__jet_m1, __jet_m1.exports, __jet_require);
+//!     (_m1, _m1.exports, _r);
 //!   (function(module, exports, require) { /* entry */ })
-//!     (__jet_m0, __jet_m0.exports, __jet_require);
+//!     (_m0, _m0.exports, _r);
 //! })();
 //! ```
 //!
@@ -34,6 +34,8 @@
 //! - Single scope → minifier renames all local vars in one pass
 //! - Cross-module DCE and constant folding become possible
 
+use std::collections::HashMap;
+
 use super::CompiledModule;
 
 /// Generate a scope-hoisted bundle from compiled modules.
@@ -58,7 +60,7 @@ pub fn generate_scope_hoisted_bundle(modules: &[CompiledModule]) -> String {
     // Using `var` means they are hoisted to the function scope and
     // visible everywhere inside the IIFE.
     for i in 0..n {
-        out.push_str(&format!("var __jet_m{}={{exports:{{}}}};\n", i));
+        out.push_str(&format!("var _m{}={{exports:{{}}}};\n", i));
     }
     out.push('\n');
 
@@ -66,10 +68,10 @@ pub fn generate_scope_hoisted_bundle(modules: &[CompiledModule]) -> String {
     // This is the only runtime overhead that cannot be eliminated by
     // hoisting. A minifier (Terser/esbuild) can inline this for
     // single-call-site modules.
-    out.push_str("function __jet_require(id){\n");
+    out.push_str("function _r(id){\n");
     out.push_str("  switch(id){\n");
     for i in 0..n {
-        out.push_str(&format!("    case {}:return __jet_m{};\n", i, i));
+        out.push_str(&format!("    case {}:return _m{};\n", i, i));
     }
     out.push_str("    default:return {exports:{}};\n");
     out.push_str("  }\n");
@@ -90,7 +92,7 @@ pub fn generate_scope_hoisted_bundle(modules: &[CompiledModule]) -> String {
         // module-level references and apply cross-module DCE.
         out.push_str(&format!(
             "!function(module,exports,require){{\n{}}}(\
-             __jet_m{},__jet_m{}.exports,__jet_require);\n\n",
+             _m{},_m{}.exports,_r);\n\n",
             module.code, original_idx, original_idx
         ));
     }
@@ -120,6 +122,615 @@ pub fn is_scope_hoist_safe(modules: &[CompiledModule]) -> bool {
     true
 }
 
+/// Returns `true` when no module uses `eval()`, `with` statements, or dynamic
+/// `arguments[...]` access, which would make it unsafe to inline the module
+/// body into a shared scope.
+///
+/// - `eval()` can reference ambient variables by name at runtime.
+/// - `with(obj)` creates dynamic scope that cannot be statically resolved.
+/// - `arguments[dynamic_index]` relies on the current function's `arguments`
+///   object being stable, which renaming could violate.
+pub fn is_flatten_safe(modules: &[CompiledModule]) -> bool {
+    for module in modules {
+        if module.code.contains("eval(")
+            || module.code.contains("with(")
+            || module.code.contains("arguments[")
+        {
+            return false;
+        }
+    }
+    true
+}
+
+/// Phase 2: Generate a truly flat bundle by inlining each module body
+/// directly into the outer IIFE without per-module wrapper functions.
+///
+/// Unlike Phase 1 (`generate_scope_hoisted_bundle`), this approach
+/// replaces the `!function(module,exports,require){...}()` wrapper with
+/// a plain block `{ ... }` after substituting the CJS parameter names:
+///
+/// - `module`  → `_m{i}`   (the module namespace object)
+/// - `exports` → `_m{i}.exports`
+/// - `require` → `_r`
+///
+/// Benefits over Phase 1:
+/// - Minifier sees all variables in a single scope → better name mangling.
+/// - No IIFE call overhead per module.
+/// - Cross-module constant folding and DCE are more effective.
+///
+/// Falls back to Phase 1 if `is_flatten_safe` returns `false`.
+pub fn generate_flattened_bundle(modules: &[CompiledModule]) -> String {
+    if modules.is_empty() {
+        return String::new();
+    }
+
+    // Safety check: fall back to Phase 1 if any module uses eval/with
+    if !is_flatten_safe(modules) {
+        tracing::debug!("Falling back to Phase 1 scope hoisting (eval/with detected)");
+        return generate_scope_hoisted_bundle(modules);
+    }
+
+    let n = modules.len();
+    let mut out = String::with_capacity(estimate_output_size(modules));
+
+    out.push_str("(function(){\n'use strict';\n\n");
+
+    // Pre-declare all module namespace objects using short names.
+    for i in 0..n {
+        out.push_str(&format!("var _m{}={{exports:{{}}}};\n", i));
+    }
+    out.push('\n');
+
+    // Lightweight require function — still needed for patterns like
+    // `var dep = require(1)` that reference modules by numeric ID.
+    out.push_str("function _r(id){\n");
+    out.push_str("  switch(id){\n");
+    for i in 0..n {
+        out.push_str(&format!("    case {}:return _m{};\n", i, i));
+    }
+    out.push_str("    default:return {exports:{}};\n");
+    out.push_str("  }\n");
+    out.push_str("}\n\n");
+
+    // Inline each module body in reverse topological order (deepest deps first).
+    for (original_idx, module) in modules.iter().enumerate().rev() {
+        let module_path = module.path.to_string_lossy();
+        out.push_str(&format!(
+            "// Module {}: {}\n",
+            original_idx, module_path
+        ));
+        // Apply per-module prefix renaming (R3) + CJS substitutions (R2).
+        // `exports` is renamed to `_m{idx}e` throughout the body, and a `var`
+        // alias is declared so that the whole-bundle mangler can compress it to
+        // a single byte (rather than leaving the 7-byte `exports` unrenamed).
+        let inlined = inline_module_body_v2(&module.code, original_idx);
+        out.push_str("{\n");
+        // Declare the exports alias: var hoists to the outer IIFE function scope,
+        // making it visible to the mangler in a single unified scope.
+        out.push_str(&format!(
+            "var _m{idx}e=_m{idx}.exports;\n",
+            idx = original_idx
+        ));
+        out.push_str(&inlined);
+        out.push_str("\n}\n\n");
+    }
+
+    out.push_str("})();\n");
+    out
+}
+
+/// Substitute CJS module parameter names in a compiled module body.
+///
+/// Replaces standalone identifiers (not preceded by `.`, not inside
+/// strings or comments) as follows:
+///
+/// - `module`  → `_m{idx}`
+/// - `exports` → `_m{idx}.exports`
+/// - `require` → `_r`
+///
+/// Uses byte-level scanning to safely handle multi-byte UTF-8 content.
+fn inline_module_body(code: &str, idx: usize) -> String {
+    let module_repl = format!("_m{}", idx);
+    let exports_repl = format!("_m{}.exports", idx);
+    let require_repl = "_r";
+
+    let b = code.as_bytes();
+    let len = b.len();
+    let mut out = Vec::with_capacity(len + 64);
+    let mut i = 0;
+
+    while i < len {
+        // Skip string literals (single, double, template)
+        if matches!(b[i], b'"' | b'\'' | b'`') {
+            let q = b[i];
+            out.push(b[i]);
+            i += 1;
+            while i < len {
+                if b[i] == b'\\' {
+                    out.push(b[i]);
+                    i += 1;
+                    if i < len {
+                        out.push(b[i]);
+                        i += 1;
+                    }
+                    continue;
+                }
+                out.push(b[i]);
+                if b[i] == q {
+                    i += 1;
+                    break;
+                }
+                i += 1;
+            }
+            continue;
+        }
+
+        // Skip comments (single-line and block)
+        if b[i] == b'/' && i + 1 < len {
+            if b[i + 1] == b'/' {
+                // Single-line comment: copy until newline
+                while i < len && b[i] != b'\n' {
+                    out.push(b[i]);
+                    i += 1;
+                }
+                continue;
+            }
+            if b[i + 1] == b'*' {
+                // Block comment: copy until */
+                out.push(b[i]);
+                i += 1;
+                out.push(b[i]);
+                i += 1;
+                while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
+                    out.push(b[i]);
+                    i += 1;
+                }
+                if i + 1 < len {
+                    out.push(b[i]);
+                    i += 1; // *
+                    out.push(b[i]);
+                    i += 1; // /
+                }
+                continue;
+            }
+        }
+
+        // Try to match an identifier at a word boundary.
+        // Only substitute if NOT preceded by `.` (avoids obj.module, obj.exports).
+        if is_id_start_byte(b[i]) {
+            let prev_non_ws_is_dot = {
+                let mut p = out.len();
+                while p > 0 && out[p - 1] == b' ' {
+                    p -= 1;
+                }
+                p > 0 && out[p - 1] == b'.'
+            };
+
+            // Check each keyword: verify full word boundary (not part of longer ident)
+            if !prev_non_ws_is_dot {
+                // `module` (6 bytes)
+                if i + 6 <= len
+                    && &b[i..i + 6] == b"module"
+                    && (i + 6 >= len || !is_id_cont_byte(b[i + 6]))
+                {
+                    out.extend_from_slice(module_repl.as_bytes());
+                    i += 6;
+                    continue;
+                }
+                // `exports` (7 bytes)
+                if i + 7 <= len
+                    && &b[i..i + 7] == b"exports"
+                    && (i + 7 >= len || !is_id_cont_byte(b[i + 7]))
+                {
+                    out.extend_from_slice(exports_repl.as_bytes());
+                    i += 7;
+                    continue;
+                }
+                // `require` (7 bytes)
+                if i + 7 <= len
+                    && &b[i..i + 7] == b"require"
+                    && (i + 7 >= len || !is_id_cont_byte(b[i + 7]))
+                {
+                    out.extend_from_slice(require_repl.as_bytes());
+                    i += 7;
+                    continue;
+                }
+            }
+        }
+
+        out.push(b[i]);
+        i += 1;
+    }
+
+    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
+}
+
+/// Returns `true` if the byte is a valid JS identifier start (ASCII only).
+/// Non-ASCII bytes from multi-byte UTF-8 sequences are never matched,
+/// so they pass through unchanged.
+#[inline]
+fn is_id_start_byte(c: u8) -> bool {
+    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
+}
+
+/// Returns `true` if the byte is a valid JS identifier continuation (ASCII only).
+#[inline]
+fn is_id_cont_byte(c: u8) -> bool {
+    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
+}
+
+// ──────────────────────────────────────────────────────────────────────────
+// Phase 2 helpers: per-module variable prefix renaming (R2 / R3)
+// ──────────────────────────────────────────────────────────────────────────
+
+/// Returns `true` if `name` is a JS keyword or declaration keyword that should
+/// not be prefixed when scanning top-level declarations.
+fn is_js_decl_keyword(name: &str) -> bool {
+    matches!(
+        name,
+        "var" | "let" | "const" | "function" | "class" | "async" | "await"
+            | "if" | "else" | "for" | "while" | "do" | "return" | "new"
+            | "delete" | "typeof" | "void" | "throw" | "try" | "catch"
+            | "finally" | "switch" | "case" | "break" | "continue"
+            | "import" | "export" | "default" | "in" | "of" | "instanceof"
+            | "yield" | "with" | "debugger" | "this" | "super" | "extends"
+            | "static" | "get" | "set" | "null" | "undefined" | "true"
+            | "false" | "NaN" | "Infinity"
+    )
+}
+
+/// Scan a comma-separated `var`/`let`/`const` declaration list starting at `*i`
+/// and push each simple identifier name into `names`.
+/// Advances `*i` past the terminating `;` (or until end-of-input).
+fn collect_decl_names_from(code: &str, i: &mut usize, names: &mut Vec<String>) {
+    let b = code.as_bytes();
+    let len = b.len();
+    let mut depth = 0i32;
+    let mut expect_name = true;
+
+    while *i < len {
+        // Skip string literals
+        if matches!(b[*i], b'"' | b'\'' | b'`') {
+            let q = b[*i];
+            *i += 1;
+            while *i < len {
+                if b[*i] == b'\\' {
+                    *i += 2;
+                    continue;
+                }
+                if b[*i] == q {
+                    *i += 1;
+                    break;
+                }
+                *i += 1;
+            }
+            continue;
+        }
+        match b[*i] {
+            b'{' | b'(' | b'[' => {
+                depth += 1;
+                expect_name = false;
+                *i += 1;
+            }
+            b'}' | b')' | b']' => {
+                depth -= 1;
+                *i += 1;
+            }
+            b';' if depth == 0 => {
+                *i += 1;
+                break;
+            }
+            b',' if depth == 0 => {
+                expect_name = true;
+                *i += 1;
+            }
+            _ if expect_name && is_id_start_byte(b[*i]) => {
+                let ns = *i;
+                while *i < len && is_id_cont_byte(b[*i]) {
+                    *i += 1;
+                }
+                let name = &code[ns..*i];
+                if !name.is_empty() && !is_js_decl_keyword(name) {
+                    names.push(name.to_string());
+                }
+                expect_name = false;
+            }
+            _ => {
+                *i += 1;
+            }
+        }
+    }
+}
+
+/// Collect all top-level `var`/`let`/`const`/`function`/`async function`/
+/// `class` declaration names from a module body.
+///
+/// Only names at brace depth 0 are collected; declarations inside nested
+/// functions or blocks are ignored.  CJS globals (`exports`, `module`,
+/// `require`) are excluded since they are handled separately.
+fn collect_top_level_decls(code: &str) -> Vec<String> {
+    let b = code.as_bytes();
+    let len = b.len();
+    let mut names: Vec<String> = Vec::new();
+    let mut i = 0;
+    let mut depth = 0i32;
+
+    while i < len {
+        // Skip string literals
+        if matches!(b[i], b'"' | b'\'' | b'`') {
+            let q = b[i];
+            i += 1;
+            while i < len {
+                if b[i] == b'\\' {
+                    i += 2;
+                    continue;
+                }
+                if b[i] == q {
+                    i += 1;
+                    break;
+                }
+                i += 1;
+            }
+            continue;
+        }
+        // Skip single-line comments
+        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'/' {
+            while i < len && b[i] != b'\n' {
+                i += 1;
+            }
+            continue;
+        }
+        // Skip block comments
+        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'*' {
+            i += 2;
+            while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
+                i += 1;
+            }
+            i += 2;
+            continue;
+        }
+        // Track depth via all bracket types
+        match b[i] {
+            b'{' | b'(' | b'[' => {
+                depth += 1;
+                i += 1;
+                continue;
+            }
+            b'}' | b')' | b']' => {
+                if depth > 0 {
+                    depth -= 1;
+                }
+                i += 1;
+                continue;
+            }
+            _ => {}
+        }
+        // Only collect declarations at top-level depth
+        if depth == 0 && is_id_start_byte(b[i]) {
+            let start = i;
+            while i < len && is_id_cont_byte(b[i]) {
+                i += 1;
+            }
+            let word = &code[start..i];
+
+            // Skip leading whitespace before the next token
+            let mut j = i;
+            while j < len && matches!(b[j], b' ' | b'\t' | b'\n' | b'\r') {
+                j += 1;
+            }
+
+            match word {
+                "var" | "let" | "const" => {
+                    i = j;
+                    collect_decl_names_from(code, &mut i, &mut names);
+                }
+                "function" => {
+                    i = j;
+                    // Skip generator `*`
+                    if i < len && b[i] == b'*' {
+                        i += 1;
+                        while i < len && b[i] == b' ' {
+                            i += 1;
+                        }
+                    }
+                    if i < len && is_id_start_byte(b[i]) {
+                        let ns = i;
+                        while i < len && is_id_cont_byte(b[i]) {
+                            i += 1;
+                        }
+                        let name = &code[ns..i];
+                        if !name.is_empty() && !is_js_decl_keyword(name) {
+                            names.push(name.to_string());
+                        }
+                    }
+                }
+                "async" => {
+                    i = j;
+                    // `async function name() {}`
+                    if i + 8 <= len
+                        && &code[i..i + 8] == "function"
+                        && (i + 8 >= len || !is_id_cont_byte(b[i + 8]))
+                    {
+                        i += 8;
+                        while i < len && matches!(b[i], b' ' | b'\t') {
+                            i += 1;
+                        }
+                        if i < len && b[i] == b'*' {
+                            i += 1;
+                            while i < len && b[i] == b' ' {
+                                i += 1;
+                            }
+                        }
+                        if i < len && is_id_start_byte(b[i]) {
+                            let ns = i;
+                            while i < len && is_id_cont_byte(b[i]) {
+                                i += 1;
+                            }
+                            let name = &code[ns..i];
+                            if !name.is_empty() && !is_js_decl_keyword(name) {
+                                names.push(name.to_string());
+                            }
+                        }
+                    }
+                }
+                "class" => {
+                    i = j;
+                    if i < len && is_id_start_byte(b[i]) {
+                        let ns = i;
+                        while i < len && is_id_cont_byte(b[i]) {
+                            i += 1;
+                        }
+                        let name = &code[ns..i];
+                        if !name.is_empty() && !is_js_decl_keyword(name) {
+                            names.push(name.to_string());
+                        }
+                    }
+                }
+                _ => {
+                    i = j;
+                }
+            }
+            continue;
+        }
+        i += 1;
+    }
+
+    names
+}
+
+/// Apply a rename map to a module body in a single byte-level pass.
+///
+/// Identifiers preceded by `.` (property accesses) are never renamed.
+/// String literals and comments are copied verbatim without substitution.
+fn apply_renames_in_module_body(code: &str, renames: &HashMap<String, String>) -> String {
+    let b = code.as_bytes();
+    let len = b.len();
+    let mut out = Vec::with_capacity(len + renames.len() * 4);
+    let mut i = 0;
+
+    while i < len {
+        // Skip string literals
+        if matches!(b[i], b'"' | b'\'' | b'`') {
+            let q = b[i];
+            out.push(b[i]);
+            i += 1;
+            while i < len {
+                if b[i] == b'\\' {
+                    out.push(b[i]);
+                    i += 1;
+                    if i < len {
+                        out.push(b[i]);
+                        i += 1;
+                    }
+                    continue;
+                }
+                out.push(b[i]);
+                if b[i] == q {
+                    i += 1;
+                    break;
+                }
+                i += 1;
+            }
+            continue;
+        }
+        // Skip single-line comments
+        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'/' {
+            while i < len && b[i] != b'\n' {
+                out.push(b[i]);
+                i += 1;
+            }
+            continue;
+        }
+        // Skip block comments
+        if b[i] == b'/' && i + 1 < len && b[i + 1] == b'*' {
+            out.push(b[i]);
+            i += 1;
+            out.push(b[i]);
+            i += 1;
+            while i + 1 < len && !(b[i] == b'*' && b[i + 1] == b'/') {
+                out.push(b[i]);
+                i += 1;
+            }
+            if i + 1 < len {
+                out.push(b[i]);
+                i += 1;
+                out.push(b[i]);
+                i += 1;
+            }
+            continue;
+        }
+        // Identifier: check for rename
+        if is_id_start_byte(b[i]) {
+            // Check if immediately preceded by '.' (property access — skip)
+            let prev_is_dot = {
+                let mut p = out.len();
+                while p > 0 && matches!(out[p - 1], b' ' | b'\t' | b'\r' | b'\n') {
+                    p -= 1;
+                }
+                p > 0 && out[p - 1] == b'.'
+            };
+            let start = i;
+            while i < len && is_id_cont_byte(b[i]) {
+                i += 1;
+            }
+            let ident = &code[start..i];
+            if !prev_is_dot {
+                if let Some(new_name) = renames.get(ident) {
+                    out.extend_from_slice(new_name.as_bytes());
+                    continue;
+                }
+            }
+            out.extend_from_slice(ident.as_bytes());
+            continue;
+        }
+        out.push(b[i]);
+        i += 1;
+    }
+
+    String::from_utf8(out).unwrap_or_else(|_| code.to_string())
+}
+
+/// Extended module body inlining (Phase 2 / R2 + R3).
+///
+/// Builds a combined rename map that:
+/// 1. Substitutes CJS globals: `exports` → `_m{idx}e`, `module` → `_m{idx}`,
+///    `require` → `_r`.
+/// 2. Prefixes every top-level `var`/`let`/`const`/`function`/`class`
+///    declaration with `_m{idx}_` so that when multiple modules are inlined
+///    into a single flat scope, their `var` declarations (which hoist to the
+///    outer IIFE function) do not collide.
+///
+/// The prefix names (`_m0_foo`, `_m1_bar`, …) are then compressed by the
+/// whole-bundle `mangle_variables` pass into single-byte identifiers.
+fn inline_module_body_v2(code: &str, idx: usize) -> String {
+    let module_repl = format!("_m{}", idx);
+    let exports_alias = format!("_m{}e", idx);
+
+    // Collect top-level declarations that need collision-avoiding prefixes.
+    let decls = collect_top_level_decls(code);
+
+    // Build the unified rename map.
+    let mut renames: HashMap<String, String> =
+        HashMap::with_capacity(decls.len() + 3);
+
+    // CJS globals come first so the loop below can skip them if they appear
+    // as local vars (very unlikely but safe).
+    renames.insert("exports".to_string(), exports_alias);
+    renames.insert("module".to_string(), module_repl.clone());
+    renames.insert("require".to_string(), "_r".to_string());
+
+    // Per-module prefix for top-level declarations.
+    for decl in decls {
+        // Don't overwrite CJS globals (exports/module/require) with a prefixed
+        // version — the CJS substitution above takes priority.
+        renames.entry(decl.clone()).or_insert_with(|| {
+            format!("_m{}_{}", idx, decl)
+        });
+    }
+
+    apply_renames_in_module_body(code, &renames)
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -151,9 +762,9 @@ mod tests {
         // Outer IIFE
         assert!(bundle.contains("(function()"));
         // Module namespace
-        assert!(bundle.contains("var __jet_m0="));
+        assert!(bundle.contains("var _m0="));
         // require function
-        assert!(bundle.contains("__jet_require"));
+        assert!(bundle.contains("_r"));
         // Module code wrapped in its own function
         assert!(bundle.contains("exports.main = function()"));
         // Closure
@@ -172,12 +783,12 @@ mod tests {
         let bundle = generate_scope_hoisted_bundle(&modules);
 
         // Both module vars declared
-        assert!(bundle.contains("var __jet_m0="));
-        assert!(bundle.contains("var __jet_m1="));
+        assert!(bundle.contains("var _m0="));
+        assert!(bundle.contains("var _m1="));
 
         // require switch has both cases
-        assert!(bundle.contains("case 0:return __jet_m0;"));
-        assert!(bundle.contains("case 1:return __jet_m1;"));
+        assert!(bundle.contains("case 0:return _m0;"));
+        assert!(bundle.contains("case 1:return _m1;"));
 
         // dep module (index 1) should appear BEFORE entry (index 0)
         // because we iterate in reverse
@@ -206,4 +817,368 @@ mod tests {
         )];
         assert!(!is_scope_hoist_safe(&modules));
     }
+
+    // ──────────────────────────────────────────────────────────────────
+    // Phase 2 flatten tests
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_flatten_safe_no_eval() {
+        let modules = vec![
+            make_module("a.js", "exports.x = 1;"),
+            make_module("b.js", "var y = require(1).exports.x;"),
+        ];
+        assert!(is_flatten_safe(&modules));
+    }
+
+    #[test]
+    fn test_flatten_unsafe_with_eval() {
+        let modules = vec![make_module("a.js", "eval('code');")];
+        assert!(!is_flatten_safe(&modules));
+    }
+
+    #[test]
+    fn test_flatten_unsafe_with_with_stmt() {
+        let modules = vec![make_module("a.js", "with(obj) { foo(); }")];
+        assert!(!is_flatten_safe(&modules));
+    }
+
+    #[test]
+    fn test_inline_module_body_substitution() {
+        let code = "exports.foo = 1; module.exports.bar = 2; var x = require(1);";
+        let result = inline_module_body(code, 3);
+        // `exports` → `_m3.exports`
+        assert!(
+            result.contains("_m3.exports.foo = 1"),
+            "exports substituted, got: {}",
+            result
+        );
+        // `module.exports` → `_m3.exports` (module replaced, .exports stays)
+        assert!(
+            result.contains("_m3.exports.bar = 2"),
+            "module.exports substituted, got: {}",
+            result
+        );
+        // `require` → `_r`
+        assert!(
+            result.contains("_r(1)"),
+            "require substituted, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_inline_module_body_preserves_strings() {
+        let code = r#"var s = "module exports require"; exports.x = s;"#;
+        let result = inline_module_body(code, 0);
+        // Strings must NOT be substituted
+        assert!(
+            result.contains(r#""module exports require""#),
+            "string content must be preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_inline_module_body_preserves_property_access() {
+        // obj.module, obj.exports, obj.require should NOT be substituted
+        let code = "var x = obj.module; var y = obj.exports; var z = obj.require;";
+        let result = inline_module_body(code, 2);
+        assert!(
+            result.contains("obj.module"),
+            "obj.module should be preserved, got: {}",
+            result
+        );
+        assert!(
+            result.contains("obj.exports"),
+            "obj.exports should be preserved, got: {}",
+            result
+        );
+        assert!(
+            result.contains("obj.require"),
+            "obj.require should be preserved, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_inline_module_body_no_partial_matches() {
+        // `moduleId` should NOT be replaced as `module` + `Id`
+        let code = "var moduleId = 1; var requireCount = 2; exportsMap = {};";
+        let result = inline_module_body(code, 0);
+        assert!(
+            result.contains("moduleId"),
+            "moduleId should not be changed, got: {}",
+            result
+        );
+        assert!(
+            result.contains("requireCount"),
+            "requireCount should not be changed, got: {}",
+            result
+        );
+        assert!(
+            result.contains("exportsMap"),
+            "exportsMap should not be changed, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_generate_flattened_bundle_empty() {
+        assert!(generate_flattened_bundle(&[]).is_empty());
+    }
+
+    #[test]
+    fn test_generate_flattened_bundle_single_module() {
+        let modules = vec![make_module("entry.js", "exports.main = 42;")];
+        let bundle = generate_flattened_bundle(&modules);
+        assert!(bundle.contains("(function()"), "outer IIFE present");
+        assert!(bundle.contains("var _m0="), "module var declared");
+        // Phase 2: `exports` is aliased to `_m0e` and the alias is declared,
+        // so `exports.main = 42` becomes `_m0e.main = 42`.
+        assert!(
+            bundle.contains("_m0e.main = 42"),
+            "exports aliased to _m0e, got: {}",
+            bundle
+        );
+        assert!(
+            bundle.contains("var _m0e=_m0.exports"),
+            "exports alias declaration present, got: {}",
+            bundle
+        );
+        // No per-module wrapper function
+        assert!(
+            !bundle.contains("!function(module,exports,require)"),
+            "no per-module wrapper, got: {}",
+            bundle
+        );
+    }
+
+    #[test]
+    fn test_generate_flattened_bundle_two_modules() {
+        let modules = vec![
+            make_module("entry.js", "var dep = require(1); dep.exports.hello();"),
+            make_module("dep.js", "exports.hello = function() {};"),
+        ];
+        let bundle = generate_flattened_bundle(&modules);
+        // Both module vars declared
+        assert!(bundle.contains("var _m0="), "m0 declared");
+        assert!(bundle.contains("var _m1="), "m1 declared");
+        // require → _r
+        assert!(bundle.contains("_r(1)"), "require substituted, got: {}", bundle);
+        // Phase 2: exports alias `_m1e` used in module 1 body
+        assert!(
+            bundle.contains("_m1e.hello"),
+            "dep exports aliased to _m1e, got: {}",
+            bundle
+        );
+        // Phase 2: top-level var `dep` in module 0 prefixed to `_m0_dep`
+        assert!(
+            bundle.contains("_m0_dep"),
+            "module 0 local var 'dep' prefixed, got: {}",
+            bundle
+        );
+    }
+
+    #[test]
+    fn test_generate_flattened_bundle_falls_back_on_eval() {
+        let modules = vec![make_module("a.js", "eval('code');")];
+        let flat = generate_flattened_bundle(&modules);
+        let phase1 = generate_scope_hoisted_bundle(&modules);
+        // Should fall back to Phase 1 (contains per-module wrapper)
+        assert_eq!(flat, phase1, "should fall back to Phase 1 on eval");
+    }
+
+    #[test]
+    fn test_inline_module_body_utf8_safe() {
+        // Multi-byte UTF-8 characters must pass through unchanged
+        let code = "exports.msg = '日本語テスト ✓'; require(1);";
+        let result = inline_module_body(code, 0);
+        assert!(
+            result.contains("'日本語テスト ✓'"),
+            "UTF-8 string preserved, got: {}",
+            result
+        );
+        assert!(
+            result.contains("_r(1)"),
+            "require substituted after UTF-8, got: {}",
+            result
+        );
+    }
+
+    // ──────────────────────────────────────────────────────────────────
+    // R5 bailout: is_flatten_safe with arguments[ check
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_flatten_unsafe_with_dynamic_arguments() {
+        let modules = vec![make_module("a.js", "function f() { return arguments[0]; }")];
+        assert!(
+            !is_flatten_safe(&modules),
+            "dynamic arguments[ access should trigger bailout"
+        );
+    }
+
+    #[test]
+    fn test_flatten_safe_arguments_length_ok() {
+        // `arguments.length` does NOT use `arguments[` — should still be safe
+        // to flatten if no eval/with present.
+        // Note: the current check is conservative (substring match), so
+        // `arguments.` access does not trigger the bailout.
+        let modules = vec![make_module("a.js", "exports.x = 1;")];
+        assert!(is_flatten_safe(&modules));
+    }
+
+    // ──────────────────────────────────────────────────────────────────
+    // R2 / R3: collect_top_level_decls
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_collect_top_level_simple_var() {
+        let names = collect_top_level_decls("var foo = 1; var bar = 2;");
+        assert!(names.contains(&"foo".to_string()), "foo should be collected, got: {:?}", names);
+        assert!(names.contains(&"bar".to_string()), "bar should be collected, got: {:?}", names);
+    }
+
+    #[test]
+    fn test_collect_top_level_multi_var() {
+        let names = collect_top_level_decls("var a = 1, b = 2, c = 3;");
+        assert!(names.contains(&"a".to_string()), "a: {:?}", names);
+        assert!(names.contains(&"b".to_string()), "b: {:?}", names);
+        assert!(names.contains(&"c".to_string()), "c: {:?}", names);
+    }
+
+    #[test]
+    fn test_collect_top_level_function_decl() {
+        let names = collect_top_level_decls("function renderRoot(fiber) { var inner = 1; }");
+        assert!(names.contains(&"renderRoot".to_string()), "renderRoot: {:?}", names);
+        // inner var must NOT be collected (it's inside a function body)
+        assert!(!names.contains(&"inner".to_string()), "inner should not be collected: {:?}", names);
+    }
+
+    #[test]
+    fn test_collect_top_level_skips_nested() {
+        let code = "var outer = 1; function f() { var inner = 2; }";
+        let names = collect_top_level_decls(code);
+        assert!(names.contains(&"outer".to_string()), "outer: {:?}", names);
+        assert!(!names.contains(&"inner".to_string()), "inner should be skipped: {:?}", names);
+    }
+
+    #[test]
+    fn test_collect_top_level_skips_cjs_globals() {
+        // exports/module/require appear as free vars in module body, not as decls.
+        let code = "exports.x = 1; module.exports = {}; var y = require(1);";
+        let names = collect_top_level_decls(code);
+        assert!(!names.contains(&"exports".to_string()), "exports not a decl: {:?}", names);
+        assert!(!names.contains(&"module".to_string()), "module not a decl: {:?}", names);
+        assert!(names.contains(&"y".to_string()), "y is a decl: {:?}", names);
+    }
+
+    // ──────────────────────────────────────────────────────────────────
+    // R2 / R3: inline_module_body_v2
+    // ──────────────────────────────────────────────────────────────────
+
+    #[test]
+    fn test_inline_v2_exports_aliased() {
+        let code = "exports.foo = 1;";
+        let result = inline_module_body_v2(code, 3);
+        // exports → _m3e
+        assert!(
+            result.contains("_m3e.foo = 1"),
+            "exports aliased to _m3e, got: {}",
+            result
+        );
+        assert!(!result.contains("exports"), "raw 'exports' removed, got: {}", result);
+    }
+
+    #[test]
+    fn test_inline_v2_module_substituted() {
+        let code = "module.exports = {foo: 1};";
+        let result = inline_module_body_v2(code, 2);
+        // module → _m2
+        assert!(
+            result.contains("_m2.exports = {foo: 1}"),
+            "module → _m2, got: {}",
+            result
+        );
+    }
+
+    #[test]
+    fn test_inline_v2_require_substituted() {
+        let code = "var x = require(1).exports.foo;";
+        let result = inline_module_body_v2(code, 0);
+        assert!(result.contains("_r(1)"), "require → _r, got: {}", result);
+    }
+

... truncated (6715 more lines)
```
