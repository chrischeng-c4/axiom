---
id: sdd-generate-sequence-plus-schema
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Sequence Plus Schema

## Overview
<!-- type: overview lang: markdown -->

Mermaid-Plus sequence diagram definition types in
`projects/agentic-workflow/src/generate/diagrams/sequence_plus/schema.rs`. Eleven serde shapes:

- `SequenceDef` — top-level diagram (id, optional title, participants map, messages, loops, alts, notes, optional description).
- `ParticipantDef` — one participant (label, type with default Participant, optional description).
- `ParticipantType` — 2-variant enum lowercase, default Participant.
- `MessageDef` — one message (from, to, text, arrow_type with default Solid, activate bool, deactivate bool, optional description).
- `ArrowType` — 4-variant enum snake_case, default Solid.
- `LoopDef` — one loop block (label, start usize, end usize, optional description).
- `AltDef` — one alt/opt block (block_type with default Alt, condition, start usize, end usize, else_branches Vec).
- `AltBlockType` — 5-variant enum lowercase, default Alt.
- `ElseBranch` — one else branch (optional condition, start, end).
- `NoteDef` — one note (text, position with default RightOf, participants Vec, optional after_message).
- `NotePosition` — 3-variant enum snake_case, default RightOf.

Codegen also owns the `serde::{Deserialize, Serialize}` import used by the
generated derives. Hand-written outside CODEGEN: module docstring, non-serde
`use` statements, and the `#[cfg(test)] mod tests` block.

This spec exercises:

1. **`x-serde-rename`** — `participant_type`, `arrow_type`, `block_type` map to JSON key `"type"`.
2. **Default variants** — multiple enums have `is_default: true` markers.
3. **`HashMap<K, V>` field** — `participants: HashMap<String, ParticipantDef>` via x-rust-type.
4. **Bare `usize` fields** — start/end fields use `x-rust-type: "usize"`.
5. **`Vec<T>` of inner types**.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ParticipantType:
    type: string
    enum: [Participant, Actor]
    description: Participant type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: lowercase
      variants:
        - { name: Participant, is_default: true, doc: "Standard participant (default)." }
        - { name: Actor, doc: "Actor (stick-figure) participant." }

  ArrowType:
    type: string
    enum: [Solid, Dotted, SolidOpen, DottedOpen]
    description: Arrow type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: snake_case
      variants:
        - { name: Solid, is_default: true, doc: "Solid arrow (default)." }
        - { name: Dotted, doc: "Dotted arrow." }
        - { name: SolidOpen, doc: "Solid arrow with open arrowhead." }
        - { name: DottedOpen, doc: "Dotted arrow with open arrowhead." }

  AltBlockType:
    type: string
    enum: [Alt, Opt, Par, Critical, Break]
    description: Alt block type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: lowercase
      variants:
        - { name: Alt, is_default: true, doc: "Alt block (default)." }
        - { name: Opt, doc: "Opt block." }
        - { name: Par, doc: "Par block (parallel)." }
        - { name: Critical, doc: "Critical block." }
        - { name: Break, doc: "Break block." }

  NotePosition:
    type: string
    enum: [RightOf, LeftOf, Over]
    description: Note position.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default, PartialEq]
      serde_rename_all: snake_case
      variants:
        - { name: RightOf, is_default: true, doc: "Right of (default)." }
        - { name: LeftOf, doc: "Left of." }
        - { name: Over, doc: "Over." }

  SequenceDef:
    type: object
    required: [id, title, participants, messages, loops, alts, notes, description]
    description: Sequence diagram definition (input from LLM).
    properties:
      id:
        type: string
        description: "Diagram identifier."
      title:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Diagram title."
      participants:
        type: object
        x-rust-type: "HashMap<String, ParticipantDef>"
        description: "Participant definitions keyed by participant ID."
      messages:
        type: array
        items: { $ref: "#/definitions/MessageDef" }
        x-rust-type: "Vec<MessageDef>"
        description: "Message sequence."
      loops:
        type: array
        items: { $ref: "#/definitions/LoopDef" }
        x-rust-type: "Vec<LoopDef>"
        x-serde-default: true
        description: "Loop blocks."
      alts:
        type: array
        items: { $ref: "#/definitions/AltDef" }
        x-rust-type: "Vec<AltDef>"
        x-serde-default: true
        description: "Alt/opt blocks."
      notes:
        type: array
        items: { $ref: "#/definitions/NoteDef" }
        x-rust-type: "Vec<NoteDef>"
        x-serde-default: true
        description: "Notes."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Diagram description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ParticipantDef:
    type: object
    required: [label, participant_type, description]
    description: Participant definition.
    properties:
      label:
        type: string
        description: "Display label."
      participant_type:
        type: string
        x-rust-type: "ParticipantType"
        x-serde-rename: "type"
        x-serde-default: true
        description: "Participant type."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  MessageDef:
    type: object
    required: [from, to, text, arrow_type, activate, deactivate, description]
    description: Message definition.
    properties:
      from:
        type: string
        description: "Source participant ID."
      to:
        type: string
        description: "Target participant ID."
      text:
        type: string
        description: "Message text."
      arrow_type:
        type: string
        x-rust-type: "ArrowType"
        x-serde-rename: "type"
        x-serde-default: true
        description: "Arrow type."
      activate:
        type: boolean
        x-serde-default: true
        description: "Activate target on this message."
      deactivate:
        type: boolean
        x-serde-default: true
        description: "Deactivate source after this message."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Message description (for documentation)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  LoopDef:
    type: object
    required: [label, start, end, description]
    description: Loop block definition.
    properties:
      label:
        type: string
        description: "Loop condition/label."
      start:
        type: integer
        x-rust-type: "usize"
        description: "Start message index (0-based)."
      end:
        type: integer
        x-rust-type: "usize"
        description: "End message index (0-based, inclusive)."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AltDef:
    type: object
    required: [block_type, condition, start, end, else_branches]
    description: Alt/Opt block definition.
    properties:
      block_type:
        type: string
        x-rust-type: "AltBlockType"
        x-serde-rename: "type"
        x-serde-default: true
        description: "Block type."
      condition:
        type: string
        description: "Primary condition."
      start:
        type: integer
        x-rust-type: "usize"
        description: "Start message index (0-based)."
      end:
        type: integer
        x-rust-type: "usize"
        description: "End message index for primary branch (0-based, inclusive)."
      else_branches:
        type: array
        items: { $ref: "#/definitions/ElseBranch" }
        x-rust-type: "Vec<ElseBranch>"
        x-serde-default: true
        description: "Else branches (for alt blocks)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ElseBranch:
    type: object
    required: [condition, start, end]
    description: Else branch definition.
    properties:
      condition:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Condition (empty for final else)."
      start:
        type: integer
        x-rust-type: "usize"
        description: "Start message index."
      end:
        type: integer
        x-rust-type: "usize"
        description: "End message index."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  NoteDef:
    type: object
    required: [text, position, participants, after_message]
    description: Note definition.
    properties:
      text:
        type: string
        description: "Note text."
      position:
        type: string
        x-rust-type: "NotePosition"
        x-serde-default: true
        description: "Position."
      participants:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Participant ID(s) the note is attached to."
      after_message:
        type: integer
        x-rust-type: "Option<usize>"
        x-serde-default: true
        description: "After which message index (optional)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/sequence_plus/schema.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - use serde::{Deserialize, Serialize}
      - SequenceDef
      - ParticipantDef
      - ParticipantType
      - MessageDef
      - ArrowType
      - LoopDef
      - AltDef
      - AltBlockType
      - ElseBranch
      - NoteDef
      - NotePosition
    description: |
      Codegen replaces the serde import and all eleven type declarations.
  - path: projects/agentic-workflow/src/generate/diagrams/sequence_plus/schema.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module docstring, non-serde `use`
      statements (`std::collections::HashMap`), and the
      `#[cfg(test)] mod tests` block.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Eleven serde shapes; mix of structs/enums; HashMap + Vec + Options + bare usize.
- [schema] All well-formed; x-serde-rename for `type`-keyed fields; default variants via is_default.
- [changes] All eleven in `replaces`; tests + module-level items hand-written.
