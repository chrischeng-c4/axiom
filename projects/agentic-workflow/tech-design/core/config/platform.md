---
id: platform-config-repo-spec
main_spec_ref: "crates/cclab-sdd/config/platform.md"
merge_strategy: extend
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Config and platform TDs define AW Core client boundary behavior."
---

# cclab sdd platform

Issue tracking platform configuration CLI. Supports GitHub, GitLab, and Jira.

## Overview
<!-- type: overview lang: markdown -->

Extend `.aw/config.toml` platform configuration from a single `[agentic_workflow.issue_platform]` section to a multi-platform architecture with four distinct sections:

| Section | Purpose | Status |
|---------|---------|--------|
| `[agentic_workflow.issue_platform]` | Issue tracking (GitHub/GitLab/Jira) | Existing — unchanged |
| `[agentic_workflow.repo_platform]` | Git repo, commits, PRs | New |
| `[agentic_workflow.spec_platform]` | Spec storage location | New |
| `[agentic_workflow.docs_platform]` | User-facing documentation | Future — commented out |

`repo_platform` enables post-merge git operations (auto-commit, auto-PR) consumed by `change-merge-git-integration`. `spec_platform` declares spec storage (currently `type = "local"` only). `docs_platform` is reserved for future use.

Parsing changes:
- `SddSection` in `config.rs` gains `repo_platform`, `spec_platform` fields
- `SddConfig` in `models/change.rs` gains `repo_platform: Option<RepoPlatformConfig>`, `spec_platform: Option<SpecPlatformConfig>`
- `platform show` CLI displays all configured platform sections
- `repo_platform.repo` is required (no fallback to `issue_platform.repo`)

### Commands

#### `cclab sdd platform set`

Interactive wizard to configure the **issue tracking** platform only.

**Flow**:
1. Select platform: GitHub (1) / GitLab (2) / Jira (3)
2. Platform-specific prompts:
   - **GitHub/GitLab**: Auto-detect repo from git remote, choose CLI OAuth or API token auth
   - **Jira**: Prompt for URL, project key, API token, and email
3. Write `[agentic_workflow.issue_platform]` section to `.aw/config.toml`

**Scope**: Only configures `[agentic_workflow.issue_platform]`. `[agentic_workflow.repo_platform]` and `[agentic_workflow.spec_platform]` must be added manually to `.aw/config.toml` or are included (commented out) in the `cclab init` template.

**Prerequisite**: `.aw/config.toml` must exist (run `cclab init` first).

#### `cclab sdd platform show`

Display current platform configuration.

**Load priority**:
1. `[agentic_workflow.issue_platform]` (new namespaced path)
2. `[platform]` (legacy fallback)
3. "No platform configured" message

Shows: source section, type, repo/URL/project, auth method, and auth details.

Additionally displays `[agentic_workflow.repo_platform]` and `[agentic_workflow.spec_platform]`:

| Section | Present | Absent |
|---------|---------|--------|
| `repo_platform` | Type, Repo, Default Branch, Auto Commit (on/off), Auto PR (on/off) | `"repo_platform: not configured"` |
| `spec_platform` | Type, Path | `"spec_platform: not configured"` |

### Configuration

#### New format: `[agentic_workflow.issue_platform]`

```yaml
agentic_workflow.issue_platform:
  type: github          # github | gitlab | jira
  repo: owner/repo      # GitHub/GitLab only
  auth_method: cli      # cli | token
  auth:                  # Only when auth_method = token
    envfile: ".env"
    envfield: GITHUB_TOKEN
```

#### Jira format

```yaml
agentic_workflow.issue_platform:
  type: jira
  url: https://yourorg.atlassian.net
  project: PROJ
  auth_method: token
  auth:
    envfile: ".env"
    envfield: JIRA_API_TOKEN
    envfield_email: JIRA_EMAIL
```

#### Legacy format (backward compatible)

```yaml
platform:
  type: github
  repo: owner/repo
  auth:
    envfile: ".env"
    envfield: GITHUB_TOKEN
```

