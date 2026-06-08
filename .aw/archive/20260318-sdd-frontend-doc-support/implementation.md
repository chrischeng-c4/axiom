---
id: implementation
type: change_implementation
change_id: sdd-frontend-doc-support
---

# Implementation

## Summary

Redesigned change-spec logic with 17 section types, group-scoped spec paths (groups/{group}/specs/), section fill order, cross-reference system, CLI-driven prompt architecture, and agent policy updates to enforce cclab sdd artifact CLI usage.

## Diff

```diff
diff --git a/.gemini/policies/sdd-agent.toml b/.gemini/policies/sdd-agent.toml
index fafe6dd4..ce7204ea 100644
--- a/.gemini/policies/sdd-agent.toml
+++ b/.gemini/policies/sdd-agent.toml
@@ -1,13 +1,28 @@
-# SDD agent policy: block local file/shell tools (agents use MCP tools only)
+# SDD agent policy: allow only cclab CLI and payload writes
 
+# Allow cclab sdd artifact CLI commands
 [[rule]]
-toolName = ["write_file", "edit", "replace"]
+toolName = "run_shell_command"
+commandPrefix = "cclab sdd artifact"
+decision = "allow"
+priority = 200
+
+# Block all other shell commands
+[[rule]]
+toolName = "run_shell_command"
 decision = "deny"
 priority = 100
-deny_message = "File writing blocked - use MCP sdd_write_artifact instead"
+deny_message = "Only 'cclab sdd artifact' commands are allowed"
 
+# Allow writing payload JSON files only
 [[rule]]
-toolName = "run_shell_command"
+toolName = "write_file"
+decision = "allow"
+priority = 200
+
+# Block direct file editing (agents write via CLI payload + artifact command)
+[[rule]]
+toolName = ["edit", "replace"]
 decision = "deny"
 priority = 100
-deny_message = "Shell commands blocked - use MCP tools instead"
+deny_message = "Use cclab sdd artifact CLI instead of direct editing"
diff --git a/.gemini/settings.json b/.gemini/settings.json
index 9fa516e9..b8ac8fe8 100644
--- a/.gemini/settings.json
+++ b/.gemini/settings.json
@@ -1,16 +1,6 @@
 {
   "mcp": {
-    "allowed": [
-      "cclab-mcp"
-    ]
   },
   "mcpServers": {
-    "cclab-mcp": {
-      "excludeTools": [
-        "sdd_delegate_agent"
-      ],
-      "type": "http",
-      "url": "http://localhost:3456/mcp"
-    }
   }
-}
\ No newline at end of file
+}
diff --git a/cclab/specs/cclab-sdd/config/agents.md b/cclab/specs/cclab-sdd/config/agents.md
index 78defa51..51f46f41 100644
--- a/cclab/specs/cclab-sdd/config/agents.md
+++ b/cclab/specs/cclab-sdd/config/agents.md
@@ -24,24 +24,22 @@ Declares project modules for monorepo-aware language detection. Used by task gen
 
 **Resolution**: `language_for_path(file)` finds the longest matching `path` prefix. `primary_language()` returns the first module's language.
 
-```toml
-[[project.modules]]
-path = "."
-language = "rust"
-
-[[project.modules]]
-path = "frontend/"
-language = "typescript"
-framework = "react"
+```yaml
+project.modules:
+  - path: "."
+    language: rust
+  - path: "frontend/"
+    language: typescript
+    framework: react
 ```
 
 ## Top-level `envfile`
 
 Global environment file loaded for all agents. Relative paths resolve against project root.
 
-```toml
+```yaml
 # Global envfile - loaded for all agents
-envfile = ".env"
+envfile: ".env"
 ```
 
 Provider-specific envfiles override global values on key collision.
diff --git a/cclab/specs/cclab-sdd/config/platform.md b/cclab/specs/cclab-sdd/config/platform.md
index 8a655de4..df4b4663 100644
--- a/cclab/specs/cclab-sdd/config/platform.md
+++ b/cclab/specs/cclab-sdd/config/platform.md
@@ -6,9 +6,10 @@ Issue tracking platform configuration CLI. Supports GitHub, GitLab, and Jira.
 
 `cclab sdd` is an alias for `cclab gen`. The `platform` subcommand group manages issue tracking platform configuration, which is stored in `cclab/config.toml` under `[sdd.issue_platform]`.
 
-```bash
-cclab sdd platform set    # Interactive setup
-cclab sdd platform show   # Show current configuration
+```yaml
+commands:
+  - cclab sdd platform set    # Interactive setup
+  - cclab sdd platform show   # Show current configuration
 ```
 
 ## Commands
@@ -41,42 +42,39 @@ Shows: source section, type, repo/URL/project, auth method, and auth details.
 
 ### New format: `[sdd.issue_platform]`
 
-```toml
-[sdd.issue_platform]
-type = "github"          # "github" | "gitlab" | "jira"
-repo = "owner/repo"      # GitHub/GitLab only
-auth_method = "cli"      # "cli" | "token"
-
-[sdd.issue_platform.auth]  # Only when auth_method = "token"
-envfile = ".env"
-envfield = "GITHUB_TOKEN"
+```yaml
+sdd.issue_platform:
+  type: github          # github | gitlab | jira
+  repo: owner/repo      # GitHub/GitLab only
+  auth_method: cli      # cli | token
+  auth:                  # Only when auth_method = token
+    envfile: ".env"
+    envfield: GITHUB_TOKEN
 ```
 
 ### Jira format
 
-```toml
-[sdd.issue_platform]
-type = "jira"
-url = "https://yourorg.atlassian.net"
-project = "PROJ"
-auth_method = "token"
-
-[sdd.issue_platform.auth]
-envfile = ".env"
-envfield = "JIRA_API_TOKEN"
-envfield_email = "JIRA_EMAIL"
+```yaml
+sdd.issue_platform:
+  type: jira
+  url: https://yourorg.atlassian.net
+  project: PROJ
+  auth_method: token
+  auth:
+    envfile: ".env"
+    envfield: JIRA_API_TOKEN
+    envfield_email: JIRA_EMAIL
 ```
 
 ### Legacy format (backward compatible)
 
