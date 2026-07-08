---
files:
  - tools/fetch_issues.rs
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd_fetch_issues: Fetch Issues with Dependency DAG

> **Deprecated as standalone CLI tool.** The fetch logic is now invoked internally by `sdd_run_change` when the `issues` param is provided (or issue refs are detected in `description`). This file is retained as internal implementation reference documenting gh CLI integration, BFS discovery, DAG construction, and dependency extraction.

Fetch **main issues** (user-provided) and their **related issues** (link items, child items — context only), extract dependency relationships between main issues, build execution DAG.

**Key concepts**:
- **Main issues**: User-provided refs. Get clarification, context creation, and drive task planning.
- **Related issues**: Auto-fetched from main issues' link/child items. Provide context but are NOT clarified or separately contexted.

**Key behaviors**:
- Fetches main issues via platform CLI
- Auto-fetches related issues (linked/child items) for each main issue
- Extracts dependency edges between main issues (platform API + body text)
- Builds DAG with topological ordering (main issues only)
- Writes per-issue artifacts + `dependency_graph.md`
- Writes DAG to STATE.yaml for `run_change` routing

## OpenRPC
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_fetch_issues
summary: Fetch main + related issues from platform with dependency DAG
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: change_id
    required: true
    schema:
      type: string
      pattern: "^[a-z0-9-]+$"
  - name: issues
    required: true
    schema:
      type: array
      minItems: 1
      items:
        type: object
        required:
          - ref
        properties:
          ref:
            type: string
            description: "Main issue reference: number, #NNN, or full URL"
result:
  name: result
  schema:
    type: object
    required:
      - status
      - change_id
      - artifacts
      - issues
      - dag
    properties:
      status:
        type: string
        enum:
          - ok
          - partial
          - error
      change_id:
        type: string
      artifacts:
        type: array
        items:
          type: string
        description: Paths to written files (issue_*_{slug}.md files)
      issues:
        type: array
        items:
          type: object
          required:
            - number
            - title
            - state
            - role
          properties:
            number:
              type: integer
            title:
              type: string
            state:
              type: string
              enum:
                - open
                - closed
            role:
              type: string
              enum:
                - main
                - related
            related_to:
              type: integer
              description: Parent main issue number (related only)
            labels:
              type: array
              items:
                type: string
            url:
              type: string
            depends_on:
              type: array
              items:
                type: integer
              description: Main issues this issue depends on (main only)
      dag:
        type: object
        required:
          - main_issues
          - related_issues
          - edges
          - topological_order
        properties:
          main_issues:
            type: array
            items:
              type: integer
            description: User-provided issue numbers
          related_issues:
            type: object
            additionalProperties:
              type: array
              items:
                type: integer
            description: "Map: main issue number -> related issue numbers"
          edges:
            type: array
            items:
              type: object
              required:
                - from
                - to
                - type
              properties:
                from:
                  type: integer
                  description: Blocking main issue
                to:
                  type: integer
                  description: Blocked main issue
                type:
                  type: string
                  enum:
                    - blocks
                    - parent_of
                    - text_pattern
          topological_order:
            type: array
            items:
              type: integer
            description: Main issues in dependency-resolved execution order
      next:
        type: array
        items:
          type: object
          required:
            - tool
          properties:
            tool:
              type: string
            args:
              type: object
```

## Fetch Flow
<!-- type: doc lang: markdown -->

```mermaid
flowchart TD
    Start([sdd_fetch_issues called]) --> FetchMain["Fetch main issues<br/>(user-provided refs via CLI)"]
    FetchMain --> FetchRelated["For each main issue:<br/>fetch linked/child items via API"]
    FetchRelated --> ExtractDeps["Extract dependency edges<br/>between main issues"]
    ExtractDeps --> BuildDAG["Build DAG (main issues only)<br/>cycle check + topological sort"]
    BuildDAG --> WriteArtifacts["Write issue_{NNN}_{slug}.md<br/>(main + related)"]
    WriteArtifacts --> WriteState["STATE.yaml → dag section"]
```

### Related Issue Auto-Fetch

For each main issue, fetch linked/child items via platform API:

```mermaid
flowchart TD
    Start([Main issue fetched]) --> Platform{Platform?}
    Platform -->|GitHub| GH["gh api graphql<br/>subIssues, trackedIssues,<br/>relates_to links"]
    Platform -->|GitLab| GL["glab api /issues/:iid/links<br/>link_type: relates_to"]
    GH --> Filter["Filter: exclude issues already<br/>in main set"]
    GL --> Filter
    Filter --> WriteRelated["Write issue_{NNN}_{slug}.md<br/>with role: related"]
