---
number: 956
title: "bug(sdd): payloads, prompts, and specs placed at change root instead of under groups/"
state: open
labels: [bug, P2, crate:sdd]
group: "group-directory-fix"
---

# #956 — bug(sdd): payloads, prompts, and specs placed at change root instead of under groups/

## Problem

In a multi-group change, `payloads/`, `prompts/` (spec-fill + implementation), and `specs/` are written to the **change root** instead of under `groups/{group}/`.

## Expected

```
cclab/changes/{id}/
├── groups/
│   ├── specir-codegen/
│   │   ├── payloads/
│   │   ├── prompts/
│   │   ├── specs/
│   │   ├── requirements.md         ✅ correct
│   │   ├── pre_clarifications.md   ✅ correct
│   │   ├── post_clarifications.md  ✅ correct
│   │   └── reference_context.md    ✅ correct
│   └── testgen-requirementplus/
│       └── (same structure)
├── STATE.yaml
└── user_input.md
```

## Actual

```
cclab/changes/{id}/
├── groups/
│   ├── specir-codegen/
│   │   ├── prompts/                ✅ (ref-context prompts only)
│   │   ├── requirements.md         ✅
│   │   ├── pre_clarifications.md   ✅
│   │   ├── post_clarifications.md  ✅
│   │   └── reference_context.md    ✅
│   └── testgen-requirementplus/
│       └── (same)
├── payloads/                       ❌ root-level, not group-scoped
├── prompts/                        ❌ spec-fill + impl prompts at root
├── specs/                          ❌ change specs at root
├── STATE.yaml
└── user_input.md
```

## Affected Phases

- `create_change_spec` — writes specs and fill prompts to root
- `create_change_implementation` — writes impl prompts to root
- Payload files written by mainthread (manual) — root payloads dir

## Impact

- Multi-group changes have ambiguous file ownership (which group does a root-level spec belong to?)
- Merge phase reads from `changes/{id}/specs/` which works but loses group traceability
- SDD viewer cannot associate specs with their originating group