The `PlatformConfig::load()` function tries `[agentic_workflow.issue_platform]` first, falling back to `[platform]`.

### Architecture

#### File layout

| File | Purpose |
|------|---------|
| `crates/cclab-cli/src/main.rs` | `#[command(alias = "sdd")]` on `Gen` variant |
| `crates/cclab-sdd/src/cli/platform.rs` | `set` + `show` commands, `detect_repo_from_git()` |
| `crates/cclab-sdd/src/cli/init.rs` | Delegates to `platform::detect_repo_from_git()` |
| `crates/cclab-sdd/src/services/platform_sync/config.rs` | `PlatformConfig::load()` with `SddSection` fallback |

#### Shared helpers

`detect_repo_from_git(project_root)` parses the git remote URL to extract `owner/repo`. Supports both SSH (`git@host:owner/repo.git`) and HTTPS (`https://host/owner/repo.git`) formats. Used by both `platform set` and `cclab init`.

#### Config upsert

`platform set` writes config by:
1. Reading `.aw/config.toml` as a string
2. Stripping old `[agentic_workflow.issue_platform]`, `[agentic_workflow.issue_platform.*]`, `[platform]`, and `[platform.*]` sections line-by-line
3. Appending the new `[agentic_workflow.issue_platform]` block
4. Writing back

This approach preserves all other config sections and comments.


## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: platform-config-repo-spec-requirements
title: Platform Config Repo Spec Requirements
requirements:
  R1:
    text: New [agentic_workflow.repo_platform] config section parsed by RepoPlatformConfig struct
    type: functional
    priority: high
    risk: medium
    verification: test
    notes: |
      Fields - type (string, required), repo (string, required — no fallback),
      default_branch (string, default "main"), auto_commit (bool, default false),
      auto_pr (bool, default false).
  R2:
    text: New [agentic_workflow.spec_platform] config section parsed by SpecPlatformConfig struct
    type: functional
    priority: medium
    risk: low
    verification: test
    notes: |
      Fields - type (string, required — currently only "local"),
      path (string, default ".aw/tech-design").
  R3:
    text: docs_platform reserved section documented in config template as commented-out block
    type: functional
    priority: low
    risk: low
    verification: inspection
    notes: |
      Not parsed — no struct needed yet. Template shows `# type = "github_pages"` as example.
  R4:
    text: SddSection struct gains repo_platform and spec_platform optional fields with serde default
    type: functional
    priority: high
    risk: low
    verification: test
    notes: |
      Existing issue_platform field unchanged.
  R5:
    text: SddConfig integration with load() and load_validated() semantics
    type: functional
    priority: high
    risk: medium
    verification: test
    notes: |
      load() returns None when sections absent; load_validated() requires both sections
      and fails with descriptive error if either absent.
  R6:
    text: cclab sdd platform show displays all configured platform sections
    type: interface
    priority: medium
    risk: low
    verification: test
    notes: |
      Issue, repo, spec sections shown with type and key fields.
      Absent sections show "not configured".
  R7:
    text: Config template includes repo_platform, spec_platform, and commented-out docs_platform
    type: functional
    priority: medium
    risk: low
    verification: inspection
    notes: |
      crates/cclab-sdd/templates/config.toml updated.

constraints:
  - "repo_platform.repo is required with no fallback (per clarification Q3)"
  - "spec_platform.type currently only supports 'local' — remote backends are future work"
  - "docs_platform is placeholder only — no runtime code changes"
  - "Existing PlatformConfig (issue_platform) struct and loading logic unchanged"
  - "SddSection in config.rs is module-private — only used for TOML deserialization"
---
requirementDiagram
    requirement R1 {
      id: R1
      text: repo_platform config section
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: spec_platform config section
      risk: low
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: docs_platform reserved section
      risk: low
      verifymethod: inspection
    }
    requirement R4 {
      id: R4
      text: SddSection extension
      risk: low
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: SddConfig integration
      risk: medium
      verifymethod: test
    }
    requirement R6 {
      id: R6
      text: platform show multi-section display
      risk: low
      verifymethod: test
    }
    requirement R7 {
      id: R7
      text: Config template update
      risk: low
      verifymethod: inspection
    }