-```toml
-[platform]
-type = "github"
-repo = "owner/repo"
-
-[platform.auth]
-envfile = ".env"
-envfield = "GITHUB_TOKEN"
+```yaml
+platform:
+  type: github
+  repo: owner/repo
+  auth:
+    envfile: ".env"
+    envfield: GITHUB_TOKEN
 ```
 
 The `PlatformConfig::load()` function tries `[sdd.issue_platform]` first, falling back to `[platform]`.
diff --git a/cclab/specs/cclab-sdd/generate/architecture.md b/cclab/specs/cclab-sdd/generate/architecture.md
index 5979a198..25edc0ee 100644
--- a/cclab/specs/cclab-sdd/generate/architecture.md
+++ b/cclab/specs/cclab-sdd/generate/architecture.md
@@ -19,7 +19,7 @@ design_elements:
 
 ## Overview
 
-Generate (極光) provides diagram and specification generation from structured input.
+Generate provides diagram and specification generation from structured input.
 
 ## Generation Pipeline
 
diff --git a/cclab/specs/cclab-sdd/generate/mermaid-plus-conversion.md b/cclab/specs/cclab-sdd/generate/mermaid-plus-conversion.md
index 474216a6..18c06151 100644
--- a/cclab/specs/cclab-sdd/generate/mermaid-plus-conversion.md
+++ b/cclab/specs/cclab-sdd/generate/mermaid-plus-conversion.md
@@ -123,29 +123,18 @@ flowchart TB
 
 Each generator follows this pattern:
 
-```rust
-pub fn generate(&self, def: &Definition, validation: ValidationResult) -> Result<Output, String> {
-    let frontmatter = self.generate_frontmatter(def)?;
-    let diagram = self.generate_mermaid(def)?;
-
-    // Combine into Mermaid+ format (frontmatter inside code block)
-    let mut combined = String::new();
-    combined.push_str("```mermaid\n");
-    combined.push_str("---\n");
-    combined.push_str(&frontmatter);
-    combined.push_str("---\n");
-    combined.push_str(&diagram);
-    combined.push_str("```\n");
-
-    // Add validation warnings as HTML comments
-    if !validation.warnings.is_empty() {
-        combined.push_str("\n<!-- Validation Warnings:\n");
-        // ...
-        combined.push_str("-->\n");
-    }
-
-    Ok(Output { frontmatter, diagram, validation, combined })
-}
+```mermaid
+flowchart TD
+    Input[Definition + ValidationResult] --> GenFM[generate_frontmatter]
+    Input --> GenMD[generate_mermaid]
+    GenFM --> Combine[combine: fence + frontmatter + diagram]
+    GenMD --> Combine
+    Combine --> CheckWarn{has warnings?}
+    CheckWarn -->|yes| Append[append HTML comment warnings]
+    CheckWarn -->|no| Output[return Output]
+    Append --> Output
 ```
 
+Output format: ` ```mermaid ` → `---` → YAML frontmatter → `---` → Mermaid diagram → ` ``` ` → optional `<!-- Validation Warnings -->` HTML comment.
+
 </spec>
diff --git a/cclab/specs/cclab-sdd/generate/spec-model.md b/cclab/specs/cclab-sdd/generate/spec-model.md
index 6470a428..38e6bd69 100644
--- a/cclab/specs/cclab-sdd/generate/spec-model.md
+++ b/cclab/specs/cclab-sdd/generate/spec-model.md
@@ -270,35 +270,36 @@ requirementDiagram
 
 **Example output (Python/pytest):**
 
-```python
-# tests/test_auth_token_refresh.py
-# Generated from Requirement Plus diagram
-from src.handlers.auth import AuthHandler      # ← satisfies R1, R2
-from src.services.token import TokenService    # ← satisfies R1
+```yaml
+# Generated test structure from Requirement Plus diagram
+test_file: tests/test_auth_token_refresh.py
+imports:
+  - module: src.handlers.auth.AuthHandler    # satisfies R1, R2
+  - module: src.services.token.TokenService  # satisfies R1
 
-class TestR1_TokenRefresh:
-    """R1: Users can refresh expired tokens (risk: Medium)"""
+test_classes:
+  - name: TestR1_TokenRefresh
+    requirement: "R1: Users can refresh expired tokens (risk: Medium)"
+    tests:
+      - name: test_valid_refresh_token
+        scenario: "valid refresh token → verifies R1"
+        given: a user with a valid refresh token
+        when: POST /refresh is called
+        then: a new access token is returned
 
-    def test_valid_refresh_token(self):
-        """Scenario: valid refresh token → verifies R1"""
-        # GIVEN a user with a valid refresh token
-        # WHEN POST /refresh is called
-        # THEN a new access token is returned
-
-class TestR2_InvalidToken:
-    """R2: Invalid tokens return 401 (risk: High) [derives R1]"""
-
-    def test_expired_token_returns_401(self):
-        """Scenario: expired token returns 401 → verifies R2"""
-        # GIVEN an expired refresh token
-        # WHEN POST /refresh is called
-        # THEN 401 is returned
-
-    def test_revoked_token_returns_401(self):
-        """Scenario: revoked token returns 401 → verifies R2"""
-        # GIVEN a revoked refresh token
-        # WHEN POST /refresh is called
-        # THEN 401 is returned
+  - name: TestR2_InvalidToken
+    requirement: "R2: Invalid tokens return 401 (risk: High) [derives R1]"
+    tests:
+      - name: test_expired_token_returns_401
+        scenario: "expired token returns 401 → verifies R2"
+        given: an expired refresh token
+        when: POST /refresh is called
+        then: 401 is returned
+      - name: test_revoked_token_returns_401
+        scenario: "revoked token returns 401 → verifies R2"
+        given: a revoked refresh token
+        when: POST /refresh is called
+        then: 401 is returned
 ```
 
 **Does NOT describe:** test implementation details (mocks, fixtures, assertions).
diff --git a/cclab/specs/cclab-sdd/generate/template-mcp-configs.md b/cclab/specs/cclab-sdd/generate/template-mcp-configs.md
index fea8c94d..1e1ccad2 100644
--- a/cclab/specs/cclab-sdd/generate/template-mcp-configs.md
+++ b/cclab/specs/cclab-sdd/generate/template-mcp-configs.md
@@ -80,11 +80,12 @@ Four MCP configuration files connect LLM clients (Claude Code, Gemini CLI, Codex
 
 ### Codex CLI — `.codex/config.toml`
 
-```toml
-[mcp_servers.cclab-mcp]
-type = "http"
-url = "http://localhost:3456/mcp"
-disabled_tools = ["sdd_delegate_agent"]
+```yaml
+mcp_servers:
+  cclab-mcp:
+    type: http
+    url: http://localhost:3456/mcp
+    disabled_tools: [sdd_delegate_agent]
 ```
 
 ## Installation
