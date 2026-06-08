---
change: jet-nx-support
group: jet-nx-support
date: 2026-03-19
written_by: artifact_cli
review_verdict: FAIL
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| workspace | cclab-jet/pkg_manager | high | R1, R2 |
| cli | cclab-jet | high | R1, R2, R3 |
| mod | cclab-jet/pkg_manager | medium | R3 |
| mod | cclab-jet/bundler | medium | R3 |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: jet-nx-support

**Verdict**: FAIL

### Summary

All four referenced spec paths do not exist in cclab/specs/. The spec naming convention uses source module paths (cclab-jet/pkg_manager, cclab-jet/bundler) rather than actual spec file paths. The closest existing specs (pkg-manager-pnpm-parity.md, aot-build.md, jit-runner.md) partially overlap with this change's requirements, but with wrong relevance scores and inaccurate key requirement IDs. Two requirements — R2 (Nx project graph via nx graph --json) and R4 (Nx config: nx.json + project.json + [nx] config.toml section) — have zero coverage by any existing spec.

### Checklist

- ❌ All affected crates/areas from pre-clarifications are covered by at least one spec
  - Pre-clarifications define five concrete requirements: Q1 workspace detection (nx.json primary, node_modules/.bin/nx + @nx/workspace secondary), Q2 project graph via nx graph --json, Q3 task pipeline with Jet executor in project.json, Q4 jet build respecting Nx graph for dependent builds, Q5 [nx] section in cclab/config.toml. No existing spec covers Q2 (Nx project graph querying) or Q5 ([nx] config section). Q1 is partially served by pkg-manager-pnpm-parity.md (pnpm-style workspace discovery) but that spec targets package.json.workspaces and jet-workspace.yaml, not nx.json detection. Q3 and Q4 are partially served by jit-runner.md (task runner, jet.config.yaml pipeline) and aot-build.md (jet build pipeline), but neither covers the Nx executor protocol in project.json or Nx-graph-aware build ordering.
- ❌ Relevance scores are reasonable (high = directly implements, medium = related, low = background)
  - No verification was possible because all four listed spec paths don't exist. If the reference context intended to point to pkg-manager-pnpm-parity.md (for workspace), its relevance score of HIGH is wrong: that spec implements pnpm-style workspace discovery (package.json.workspaces, workspace:* protocol, catalog), not Nx workspace detection (nx.json fingerprinting). The correct score would be LOW — it is background context showing what workspace discovery looks like but not the Nx-specific implementation. aot-build.md (intended for bundler/mod) would be MEDIUM at most: jet build exists but lacks Nx graph awareness.
- ❌ Key requirements listed per spec are accurate (match actual requirement IDs)
  - The requirements.md defines R1 (workspace detection), R2 (Nx project graph), R3 (task pipeline as Nx targets), R4 (Nx config handling: nx.json + project.json). The reference context lists 'R1, R2' for workspace and 'R1, R2, R3' for cli without tying them to verifiable spec content. Since the spec paths don't exist, the requirement IDs cannot be cross-checked. Even assuming the best-case mapping to existing specs: pkg-manager-pnpm-parity.md does not implement R2 (Nx project graph) at all; jit-runner.md's task runner covers a different pipeline (jet.config.yaml, not project.json Nx executors) making its R3 mapping inaccurate.
- ❌ No irrelevant specs included
  - mod in cclab-jet/bundler (intended to map to bundler/mod.rs or aot-build.md) is only tangentially relevant: the bundler must eventually respect Nx graph-ordered builds (Q4), but the current Bundler struct has no interface for cross-project dependency ordering. Citing the bundler as MEDIUM for R3 overstates its role in the Nx task pipeline, which is primarily about the CLI and a new NxWorkspace module, not the bundler internals.

### Issues

- **[CRITICAL]** All four spec entries reference non-existent paths. The spec directory cclab/specs/cclab-jet/ contains: aot-build.md, bundle-optimization-hoisting.md, jit-runner.md, pkg-manager.md, pkg-manager-pnpm-parity.md, scope-hoisting.md, tree-shaking.md, variable-mangling.md. There are no subdirectories pkg_manager/ or bundler/ under cclab/specs/cclab-jet/, and no files named workspace.md or cli.md at any level. The reference context appears to have been generated using Rust source module paths (crates/cclab-jet/src/pkg_manager/workspace.rs, crates/cclab-jet/src/cli.rs) rather than spec file paths.
  - *Recommendation*: Replace all four entries with references to actual existing spec files. Suggested mapping: workspace detection (R1) → pkg-manager-pnpm-parity.md (LOW, background on workspace discovery); task pipeline / jet build (R3, R4) → jit-runner.md (MEDIUM, task runner concepts) and aot-build.md (MEDIUM, jet build pipeline). Add missing coverage for R2 and R4 by noting they require new specs.
- **[HIGH]** R2 (Nx project graph integration via nx graph --json) has zero spec coverage. This is a net-new integration: spawn nx graph --json as a subprocess, parse the project-node/dependency JSON output, and use it as the build order for dependent Jet projects. No existing spec models this interface, the NxGraph data structure, or the subprocess invocation protocol.
  - *Recommendation*: Create a new spec cclab/specs/cclab-jet/nx-support.md (or equivalent) covering: NxWorkspace detection logic (nx.json fingerprint), NxGraph schema (project nodes, dependencies), nx graph --json subprocess invocation, and the [nx] section in cclab/config.toml. Until this spec exists, it must be listed as a missing-coverage gap in the reference context.
- **[HIGH]** R4 (Nx configuration handling) has zero spec coverage. Pre-clarification Q5 requires a [nx] section in cclab/config.toml, and Q3 requires parsing project.json for Jet-specific executor targets. The existing cclab/config.toml has no [nx] section, and no spec defines the schema for NxConfig or the executor protocol in project.json.
  - *Recommendation*: Add [nx] config schema to the proposed nx-support.md spec: fields for project_graph_cmd, default_targets (build, install), executor_name. Also add project.json executor schema (jet executor target definition) so the change-spec author has a contract to implement against.
- **[MEDIUM]** pkg-manager-pnpm-parity.md is the closest existing spec to workspace concepts but implements pnpm-style workspace (package.json.workspaces glob patterns, workspace:* protocol, catalog shared versions, WorkspaceManager::discover_workspace reading jet-workspace.yaml). Nx workspace detection (primary: nx.json at root; secondary: node_modules/.bin/nx presence, @nx/workspace in package.json.devDependencies) is structurally different. Citing this spec at HIGH relevance misleads the change-spec author into reusing pnpm workspace scaffolding for a fundamentally different detection mechanism.
  - *Recommendation*: If pkg-manager-pnpm-parity.md is retained in the reference context, downgrade it to LOW relevance with a note: 'Background: workspace manager pattern for reference; Nx detection logic is separate and must be implemented in a new NxWorkspace module.'
- **[MEDIUM]** jit-runner.md (closest to cclab-jet task runner) covers an internal task pipeline using jet.config.yaml (TaskDef, dependsOn, DAG scheduling). Nx task pipeline (Q3) uses project.json with Nx executor protocol: { executor: '@cclab/jet:build', options: { ... } }. The existing TaskRunner::new(root_dir) reads jet.config.yaml, not nx project graph. Citing jit-runner.md as covering R3 implies the Nx pipeline can be bolted onto the existing task runner, which may not be the correct architecture.
  - *Recommendation*: Retain jit-runner.md at LOW-to-MEDIUM relevance as background (task DAG concepts, caching model), but clarify that the Nx executor integration requires a new NxTargetRunner or delegation path that invokes nx run <project>:<target> rather than the existing TaskRunner.