```


## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  S1:
    name: Full repo_platform config loaded
    verifies: [R1, R5]
    given: |
      .aw/config.toml contains:
        [agentic_workflow.repo_platform]
        type = "github"
        repo = "owner/repo"
        default_branch = "main"
        auto_commit = true
        auto_pr = false
    when: |
      SddConfig::load() is called
    then: |
      config.repo_platform is Some(RepoPlatformConfig) with type_ = "github",
      repo = "owner/repo", default_branch = "main", auto_commit = true, auto_pr = false.
  S2:
    name: repo_platform section absent — validation error
    verifies: [R5]
    given: |
      .aw/config.toml has no [agentic_workflow.repo_platform] section
    when: |
      SddConfig::load_validated() is called (workflow tools, merge)
    then: |
      - Validation fails with error "Missing [agentic_workflow.repo_platform] in .aw/config.toml" and config example.
      - SddConfig::load() (non-validated) returns None without error.
  S3:
    name: repo_platform with defaults only
    verifies: [R1]
    given: |
      .aw/config.toml contains:
        [agentic_workflow.repo_platform]
        type = "github"
        repo = "owner/repo"
    when: |
      RepoPlatformConfig is deserialized
    then: |
      default_branch = "main", auto_commit = false, auto_pr = false (all defaults applied).
  S4:
    name: spec_platform local config
    verifies: [R2, R5]
    given: |
      .aw/config.toml contains:
        [agentic_workflow.spec_platform]
        type = "local"
        path = ".aw/tech-design"
    when: |
      SddConfig::load() is called
    then: |
      config.spec_platform is Some(SpecPlatformConfig) with type_ = "local", path = ".aw/tech-design".
  S5:
    name: spec_platform absent — validation error
    verifies: [R5]
    given: |
      .aw/config.toml has no [agentic_workflow.spec_platform] section
    when: |
      SddConfig::load_validated() is called (workflow tools, merge)
    then: |
      - Validation fails with error "Missing [agentic_workflow.spec_platform] in .aw/config.toml" and config example.
      - SddConfig::load() (non-validated) returns None without error.
  S6:
    name: SddSection deserializes all platform sections
    verifies: [R4]
    given: |
      .aw/config.toml contains [agentic_workflow.issue_platform], [agentic_workflow.repo_platform], and [agentic_workflow.spec_platform] sections
    when: |
      TOML is parsed into ConfigFile then SddSection
    then: |
      All three Option fields are Some(...). Each section is independently parsed.
  S7:
    name: platform show displays all sections
    verifies: [R6]
    given: |
      .aw/config.toml has issue_platform (github), repo_platform (github, auto_commit=true),
      and spec_platform (local)
    when: |
      cclab sdd platform show is run
    then: |
      Output shows three sections with their types and key fields.
      Missing sections show "not configured".
  S8:
    name: repo_platform missing repo field — parse error
    verifies: [R1]
    given: |
      .aw/config.toml contains:
        [agentic_workflow.repo_platform]
        type = "github"
        auto_commit = true
      (missing required repo field)
    when: |
      TOML is parsed
    then: |
      Deserialization fails with error indicating repo field is required.
```


## Config
<!-- type: config lang: yaml -->