```

Related issues include: `relates_to` links, child/sub-issues, tracked issues. Dependency edges (`blocks`/`is_blocked_by`) are used for main issue DAG only.

## Dependency Extraction (main issues only)
<!-- type: doc lang: markdown -->

```mermaid
flowchart TD
    Start([Main issues fetched]) --> Platform{Platform?}

    Platform -->|GitHub| GHAPI["gh api graphql<br/>-H 'GraphQL-Features: sub_issues'<br/>blockedBy, blocking, parent, subIssues"]
    Platform -->|GitLab| GLAPI["glab api /projects/:id/issues/:iid/links<br/>link_type: blocks / is_blocked_by"]

    GHAPI --> TextParse["Parse body text patterns<br/>(both platforms, fallback)"]
    GLAPI --> TextParse

    TextParse --> MergeEdges["Merge all edges<br/>(API + text, deduplicate)"]
    MergeEdges --> FilterScope["Filter: keep only edges<br/>between main issues"]
    FilterScope --> BuildDAG["Build adjacency list"]
    BuildDAG --> CycleCheck{Cycle detected?}
    CycleCheck -->|Yes| ErrCycle[/"Error: cyclic dependency"/]
    CycleCheck -->|No| TopoSort["Topological sort → execution order"]
```

### Platform API Details

**GitHub** (GraphQL):

```graphql
query($owner: String!, $repo: String!, $numbers: [Int!]!) {
  repository(owner: $owner, name: $repo) {
    issues(first: 100, filterBy: {numbers: $numbers}) {
      nodes {
        number
        title
        parent { number }
        subIssues(first: 50) { nodes { number } }
        blockedBy(first: 50) { nodes { number } }
        blocking(first: 50) { nodes { number } }
        trackedIssues(first: 50) { nodes { number } }
      }
    }
  }
}
```

Requires header: `GraphQL-Features: sub_issues`

**GitLab** (REST — one call per issue):

```
GET /projects/:id/issues/:iid/links
```

| `link_type` | Usage | Tier |
|-------------|-------|------|
| `blocks` / `is_blocked_by` | Main issue DAG edges | Premium+ |
| `relates_to` | Related issues (context) | Free |

### Body Text Patterns (fallback)

| Pattern | Regex | Edge direction |
|---------|-------|---------------|
| `depends on #N` | `(?i)depends?\s+on\s+#(\d+)` | N blocks current |
| `blocked by #N` | `(?i)blocked?\s+by\s+#(\d+)` | N blocks current |
| `blocks #N` | `(?i)blocks?\s+#(\d+)` | current blocks N |
| `after #N` | `(?i)after\s+#(\d+)` | N blocks current |
| `requires #N` | `(?i)requires?\s+#(\d+)` | N blocks current |

## Platform Resolution
<!-- type: doc lang: markdown -->

```mermaid
flowchart TD
    Start([Parse each ref]) --> CheckRef{Full URL?}
    CheckRef -->|Yes| ParseURL["Extract platform + repo + number<br/>github.com → gh / gitlab.com → glab"]
    CheckRef -->|No| LoadConfig{config.toml [platform]?}
    LoadConfig -->|Yes| UseConfig["Use config.platform.type + repo"]
    LoadConfig -->|No| ErrConfig[/"Error: cannot resolve '#NNN'<br/>without [platform] config"/]
    ParseURL --> Fetch["Fetch via CLI"]
    UseConfig --> Fetch
```

| `platform.type` | CLI | Fetch command |
|-----------------|-----|---------------|
| `github` | `gh` | `gh issue view NNN --repo owner/repo --json number,title,body,labels,state,comments` |
| `gitlab` | `glab` | `glab issue view NNN --repo owner/repo` |

## Sequence Diagram
<!-- type: doc lang: markdown -->

