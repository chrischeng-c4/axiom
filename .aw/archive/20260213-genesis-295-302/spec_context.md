---
change_id: genesis-295-302
type: spec_context
created_at: 2026-02-13T07:15:38.458344+00:00
updated_at: 2026-02-13T07:15:38.458344+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-genesis
  - cclab-cli
---

# Spec Context

## Relevant Specs

- **fetch-issues** (group: cclab-genesis)
  - relevance: high
  - reason: Core for Issue #302: adding issues param to run_change and eliminating fetch_issues action.
  - key sections: OpenRPC, Fetch Flow, Description Resolution
- **create-spec** (group: cclab-genesis)
  - relevance: high
  - reason: Core for Issue #295 (Class Plus), #298 (Tag system), #300 (Flowchart Plus), #301 (Requirement Plus). Defines the new tag-based requirement system.
  - key sections: Compositional Tag System, Requirement Diagram Mapping, OpenRPC: genesis_create_spec
- **delegate-agent** (group: cclab-genesis)
  - relevance: medium
  - reason: Relevant for #296 (OpenRPC schemas) and #299 (Sequence Plus for service-MCP-STATE interaction).
  - key sections: Verification Table, OpenRPC Method Definition
- **verdict-unification** (group: cclab-genesis)
  - relevance: medium
  - reason: Relevant for #298 (Tag/type validation mismatch) as it touches verdict enums.
  - key sections: Requirements
- **generate-tasks** (group: cclab-genesis)
  - relevance: medium
  - reason: Relevant for #300 (Flowchart Plus for algorithms) and mapping spec types to layers.
  - key sections: Task Generation Algorithm
- **architecture** (group: cclab-cli)
  - relevance: medium
  - reason: Relevant for #297 (Removing legacy code/aliases like Argus/Lint).
  - key sections: Command Structure, Command Dispatch Flow