```yaml
# New platform config sections — JSON Schema dialect expressed as YAML.

repo_platform:
  $id: repo-platform-config
  title: RepoPlatformConfig
  description: "Git repository and PR operations config — [agentic_workflow.repo_platform] in .aw/config.toml"
  type: object
  properties:
    type:
      type: string
      enum: [github, gitlab]
      description: VCS platform type
    repo:
      type: string
      pattern: "^[\\w.-]+/[\\w.-]+$"
      description: "Repository in owner/repo format. Required — no fallback to issue_platform.repo."
    default_branch:
      type: string
      default: main
      description: Target branch for auto-PR creation
    auto_commit:
      type: boolean
      default: false
      description: Auto git-commit cclab/ changes after merge archive
    auto_pr:
      type: boolean
      default: false
      description: Auto-create PR after auto-commit. Requires auto_commit=true.
  required: [type, repo]
  additionalProperties: false

spec_platform:
  $id: spec-platform-config
  title: SpecPlatformConfig
  description: "Spec storage config — [agentic_workflow.spec_platform] in .aw/config.toml"
  type: object
  properties:
    type:
      type: string
      enum: [local]
      description: Storage backend type. Currently only 'local' supported.
    path:
      type: string
      default: .aw/tech-design
      description: Relative path to spec storage directory from project root
  required: [type]
  additionalProperties: false

docs_platform:
  $id: docs-platform-config
  title: DocsPlatformConfig
  description: "[agentic_workflow.docs_platform] — reserved for future use. Not parsed at runtime."
  type: object
  properties:
    type:
      type: string
      enum: [github_pages, notion, confluence]
      description: Documentation platform type
  required: [type]
  additionalProperties: true

toml_example: |
  [agentic_workflow.issue_platform]
  type = "github"
  repo = "chrischeng-c4/cclab"
  auth_method = "cli"

  [agentic_workflow.repo_platform]
  type = "github"
  repo = "chrischeng-c4/cclab"
  default_branch = "main"
  auto_commit = true
  auto_pr = false

  [agentic_workflow.spec_platform]
  type = "local"
  path = ".aw/tech-design"

  # [agentic_workflow.docs_platform]
  # type = "github_pages"
```


## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/cclab-sdd/src/models/change.rs
    section: config
    action: modify
    impl_mode: codegen
    desc: |
      Add RepoPlatformConfig and SpecPlatformConfig structs.

      Add struct: RepoPlatformConfig {
        #[serde(rename = "type")] type_: String,
        repo: String,
        #[serde(default = "default_main")] default_branch: String,
        #[serde(default)] auto_commit: bool,
        #[serde(default)] auto_pr: bool,
      }

      Add struct: SpecPlatformConfig {
        #[serde(rename = "type")] type_: String,
        #[serde(default = "default_specs_path")] path: String,
      }

      Add fields to SddConfig:
        #[serde(default, skip_serializing_if = "Option::is_none")]
        repo_platform: Option<RepoPlatformConfig>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spec_platform: Option<SpecPlatformConfig>,

  - path: crates/cclab-sdd/src/services/platform_sync/config.rs
    section: config
    action: modify
    impl_mode: codegen
    desc: |
      Extend SddSection to parse new platform sections.

      Import RepoPlatformConfig, SpecPlatformConfig from models::change.

      Add fields to SddSection:
        #[serde(default)]
        repo_platform: Option<RepoPlatformConfig>,
        #[serde(default)]
        spec_platform: Option<SpecPlatformConfig>,

      No changes to PlatformConfig::load() — it only loads issue_platform.
      New sections are loaded via SddConfig::load() path.

  - path: crates/cclab-sdd/src/cli/platform.rs
    section: cli
    action: modify
    impl_mode: codegen
    desc: |
      Extend `run_show` to display repo_platform and spec_platform sections.

      After displaying issue_platform, load SddConfig and:
      - If repo_platform is Some: print section header, type, repo,
        default_branch, auto_commit, auto_pr
      - If repo_platform is None: print "repo_platform: not configured"
      - If spec_platform is Some: print section header, type, path
      - If spec_platform is None: print "spec_platform: not configured"

  - path: crates/cclab-sdd/templates/config.toml
    section: config
    action: modify
    impl_mode: codegen
    desc: |
      Add repo_platform and spec_platform sections to config template.

      Add after [agentic_workflow.issue_platform] section:
        [agentic_workflow.repo_platform]
        type = "github"
        repo = "owner/repo"
        default_branch = "main"
        auto_commit = false
        auto_pr = false

        [agentic_workflow.spec_platform]
        type = "local"
        path = ".aw/tech-design"

        # [agentic_workflow.docs_platform]
        # type = "github_pages"
  - action: annotate
    section: requirements
    impl_mode: hand-written
    description: "Traceability metadata edge for the requirements section."

  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

```