```mermaid
sequenceDiagram
    participant MT as Mainthread
    participant FI as sdd_fetch_issues
    participant API as gh api / glab api
    participant CLI as gh / glab
    participant FS as Filesystem

    MT->>FI: fetch_issues(change_id, issues=[{ref:"#188"},{ref:"#189"}])

    par Fetch main issues
        FI->>CLI: gh issue view 188 --json ...
        CLI-->>FI: {number:188, title:"Auth", body:"..."}
        FI->>CLI: gh issue view 189 --json ...
        CLI-->>FI: {number:189, title:"Authz", body:"depends on #188"}
    end

    FI->>API: gh api graphql (relationships for #188, #189)
    API-->>FI: blockedBy/blocking + subIssues + trackedIssues

    par Fetch related issues (auto-discovered)
        FI->>CLI: gh issue view 190 --json ... (child of #188)
        CLI-->>FI: {number:190, title:"OAuth Flow"}
        FI->>CLI: gh issue view 191 --json ... (linked to #189)
        CLI-->>FI: {number:191, title:"RBAC Design"}
    end

    FI->>FI: Build DAG (main issues only: 188 → 189)

    par Write artifacts
        FI->>FS: write issue_188_authentication.md (role: main)
        FI->>FS: write issue_189_authorization.md (role: main)
        FI->>FS: write issue_190_oauth-flow.md (role: related, related_to: 188)
        FI->>FS: write issue_191_rbac-design.md (role: related, related_to: 189)
    end

    FI->>FS: STATE.yaml → write dag section
    FI-->>MT: {status:"ok", dag:{main_issues:[188,189], related_issues:{188:[190],189:[191]}, topological_order:[188,189]}}
```

## Artifact Formats
<!-- type: doc lang: markdown -->

### Per-Issue: `issue_{NNN}_{slug}.md`

Filename includes a slugified title for readability (e.g., `issue_188_authentication.md`). Slug is lowercase alphanumeric with hyphens, max 50 chars.

```markdown
---
number: {{number}}
title: "{{title}}"
state: {{state}}
role: {{main|related}}
related_to: {{parent_main_issue_number|null}}
labels:
  - {{label}}
url: {{url}}
depends_on: [{{dep_numbers}}]
fetched_at: {{iso8601}}
---

# Issue #{{number}}: {{title}}

\## Body

{{body}}

\## Labels

- {{label}}

\## Comments

### Comment by {{author}} ({{created_at}})

{{comment_body}}
```

### Dependency Graph

> **Note**: `dependency_graph.md` is NOT written as a standalone file. The dependency graph is embedded in `context_clarifications.md` as a `## Dependency Graph` section when DAG exists. See [pre-clarifications.md](../workflow/pre-clarifications.md) for the artifact format.

## STATE.yaml DAG Section
<!-- type: doc lang: markdown -->

Written by `sdd_fetch_issues`, consumed by `run_change`:

```yaml
dag:
  issues:
    - number: 188
      title: "Authentication"
      blocked_by: []
    - number: 189
      title: "Authorization"
      blocked_by: [188]
  clarify_index: 0       # current main issue being clarified
  context_index: 0        # current main issue in context loop
```

## Per-Issue Clarification Loop
<!-- type: doc lang: markdown -->

After fetch, `run_change` routes to `clarify` per main issue in topological order.

```mermaid
flowchart TD
    Start([run_change: issues fetched, phase before clarified]) --> CheckIndex{clarify_index < main_count?}
    CheckIndex -->|Yes| Clarify["action: clarify<br/>prompt includes issue at clarify_index<br/>+ its related issues as context"]
    Clarify --> UserQA["Mainthread asks user, calls<br/>sdd_write_artifact(issue=NNN)"]
    UserQA --> Advance["dag.clarify_index += 1"]
    Advance --> CheckIndex
    CheckIndex -->|No: all clarified| Done["phase: clarified<br/>→ context loop"]
```

Phase stays `clarified` after each issue. `run_change` checks `dag.clarify_index` to decide whether to loop or proceed to context.

## Per-Issue Context Loop (Spec-Issue Relevance Map)
<!-- type: doc lang: markdown -->

After all issues clarified, context creation runs **per main issue in topological order**. The goal is NOT per-issue fix planning — it's building a **cumulative spec-issue relevance map**.