diff --git a/cclab/specs/cclab-sdd/interfaces/cli/commands.md b/cclab/specs/cclab-sdd/interfaces/cli/commands.md
index 7ea41741..01703246 100644
--- a/cclab/specs/cclab-sdd/interfaces/cli/commands.md
+++ b/cclab/specs/cclab-sdd/interfaces/cli/commands.md
@@ -55,18 +55,13 @@ cclab sdd:
 
 ## Registration
 
-```rust
-// cclab-sdd-cli/src/lib.rs
-impl CliModule for SddCli {
-    fn name(&self) -> &'static str { "sdd" }
-}
+```yaml
+# cclab-sdd-cli/src/lib.rs
+CliModule:
+  name: "sdd"
+  struct: SddCli
+  distributed_slice: CLI_MODULES
 
-#[distributed_slice(CLI_MODULES)]
-static SDD_CLI: &dyn CliModule = &SddCli;
-```
-
-Force-link in `cclab-cli/src/main.rs`:
-
-```rust
-use cclab_sdd_cli as _;
+# cclab-cli/src/main.rs
+force_link: cclab_sdd_cli
 ```
diff --git a/cclab/specs/cclab-sdd/logic/change-merge.md b/cclab/specs/cclab-sdd/logic/change-merge.md
index f4438758..8f7ac07d 100644
--- a/cclab/specs/cclab-sdd/logic/change-merge.md
+++ b/cclab/specs/cclab-sdd/logic/change-merge.md
@@ -2,7 +2,7 @@
 id: change-merge-logic
 type: spec
 title: "Change Merge — Logic"
-version: 1
+version: 3
 files:
   - mcp/tools/change_merge/create.rs
   - workflow/merge.rs
@@ -26,15 +26,12 @@ crr: false  # programmatic merge, no CRR
 
 ```mermaid
 flowchart TD
-    Start([workflow_create_change_merge]) --> FindSpecs[find specs in changes/{id}/specs/]
+    Start([workflow_create_change_merge]) --> FindSpecs[find specs in changes/{id}/groups/*/specs/]
     FindSpecs --> Empty{specs found?}
     Empty -->|no| ArchiveEmpty[archive with no merge]
     Empty -->|yes| Loop[for each spec file]
     Loop --> ReadFM[read frontmatter: main_spec_ref]
-    ReadFM --> HasRef{main_spec_ref set?}
-    HasRef -->|yes| UsePath[target = cclab/specs/{main_spec_ref}]
-    HasRef -->|no| Derive[derive from spec_id + change scope]
-    Derive --> UsePath
+    ReadFM --> UsePath[target = cclab/specs/main_spec_ref]
     UsePath --> Strip[strip change-spec-only frontmatter fields]
     Strip --> Write[write to cclab/specs/{target}]
     Write --> Loop
@@ -51,6 +48,8 @@ stripped_fields:
   - main_spec_ref      # only used for merge routing
   - merge_strategy     # only used during merge
   - create_complete    # internal marker
+  - fill_sections      # internal tracking
+  - filled_sections    # internal tracking
 ```
 
 ## Merge Strategy
diff --git a/cclab/specs/cclab-sdd/logic/change-spec.md b/cclab/specs/cclab-sdd/logic/change-spec.md
index 032ab70d..4123ea0e 100644
--- a/cclab/specs/cclab-sdd/logic/change-spec.md
+++ b/cclab/specs/cclab-sdd/logic/change-spec.md
@@ -2,7 +2,7 @@
 id: change-spec-logic
 type: spec
 title: "Change Spec — Logic"
-version: 3
+version: 7
 files:
   - mcp/tools/change_spec/common.rs
   - mcp/tools/change_spec/create.rs
@@ -28,16 +28,15 @@ max_revisions: 2
 
 ```mermaid
 stateDiagram-v2
-    [*] --> Skeleton: no spec file
-    Skeleton --> Analyze: skeleton written to specs/{spec_id}.md
-    Analyze --> Fill_1: fill_sections determined
-    Fill_1 --> Fill_2: section written
-    Fill_2 --> Fill_N: next section
+    [*] --> Prepared: spec preparation (copy or skeleton)
+    Prepared --> Fill_1: fill_sections from spec_plan.sections
+    Fill_1 --> Fill_2: section written via CLI
+    Fill_2 --> Fill_N: next section (fill_order priority)
     Fill_N --> Prune: all fill_sections done
     Prune --> Review: TODO sections removed, create_complete=true
     Review --> Revise: REVIEWED verdict
     Review --> NextSpec: APPROVED verdict
-    Revise --> Review: re-review
+    Revise --> Review: re-review after fix
     NextSpec --> [*]: all specs done
 ```
 
@@ -60,101 +59,390 @@ Same pattern as reference-context (see `logic/reference-context.md` § Artifact
 2. **Post-agent verification** — Check `filled_sections` frontmatter updated by artifact CLI
 3. **Mainthread fallback** — If agent wrote spec file directly, program reads content, extracts section text, calls `execute_artifact()` to rewrite with proper frontmatter tracking
 
-## Create Mode: Iterative Section-by-Section
+## Spec Preparation (pre-step)
 
-Each spec is built **one section at a time**. The workflow tool drives the loop:
+Before change_spec phase begins, the system prepares spec files using the `spec_plan` from reference_context:
 
-### Mode 1: New spec (no existing main spec)
+| action | what happens |
+|--------|-------------|
+| `modify` | Copy `cclab/specs/{source}` → `groups/{group}/specs/{spec_id}.md`, set `main_spec_ref` in frontmatter |
+| `create` | Write skeleton → `groups/{group}/specs/{spec_id}.md`, set `main_spec_ref` in frontmatter |
 
-1. **Skeleton** — `sdd_workflow_create_change_spec` generates a universal skeleton with all sections as `<!-- TODO -->`
-2. **Analyze** — Agent reads context, determines `fill_sections` list and `main_spec_ref`
-3. **Fill** — For each section in `fill_sections`, workflow returns a section-specific prompt. Agent writes ONE section per `sdd_artifact_create_change_spec` call
-4. **Prune** — When all `fill_sections` are in `filled_sections`, unfilled `<!-- TODO -->` sections are removed, `create_complete: true` is set
+After preparation, every spec file already has `main_spec_ref` set. Agent never needs to determine it.
 
-### Mode 2: Existing main spec (update/extend)
+### Cross-group main_spec_ref deduplication
 
-1. **Copy** — Existing spec from `cclab/specs/{main_spec_ref}` is copied to `changes/{id}/specs/{spec_id}.md`
-2. **Analyze** — Agent reads context + existing spec, determines which sections need changes → sets `fill_sections`
-3. **Fill** — Same iterative loop: one section per artifact call, agent modifies only flagged sections
-4. **Prune** — Same as Mode 1
+**Constraint**: No two groups may target the same `main_spec_ref`. Enforced automatically after all groups complete reference_context, before prepare_specs begins.
+
+**Resolution**: conflicting specs are moved to the earliest group that claims them.
+
+```mermaid
+flowchart TD
+    Start([all groups have spec_plan]) --> Collect[collect all spec_plan entries ordered by group index]
+    Collect --> Loop[for each group, for each spec]
+    Loop --> Check{main_spec_ref already seen?}
+    Check -->|no| Record[record main_spec_ref → current group]
+    Check -->|yes| Merge[merge requirements into earlier group's spec]
+    Merge --> Remove[remove spec from current group's plan]
+    Record --> Loop
+    Remove --> Loop
+    Loop -->|done| Ready[all main_spec_ref unique across groups]
+```
+
+The earlier group's spec absorbs the later group's requirements for that `main_spec_ref`. The later group no longer owns that spec — it can still read it as reference but does not produce a change spec for it.
+
+## Section Selection
+
+Sections for each spec are determined by `spec_plan.sections` from reference_context. Two sources:
+
+### Rule engine (CLI-side, no agent)
+
+Requirements text is matched against keyword rules to suggest section types:
+
+```yaml
+section_rules:
+  - match: "endpoint|route|api|REST|HTTP"
+    sections: [rest-api, schema]
+  - match: "rpc|json-rpc|MCP tool"
+    sections: [rpc-api, schema]
+  - match: "queue|pubsub|webhook|background|async"
+    sections: [async-api]
+  - match: "database|model|table|migration|collection"
+    sections: [db-model]
+  - match: "state|phase|lifecycle|transition"
+    sections: [state-machine]
+  - match: "UI|page|component|layout|frontend"
+    sections: [wireframe, component]
+  - match: "CLI|command|subcommand|flag"
+    sections: [cli]
+  - match: "config|env|settings|.toml|.env"
+    sections: [config]
+  - match: "token|color|spacing|typography|theme"
+    sections: [design-token]
+  - always: [overview, changes]
+  - if_section_count_gt_2: [test-plan]
+  - if_section_count_gt_3: [interaction, logic, dependency]
+```
+
+### Review fallback (two-layer CRR)
+
+Section selection is **best effort** — review catches gaps:
+
+1. **reference_context CRR** (max 1 revision) — reviews spec_plan.sections completeness
+2. **change_spec CRR** (max 2 revisions) — reviews content, can request missing sections
+
+## Section Fill Order
+
+Sections within a spec are filled in dependency order (hardcoded priority):
+
+```yaml
+fill_order:
+  - overview          # 0: understand scope first
+  - db-model          # 1: data layer
+  - schema            # 2: referenced by API types
+  - state-machine     # 3: state transitions
+  - logic             # 4: business logic
+  - dependency        # 5: architecture
+  - interaction       # 6: call chains
+  - rest-api          # 7: API surface (refs schema)
+  - rpc-api           # 7
+  - async-api         # 7
+  - cli               # 7
+  - wireframe         # 8: UI layout
+  - component         # 8: UI components
+  - design-token      # 8: design system
+  - config            # 9
+  - test-plan         # 10: needs all others
+  - changes           # 11: last
+```
+
+## Create Mode: CLI-Driven Section Fill
+
+Each section is filled via structured CLI call. Agent provides flag values, CLI generates formatted content.
+
+### CLI Command
+
+```
+cclab sdd artifact create-change-spec {change_id} {spec_id} \
+  --type {section-type} [per-type flags...] \
+  --sdd-id {id} --sdd-refs "#ref1,#ref2"
+```
+
+### Prompt Architecture
+
+**1 base template + 17 type-specific inserts** (stored as data, not separate prompts):
+
+```markdown
+# Task: Fill {{section_type}} section for spec '{{spec_id}}'
+
+## Context
+- Requirements: groups/{{group_id}}/requirements.md
+- Reference: groups/{{group_id}}/reference_context.md
+- Filled so far: {{filled_sections}}
+
+## Command
+cclab sdd artifact create-change-spec {{change_id}} {{spec_id}} --type {{section_type}}
+
+## Flags
+{{type_specific_flags}}
+
+Read context, determine flag values, run the command.
+```
+
+Type-specific flag descriptions are stored as data:
+
+```yaml
+section_prompts:
+  rest-api:
+    flags:
+      --endpoint: "HTTP method + path (e.g. POST /docs/{id}/pages)"
+      --request-schema: "Request body schema name"
+      --response-schema: "Response schema name"
+      --status-codes: "Comma-separated (e.g. 201,400,404)"
+    guidance: "One endpoint per section. Include error responses."
+
+  logic:
+    flags:
+      --nodes: "Node id:label pairs (e.g. A:validate,B:check_quota)"
+      --edges: "Edges (e.g. A-->B,B-->|valid|C)"
+      --conditions: "Condition labels on decision edges"
+    guidance: "One function/handler per section. Max 10 nodes."
+
+  db-model:
+    flags:
+      --entities: "Entity names (e.g. DocPageVersion)"
+      --fields: "Per-entity fields (e.g. DocPageVersion:id,page_id,content)"
+      --relations: "Relations (e.g. DocPage||--o{DocPageVersion:has)"
+    guidance: "Use DB column types, not language types."
+
+  # ... 14 more types, same pattern
+```
+
+### Fill Loop
+
+```mermaid
+flowchart TD
+    Start([spec_plan.sections]) --> Sort[sort by fill_order priority]
+    Sort --> Loop[for each section type]
+    Loop --> BuildPrompt[inject type flags into base template]
+    BuildPrompt --> Agent[agent reads context + runs CLI]
+    Agent --> Verify{CLI succeeded?}
+    Verify -->|yes| Next[next section]
+    Verify -->|no| Retry[retry once]
+    Next --> Loop
+    Loop -->|done| Prune[prune unfilled TODO sections]
+    Prune --> Complete[create_complete = true]
+```
+
+### Mode 1: New spec (skeleton from preparation)
+
+Skeleton has `<!-- TODO -->` for each section in `spec_plan.sections`. Fill loop fills them in order.
+
+### Mode 2: Existing spec (copied from preparation)
+
+Copied spec has existing content. Agent modifies only sections listed in `fill_sections`.
 
 ### Frontmatter tracking
 
 ```yaml
 ---
 id: {spec_id}
-main_spec_ref: ~                    # target path in cclab/specs/ (null until analyze)
+main_spec_ref: cclab-sdd/logic/my-spec.md   # target path in cclab/specs/ (set by spec preparation)
 merge_strategy: new | append | replace
-spec_type: http-api | event-driven | data-model | algorithm | integration | rpc-api | workflow | utility
 refs: [dep-spec-1, dep-spec-2]     # topological dependencies
-fill_sections: [overview, requirements, scenarios]  # set during analyze
+fill_sections: [overview, rest-api, schema, interaction]  # from spec_plan.sections
 filled_sections: [overview]         # incremented per artifact call
 create_complete: true               # set after prune
 ---
 ```
 
+### main_spec_ref requirement
+
+`main_spec_ref` is the target path under `cclab/specs/` where the spec will be merged. Set by **spec preparation** from `spec_plan` — never by the agent.
+
+| Mode | Source | main_spec_ref |
+|------|--------|---------------|
+| `modify` | `cclab/specs/{source}` copied into change | Same path — merge overwrites the original |
+| `create` | No existing spec | Target path from `spec_plan.main_spec_ref` — merge creates new file |
+
+**Validation gate**: Prune step rejects specs with `main_spec_ref: ~` (should never happen if spec preparation ran correctly).
+
 ### Artifact call per section
 
-Each `sdd_artifact_create_change_spec` call writes exactly **one** section:
+Each `cclab sdd artifact create-change-spec` call writes exactly **one** section via structured CLI flags:
 
-```json
-{
-  "spec_id": "my-spec",
-  "section": "overview",
-  "content": "Markdown content after H2 heading",
-  "fill_sections": ["overview", "requirements", "scenarios"],
-  "main_spec_ref": "cclab-sdd/tools/my-spec.md",
-  "merge_strategy": "new"
-}
+```
+cclab sdd artifact create-change-spec {change_id} {spec_id} \
+  --type {section-type} [per-type flags...] \
+  --sdd-id {id} --sdd-refs "#ref1,#ref2"
+```
+
+The CLI generates formatted content (OpenAPI YAML, Mermaid, JSON Schema, etc.) and updates `filled_sections` in frontmatter. `fill_sections`, `main_spec_ref`, and `merge_strategy` are set by spec preparation and not modified by the agent.
+
+## Directory Structure
+
+Specs live **under group**, not at change root. Each group is a self-contained unit:
+
+```
+changes/{change-id}/
+├── STATE.yaml
+├── user_input.md
+├── issues/
+└── groups/
+    └── {group-id}/
+        ├── requirements.md
+        ├── pre_clarifications.md
+        ├── reference_context.md
+        ├── post_clarifications.md
+        └── specs/
+            ├── {spec-id-1}.md
+            ├── {spec-id-2}.md
+            └── ...
 ```
 
-The `fill_sections`, `main_spec_ref`, and `merge_strategy` are persisted to frontmatter on the first call and ignored on subsequent calls if already set.
+Every phase iterates `for group in groups: do(group)` — the group carries its own requirements, clarifications, reference context, and specs through the full lifecycle.
 
 ## Spec Execution Order
 
-Topological sort on `refs:` frontmatter field. Specs with dependencies are created after their deps.
+Topological sort on `refs:` frontmatter field within the same group. Specs with dependencies are created after their deps.
 
-## Universal Skeleton
+## Section Type System
+
+Each section in a spec is **one section = one type**. Sections are self-describing via an HTML comment annotation after the heading:
+
+```markdown
+## {section title}
+<!-- type: {spec-type} lang: {spec-lang} -->
+
+{section desc}
+
+```{spec-lang}
+{content}
+```
+```
+
+### Section Type → Spec Lang Mapping
+
+| spec-type | lang | code fence | use for |
+|-----------|------|------------|---------|
+| `rest-api` | `yaml` | ` ```yaml ` | REST API interface (OpenAPI 3.1) |
+| `rpc-api` | `json` | ` ```json ` | JSON-RPC interface (OpenRPC 1.3) |
+| `async-api` | `yaml` | ` ```yaml ` | Background/WebSocket (AsyncAPI 2.6) |
+| `cli` | `yaml` | ` ```yaml ` | CLI command tree + args |
+| `schema` | `json` | ` ```json ` | Interface/data schema (JSON Schema) |
+| `logic` | `mermaid` | ` ```mermaid ` | Business logic (flowchart) |
+| `interaction` | `mermaid` | ` ```mermaid ` | Actor interaction (sequence diagram) |
+| `state-machine` | `mermaid` | ` ```mermaid ` | State transitions (stateDiagram-v2) |
+| `db-model` | `mermaid` | ` ```mermaid ` | Database model (erDiagram) |
+| `test-plan` | `mermaid` | ` ```mermaid ` | Test coverage (requirementDiagram) |
+| `dependency` | `mermaid` | ` ```mermaid ` | Dependency/type hierarchy (classDiagram) |
+| `wireframe` | `yaml` | ` ```yaml ` | UI wireframe (framework-agnostic YAML DSL) |
+| `component` | `json` | ` ```json ` | UI component contract — Custom Elements Manifest (CEM) |
+| `design-token` | `json` | ` ```json ` | Design tokens — W3C DTCG 2025.10 |
+| `config` | `json` | ` ```json ` | Config file schema (JSON Schema) |
+| `overview` | `markdown` | (no fence) | Description, prose only |
+| `changes` | `yaml` | ` ```yaml ` | File change list (path + action) |
+
+### Cross-Reference System
+
+Sections link to each other via **content-level** `id` and `$ref` — not in the HTML annotation. Each spec lang has its own standard mechanism:
+
+| spec lang family | id mechanism | ref mechanism |
+|-----------------|-------------|---------------|
+| OpenAPI 3.1 | `x-sdd.id` | `x-sdd.refs[*].$ref` |
+| OpenRPC 1.3 | `x-sdd.id` | `x-sdd.refs[*].$ref` |
+| AsyncAPI 2.6 | `x-sdd.id` | `x-sdd.refs[*].$ref` |
+| JSON Schema | `$id` | `$ref` |
+| CEM (component) | `x-sdd.id` | `x-sdd.refs[*].$ref` |
+| DTCG (design-token) | `$extensions.sdd.id` | `$extensions.sdd.refs[*].$ref` |
+| Mermaid Plus | frontmatter `id` | frontmatter `refs[*].$ref` |
+| YAML DSL (wireframe, cli, config, changes) | `_sdd.id` | `_sdd.refs[*].$ref` |
+
+**$ref syntax** (unified across all langs):
+- `#local-id` — same file
+- `other-spec#remote-id` — cross file
+
+**Example — OpenAPI linking to Mermaid Plus**:
 
 ```yaml
-sections:
-  - overview          # always required
-  - requirements      # always required
-  - scenarios         # always required
-  - diagrams          # conditional on spec_type
-  - api_spec          # conditional on spec_type
-  - test_plan         # always required
-  - changes           # always required
+# rest-api section
+paths:
+  /docs/{id}/pages:
+    post:
+      summary: Create page
+      x-sdd:
+        id: create-page-api
+        refs:
+          - $ref: "#create-page-flow"
 ```
 
-## spec_type → Required Elements
+```mermaid
+---
+id: create-page-flow
+refs:
+  - $ref: "#doc-service-logic"
+  - $ref: "#docpage-model"
+---
+sequenceDiagram
+    Router->>DocService: create_page()
+    Router->>AuthService: check()
+```
+
+```mermaid
+---
+id: doc-service-logic
+refs:
+  - $ref: "#docpage-model"
+---
+flowchart TD
+    A[validate] --> B[insert DocPage]
+```
+
+**Traversal**: API endpoint → interaction flow → business logic → data model. Each layer's content carries its own `id` and `refs`, forming a DAG.
+
+**Rule**: If a section may be referenced by other sections, its content MUST declare an `id`. Leaf sections (overview, changes) typically don't need one.
+
+### Parsing
+
+Section annotations are extracted by regex:
+
+```
+^## (.+)\n<!-- type: ([\w-]+) lang: (\w+) -->
+```
+
+Cross-references are extracted from content:
+- Mermaid: YAML frontmatter `id` and `refs`
+- OpenAPI/OpenRPC/AsyncAPI/CEM: `x-sdd.id` and `x-sdd.refs`
+- JSON Schema: `$id` and `$ref`
+- YAML DSL: `_sdd.id` and `_sdd.refs`
 
-| spec_type | diagrams | api_spec |
-|-----------|----------|----------|
-| http-api | sequence, flowchart | OpenAPI 3.1 |
-| event-driven | sequence, state | AsyncAPI 2.6 |
-| data-model | erd, class | JSON Schema |
-| algorithm | flowchart, state | — |
-| integration | sequence, flowchart | depends |
-| rpc-api | sequence | OpenRPC 1.3 |
-| workflow | state, flowchart | Serverless Workflow 0.8 |
-| utility | flowchart | — |
+This enables:
+- **Extract** — pull a specific section by type
+- **Insert** — generate section with correct lang + code fence from type
+- **Validate** — verify code fence content matches spec-lang format
+- **Trace** — follow `$ref` links to build dependency DAG across sections and files
 
-## Diagram Format
+### Migration from spec_type
 
-All diagrams MUST use Mermaid. Type selection per CLAUDE.md Mermaid Diagram Selection.
+The old file-level `spec_type` frontmatter field is **deprecated**. Section types replace it:
+- Old: one `spec_type` per file → determines required diagrams + api_spec
+- New: each section declares its own type → agent senses what sections are needed
 
 ## Review
 
 ### Checklist
 
-1. spec_type matches actual content
-2. Requirements: complete, no gaps vs reference context
-3. Scenarios: cover happy path + error cases
-4. Diagrams: correct type for structure, syntactically valid Mermaid
-5. API spec: semantically valid, matches requirements
-6. Test plan: covers all requirements
-7. Dependencies (`refs:`) consistent with other specs
+1. Each section has `<!-- type: ... lang: ... -->` annotation
+2. Section type matches actual content (e.g. `state-machine` section contains `stateDiagram-v2`)
+3. Code fence lang matches declared lang
+4. Cross-references: all `$ref` targets exist (no dangling refs)
+5. Referenceable sections have `id` declared in content
+6. Requirements: complete, no gaps vs reference context
+7. Scenarios: cover happy path + error cases
+8. Mermaid sections: syntactically valid, correct diagram type for declared section type
+9. API spec sections: semantically valid, matches requirements
+10. Test plan: covers all requirements
+11. Dependencies (`refs:`) consistent with other specs
 
 ### Verdict
 
diff --git a/cclab/specs/cclab-sdd/logic/executor-resolution.md b/cclab/specs/cclab-sdd/logic/executor-resolution.md
index 09b0a799..de5371db 100644
--- a/cclab/specs/cclab-sdd/logic/executor-resolution.md
+++ b/cclab/specs/cclab-sdd/logic/executor-resolution.md
@@ -17,21 +17,20 @@ Cross-cutting concern: how each workflow action resolves which agent (or mainthr
 
 `cclab/config.toml` (under project root):
 
-```toml
-[workflow]
-version = "v6"
-
-[workflow.agents]
-create_spec = "gemini:pro"
-review_spec = "codex:max"
-revise_spec = "gemini:pro"
-implement = "mainthread"
-code_review = "codex:balanced"
-resolve = "mainthread"
-begin_merge = "mainthread"
-review_merge = "codex:max"
-fix_merge = "mainthread"
-explore = "gemini:flash"
+```yaml
+workflow:
+  version: v6
+  agents:
+    create_spec: "gemini:pro"
+    review_spec: "codex:max"
+    revise_spec: "gemini:pro"
+    implement: "mainthread"
+    code_review: "codex:balanced"
+    resolve: "mainthread"
+    begin_merge: "mainthread"
+    review_merge: "codex:max"
+    fix_merge: "mainthread"
+    explore: "gemini:flash"
 ```
 
 ## Agent String Format
diff --git a/cclab/specs/cclab-sdd/logic/implement-task.md b/cclab/specs/cclab-sdd/logic/implement-task.md
index 627342d7..b5877b41 100644
--- a/cclab/specs/cclab-sdd/logic/implement-task.md
+++ b/cclab/specs/cclab-sdd/logic/implement-task.md
@@ -2,7 +2,7 @@
 id: implement-task-logic
 type: spec
 title: "Implementation — Logic"
-version: 1
+version: 2
 files:
   - mcp/tools/change_impl/common.rs
   - mcp/tools/change_impl/create.rs
@@ -63,9 +63,11 @@ ImplSubState:
 
 Kahn's algorithm on `refs:` frontmatter — topological sort of spec DAG.
 
-```rust
-// From change_impl/common.rs
-build_spec_execution_order(specs_dir: &Path) -> Vec<String>
+```yaml
+# From change_impl/common.rs
+function: build_spec_execution_order
+input: groups/{group_id}/specs/ (Path)
+output: Vec<String>  # ordered spec ids
 ```
 
 ## Codegen Routing
@@ -88,8 +90,8 @@ flowchart TD
 
 Read all approved specs and implement them in dependency order.
 
-1. Read specs: `sdd_read_artifact(scope="specs")`
-2. Read requirements for context
+1. Read specs: `groups/{{group_id}}/specs/*.md`
+2. Read requirements: `groups/{{group_id}}/requirements.md`
 3. Implement ALL tasks following layer order: data → logic → integration → testing
 4. Maintain code quality: tests, error handling, documentation
 ```
@@ -107,7 +109,7 @@ Read the review in implementation.md and address all issues.
 {{/if}}
 
 ## Steps
-1. Read spec: `changes/{{change_id}}/specs/{{spec_id}}.md`
+1. Read spec: `changes/{{change_id}}/groups/{{group_id}}/specs/{{spec_id}}.md`
 2. Implement code changes
 3. Run tests to verify
 ```
diff --git a/cclab/specs/cclab-sdd/logic/reference-context.md b/cclab/specs/cclab-sdd/logic/reference-context.md
index cf9420e4..d9e760df 100644
--- a/cclab/specs/cclab-sdd/logic/reference-context.md
+++ b/cclab/specs/cclab-sdd/logic/reference-context.md
@@ -2,7 +2,7 @@
 id: reference-context-logic
 type: spec
 title: "Reference Context — Logic"
-version: 2
+version: 4
 files:
   - tools/common_reference_context.rs
   - tools/create_reference_context.rs
@@ -100,22 +100,31 @@ Explore the codebase and specs to identify relevant references for this group.
 
 ## Steps
 
-1. Read: `sdd_read_artifact(scope="pre_clarifications", group_id="{{group_id}}")`
-2. Read: `sdd_read_artifact(scope="user_input")`
+1. Read: `groups/{{group_id}}/pre_clarifications.md`
+2. Read: `user_input.md`
 3. Explore:
    - Search `cclab/specs/` for related specs
    - Search `cclab/knowledge/` for relevant docs
-   - Use Lens/Prism tools to analyze code structure
 4. For each relevant spec/doc, assess relevance (high/medium/low)
-5. Call `sdd_artifact_create_reference_context` with structured specs array
+5. Write payload JSON, then run:
+   `cclab sdd artifact create-reference-context {{change_id}} <payload_path>`
 
-## Output Format
+## Output: specs array
 
 Each spec reference must include:
 - spec_id: path relative to cclab/specs/
 - spec_group: logical grouping (e.g. "mcp-tools", "state-machine")
 - relevance: high | medium | low
 - key_requirements: array of relevant requirement summaries
+
+## Output: spec_plan array
+
+For each change spec that will be created:
+- spec_id: identifier for the new change spec
+- action: "modify" (copy existing) or "create" (new skeleton)
+- main_spec_ref: target path in cclab/specs/ (REQUIRED)
+- source: path of existing spec to copy (only for "modify")
+- sections: array of section types this spec needs (see change-spec.md § Section Selection)
 ```
 
 ### Review
@@ -130,6 +139,9 @@ Each spec reference must include:
 - [ ] Key requirements: accurately summarize what matters
 - [ ] No false positives: irrelevant specs not included
 - [ ] Completeness: knowledge docs and code references included
+- [ ] spec_plan: every entry has main_spec_ref set (not null)
+- [ ] spec_plan: sections are reasonable for the requirements
+- [ ] spec_plan: modify entries have valid source paths
 
 ## Verdict
 
@@ -145,9 +157,10 @@ Each spec reference must include:
 
 Read review feedback and update reference context.
 
-1. Read current: `sdd_read_artifact(scope="reference_context", group_id="{{group_id}}")`
+1. Read current: `groups/{{group_id}}/reference_context.md`
 2. Address each review issue
-3. Call `sdd_artifact_revise_reference_context` with corrected specs array
+3. Write corrected payload JSON, then run:
+   `cclab sdd artifact revise-reference-context {{change_id}} <payload_path>`
 ```
 
 ## CRR Cycle
@@ -184,6 +197,40 @@ Max 1 revision per group. Auto-approve on exceed.
 }
 ```
 
+### spec_plan array (create input)
+
+Determines which change specs will be created, where they merge to, and which sections each spec needs.
+
+```json
+{
+  "type": "array",
+  "minItems": 1,
+  "items": {
+    "type": "object",
+    "required": ["spec_id", "action", "main_spec_ref", "sections"],
+    "properties": {
+      "spec_id": { "type": "string", "description": "Change spec identifier" },
+      "action": { "type": "string", "enum": ["modify", "create"] },
+      "main_spec_ref": { "type": "string", "description": "Target path in cclab/specs/ (REQUIRED)" },
+      "source": { "type": "string", "description": "Existing spec to copy (only for modify)" },
+      "sections": {
+        "type": "array",
+        "items": { "type": "string", "enum": ["overview","rest-api","rpc-api","async-api","cli","schema","logic","interaction","state-machine","db-model","test-plan","dependency","wireframe","component","design-token","config","changes"] },
+        "description": "Section types this spec needs. Determined by rule engine + agent input."
+      }
+    }
+  }
+}
+```
+
+**Section selection**: CLI rule engine matches requirements text against keyword patterns to suggest sections (see `change-spec.md` § Section Selection). Agent may adjust during reference_context creation. Review CRR catches gaps.
+
+After reference_context is approved, the system uses `spec_plan` to **prepare spec files**:
+- `action: modify` → copy `cclab/specs/{source}` to `groups/{group}/specs/{spec_id}.md`, set `main_spec_ref`
+- `action: create` → write skeleton with `<!-- TODO -->` for each section in `sections`, set `main_spec_ref`
+
+This ensures every spec has `main_spec_ref` and `sections` set before change_spec phase begins.
+
 ### review params
 
 ```json
diff --git a/cclab/specs/cclab-sdd/tools/utils/platform-sync.md b/cclab/specs/cclab-sdd/tools/utils/platform-sync.md
index 722055f9..ba9daaa1 100644
--- a/cclab/specs/cclab-sdd/tools/utils/platform-sync.md
+++ b/cclab/specs/cclab-sdd/tools/utils/platform-sync.md
@@ -58,30 +58,25 @@ Config is loaded from `cclab/config.toml` (preferred) or `cclab/config.yaml` (le
 
 ### Full Config Schema (TOML)
 
-```toml
-[sdd.issue_platform]
-type = "github"          # "github" or "gitlab" (gitlab not yet implemented)
-repo = "owner/repo"      # Repository in owner/repo format
-
-[sdd.issue_platform.auth]
-envfile = ".env"          # Path to .env file (relative to project root)
-envfield = "GITHUB_TOKEN" # Field name in .env file
-
-[sdd.issue_platform.labels]
-auto_create = true                # Auto-create labels if they don't exist
-proposal = "cclab:sdd:proposal"   # Label for proposal (parent) issues
-spec = "cclab:sdd:spec"           # Label for spec (child) issues
-
-[sdd.issue_platform.labels.scope]
-enabled = true
-pattern = "crate:{scope}"         # Label format ({scope} replaced with detected scope)
-
-[sdd.issue_platform.labels.scope.auto_detect]
-path_regex = "crates/cclab-([^/]+)/"  # Regex group 1 = scope name
-
-[sdd.issue_platform.title]
-proposal = "[{change_id}] {title}"         # Proposal issue title format
-spec = "[{change_id}/spec] {spec_id}"      # Spec issue title format
+```yaml
+sdd.issue_platform:
+  type: github          # github or gitlab (gitlab not yet implemented)
+  repo: owner/repo      # Repository in owner/repo format
+  auth:
+    envfile: ".env"          # Path to .env file (relative to project root)
+    envfield: GITHUB_TOKEN   # Field name in .env file
+  labels:
+    auto_create: true
+    proposal: "cclab:sdd:proposal"
+    spec: "cclab:sdd:spec"
+    scope:
+      enabled: true
+      pattern: "crate:{scope}"
+      auto_detect:
+        path_regex: "crates/cclab-([^/]+)/"
+  title:
+    proposal: "[{change_id}] {title}"
+    spec: "[{change_id}/spec] {spec_id}"
 ```
 
 ### Authentication Resolution Order

```

## Review: sdd-frontend-doc-support-spec

verdict: REJECTED
reviewer: reviewer
iteration: 1
change_id: sdd-frontend-doc-support

**Summary**: The implementation does not yet satisfy the Phase 1 spec. The new section model is only partially wired: frontend/doc types are declared but cannot be created through the skeleton or artifact section list, the review/revise flow still hardcodes legacy `changes/{id}/specs/` paths instead of the new group-scoped layout, the promised generator CLI and YAML-backed prompt architecture are not exposed at runtime, and the crate test suite is not green.

### Checklist

- [FAIL] Group-scoped spec directory migration works end to end
  - `resolve_next_spec`, `review_change_spec`, and `revise_change_spec` still read/write legacy `changes/{id}/specs/` paths, so grouped specs are not handled correctly through the full lifecycle.
- [FAIL] Frontend/doc section support is reachable in generated specs
  - The universal skeleton and allowed section list still only expose the legacy section set, so `frontend` and `doc` cannot actually be generated or filled.
- [FAIL] Wave 1 generator CLI and prompt architecture are wired
  - Generator modules were added, but the `spec` CLI still only exposes `list` and `create`, and the new `section_prompts.yaml` data is not consumed by the runtime.
- [FAIL] Automated tests pass
  - `cargo test -p cclab-sdd --lib --quiet` is failing in this tree, including changed-area tests such as `tools::create_change_spec::tests::test_workflow_creates_skeleton` and `tools::review_change_spec::tests::test_review_workflow_returns_prompt`.

### Issues

- **[HIGH]** The new group-scoped spec layout is not implemented consistently across the change-spec lifecycle.
  - *Recommendation*: Update spec resolution, review, revise, and prompt generation to use the shared group-aware path helpers instead of hardcoded `change_dir/specs` paths.
- **[HIGH]** Frontend/doc support is not actually usable because the skeleton and artifact section routing still only support the old legacy section set.
  - *Recommendation*: Add the missing typed sections to the skeleton and artifact routing, or otherwise expose a real path for authoring `frontend`, `doc`, and the other new section types.
- **[HIGH]** The promised generator CLI and YAML-driven prompt architecture are not reachable at runtime.
  - *Recommendation*: Wire a real `spec gen` command or equivalent CLI entrypoint to `GeneratorArgs`/`generate_section`, and load the embedded section prompt data instead of keeping guidance hardcoded.
- **[MEDIUM]** The changed-area test suite is failing, so the implementation is not in a releasable state.
  - *Recommendation*: Fix the failing workflow tests and rerun the relevant crate test suite before re-review.