```mermaid
flowchart TD
    Start([phase: clarified, all issues clarified]) --> Ctx["Per main issue in topological order:<br/>create + review spec_context<br/>create + review knowledge_context<br/>create + review codebase_context"]
    Ctx --> Cumulative["Context files are CUMULATIVE<br/>Each issue EXTENDS spec_context.md<br/>with its spec-issue relevance"]
    Cumulative --> Result["Result: spec_context.md contains<br/>per-issue sections + cumulative spec-issue map"]
    Result --> Gap["→ gap analysis uses full map<br/>to plan tasks by affected specs"]
```

### run_change Routing at codebase_context_approved

```mermaid
flowchart TD
    Start([codebase_context_approved]) --> HasDAG{dag exists?}
    HasDAG -->|No| Gap["→ gap_codebase_spec"]
    HasDAG -->|Yes| CheckNodes{context_index < main_count - 1?}
    CheckNodes -->|Yes| Advance["dag.context_index += 1"]
    Advance --> RouteSpec["action: explore_spec<br/>prompt: extend context for next main issue"]
    CheckNodes -->|No: all done| Gap
```

### Per-Node Prompt Enrichment

| Field | Value |
|-------|-------|
| `current_issue` | Main issue number + title + its related issues |
| `processed_issues` | Already-contexted main issue numbers |
| `remaining_issues` | Not-yet-processed main issue numbers |
| `previous_context` | Existing cumulative context file content |

### Full Traversal Example

```mermaid
sequenceDiagram
    participant MT as Mainthread
    participant RC as run_change

    Note over RC: DAG main: [#188, #189]

    rect rgb(255, 245, 230)
        Note over MT,RC: Clarify #188 (clarify_index=0)
        MT->>RC: run_change → clarify (issue #188 + related #190)
        Note over MT: Ask user questions about #188
        Note over MT,RC: Clarify #189 (clarify_index=1)
        MT->>RC: run_change → clarify (issue #189 + related #191)
        Note over MT: Ask user questions about #189
        Note over RC: All clarified → phase: clarified
    end

    rect rgb(230, 245, 255)
        Note over MT,RC: Context for #188 (context_index=0)
        MT->>RC: run_change → explore_spec (#188)
        MT->>RC: run_change → review_spec_context
        MT->>RC: run_change → explore_knowledge (#188)
        MT->>RC: run_change → review → create_codebase → review
        Note over RC: codebase_context_approved → advance
    end

    rect rgb(230, 255, 230)
        Note over MT,RC: Context for #189 (context_index=1, extends #188)
        MT->>RC: run_change → explore_spec (extends with #189)
        MT->>RC: run_change → review → ... → codebase_context_approved
        Note over RC: All main issues done → gap_codebase_spec
    end

    Note over RC: Gap analysis sees cumulative spec-issue map
    Note over RC: Tasks planned by affected specs, NOT per issue
```

### No New Phases Required

Both loops use **counters**, not new phases:

| Counter | Controls | Checked by |
|---------|----------|------------|
| `dag.clarify_index` | Per-issue clarification loop | `run_change` at phase `clarified` |
| `dag.context_index` | Per-issue context loop | `run_change` at phase `codebase_context_approved` |

## Description Resolution (in run_change code)
<!-- type: doc lang: markdown -->

`run_change` parses `description` for issue references using regex. If refs are found, it returns `action: "fetch_issues"` with `next` pointing to `sdd_fetch_issues` — **before** routing to `clarify`.

```mermaid
flowchart TD
    Start([run_change: new change]) --> Parse["Parse description for issue refs<br/>(regex: #\\d+, URLs, plain numbers)"]
    Parse --> HasRefs{Refs found?}
    HasRefs -->|Yes| FetchAction["action: fetch_issues<br/>next: [{tool: sdd_fetch_issues, args: {issues}}]"]
    HasRefs -->|No| ClarifyAction["action: clarify<br/>next: [{tool: sdd_write_artifact}]"]
```

Issue detection regex patterns:
- `#(\d+)` — hash ref
- `https?://(?:github|gitlab)\.com/.+/issues/(\d+)` — full URL
- `(\d+)` — plain number (only when description is purely numeric refs)

## Side Effects
<!-- type: doc lang: markdown -->

| Effect | Value |
|--------|-------|
| Files written | `issue_{NNN}_{slug}.md` per main + related issue |
| STATE.yaml `dag` | `{issues: [{number, title, blocked_by}], clarify_index: 0, context_index: 0}` |
| STATE.yaml `phase` | **not modified** |

> Dependency graph is embedded in `context_clarifications.md`, not as a standalone file.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

```