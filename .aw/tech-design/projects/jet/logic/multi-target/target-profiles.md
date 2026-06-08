---
id: projects-jet-logic-multi-target-target-profiles-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# jet multi-target — target profiles

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/multi-target/target-profiles.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# jet multi-target — target profiles

### Overview

Specifies the **capability profile** each target publishes. Profiles
let Cue components ask the substrate "is feature X available here?"
and let the build pipeline reject a UI source that requires a feature
the target doesn't support.

The capability matrix lives in `target-profiles.yaml`; this doc
explains the schema, the semantics of each capability, and the
**degradation rules** that govern what happens when a target can't
honor a style or element.

### Capability schema

The full schema is in `target-profiles.yaml.schema.json`; the matrix
is in `target-profiles.yaml`. Summary of the field shape:

```yaml
target: web | desktop | tui            # canonical target name
display_kinds: [block, flex]           # ../../wasm-renderer/layout-runtime.md DisplayKind enum
color_depth: rgba | indexed-256 | ansi-16 | mono
text:
  shaping: full | grid | none          # full = HarfBuzz; grid = monospace cell; none = no text
  bidi: true | false                   # ../../wasm-renderer/line-bidi.md
  fonts: any | system | terminal       # font selection scope
input:
  pointer: true | false                # mouse / trackpad / pen
  keyboard: true | false               # always true for web/desktop/tui
  touch: true | false                  # primary touches only — see element-contract §"Out of scope"
events:
  - mouse                              # ../../wasm-renderer/event-pipeline.md
  - key
  - focus
  - lifecycle
hooks: [useState, useEffect, useRef, useCallback, useMemo, useTarget]
limits:
  max_layout_nodes: 65535              # informational; no hard cap
animation:
  paint_loop: raf | poll | none        # raf=requestAnimationFrame; poll=fixed Hz; none=event-driven only
  css_transitions: true | false
```

### Profile entries

The full per-target matrix is in `target-profiles.yaml`. Highlights:

### Web profile

| Capability | Value | Source |
|------------|-------|--------|
| display_kinds | `[block, flex]` | `../../wasm-renderer/layout-runtime.md` (already verified). |
| color_depth | `rgba` | Canvas 2D over the web baseline. |
| text.shaping | `full` | `../../wasm-renderer/text-shaping.md`. |
| text.bidi | `true` | `../../wasm-renderer/line-bidi.md`. |
| input.pointer | `true` | Native click via `../../wasm-renderer/event-pipeline.md`. |
| input.touch | `true` (primary only) | Touch synthesizes into `mouse` SyntheticEvent. |
| events | `[mouse, key, focus, lifecycle]` | All four sources native. |
| animation.paint_loop | `raf` | Browser-native `requestAnimationFrame`. |

### Desktop profile

| Capability | Value | Source |
|------------|-------|--------|
| Inherits | web profile in full | Tauri wraps the web bundle (#1242). |
| Additions | `lifecycle.suspend` / `lifecycle.resume` from OS power events. | OS-shell signals only available on desktop. |

The desktop profile is **identical** to web for all paint/layout/text
concerns. Divergence is limited to OS-shell behavior (window chrome,
notifications, system tray, file dialogs) which is out of scope for
the renderer-neutral contract.

### TUI profile

| Capability | Value | Source |
|------------|-------|--------|
| display_kinds | `[block, flex]` | Same enum as web; cell-based metrics — see Degradation §T1. |
| color_depth | `ansi-16` (default) `\| indexed-256` (opt-in) | Terminal capability detection at mount time. |
| text.shaping | `grid` | Monospace cell metrics; no kerning, no ligatures, no proportional widths. |
| text.bidi | `false` (v1) | Deferred. RTL/BiDi text in TUI is its own design problem. |
| text.fonts | `terminal` | Whatever the user's terminal renders; spec MUST NOT specify family/weight. |
| input.pointer | `true` (where terminal supports mouse escape sequences) | Optional capability — falls back to keyboard-only. |
| input.touch | `false` | N/A. |
| events | `[mouse?, key, focus, lifecycle]` | `mouse` only when terminal supports it. |
| animation.paint_loop | `poll` (default 30 Hz) | No raf equivalent; ratatui draws on a timer. |
| animation.css_transitions | `false` | No frame budget for interpolation in v1. |

### Degradation rules

When a UI source uses a feature the target's profile does not include,
the substrate degrades per the rules below. Targets MUST NOT silently
drop or panic (C6).

### TUI degradation

| ID | Source feature | TUI handling | Justification |
|----|----------------|--------------|---------------|
| T1 | Pixel dimensions (`width: { px: 320 }`) | Round to nearest cell using a configurable cell-pixel ratio (default 8 px = 1 column, 16 px = 1 row). | TUI has cell-based geometry; px is meaningless without conversion. Rounding lets pixel-aware components keep working at low fidelity. |
| T2 | Percentage dimensions (`width: { pct: 50 }`) | Pass through unchanged — taffy already resolves against parent in cells. | No conversion needed; percentage is dimensionless. |
| T3 | Auto dimensions (`auto`) | Pass through unchanged. | Auto resolves to content size in cells; works natively. |
| T4 | RGBA color | Quantize to nearest of the 16 ANSI colors (or indexed-256 if opted in). | TUI lacks truecolor in the default profile. Quantization preserves intent at lower fidelity. |
| T5 | Font family / weight / style | Drop silently — TUI uses the terminal's font. | Terminal owns the font; per-element font props have no meaning. |
| T6 | Text shadow / box shadow | Drop silently. | No paint primitive. |
| T7 | Border radius | Round to 0 (square corners). | No paint primitive in cell-grid TUI. |
| T8 | CSS transitions / transforms | Drop silently. | `animation.css_transitions = false` in profile. |
| T9 | `mouse` events on a terminal without mouse support | Convert to nearest equivalent keyboard binding (Tab to focus + Enter to activate). | Keeps keyboard-driven Cue UI fully functional. |
| T10 | An element type the renderer doesn't paint (e.g., a hypothetical `<canvas>`) | Render a labeled placeholder (`[unsupported: canvas]`) with the element's `aria-label` if present. | Visible degradation > silent drop. |
| T11 | A `<markdown body=...>` body whose features exceed the active profile's `markdown` capability matrix (e.g., tables on TUI) | Per-feature drop: tables → `[table omitted]`, images → `aria-label` text or `[image]`, links → label + footnote-style URL. Inline styles / lists / code blocks always render. | TUI's grid text cannot draw tables or images; visible per-feature degradation preserves the rest of the document. |

### Web/desktop degradation

Web baseline and desktop are intentionally permissive — most degradation
rules don't apply because the canvas surface supports the full profile.
The two exceptions:

| ID | Source feature | Web/desktop handling |
|----|----------------|----------------------|
| W1 | A capability the source explicitly gates with `useTarget().is('tui')` | Skip the gated subtree on web/desktop. |
| W2 | A future TUI-only widget (e.g., a hypothetical `<TextField asciiOnly>`) | Render the standard `<TextField>` semantics; ignore the TUI-only props. |

### Markdown capability

The `markdown` capability matrix lets the build pipeline and app code
branch on which Markdown features the active renderer supports.
Schema (lives in `target-profiles.yaml.schema.json#/definitions/MarkdownCapability`):

```yaml
markdown:
  inline_styles: boolean   # bold, italic, code spans
  lists:         boolean   # bullet + ordered
  code_blocks:   boolean
  tables:        boolean
  images:        boolean
  links:         boolean   # clickable on web/desktop; label + footnote URL on TUI
```

Per-target defaults:

| Target | inline_styles | lists | code_blocks | tables | images | links |
|--------|---------------|-------|-------------|--------|--------|-------|
| web | true | true | true | true | true | true |
| desktop | inherits web | inherits | inherits | inherits | inherits | inherits |
| TUI | true | true | true | false | false | true (rendered as plain text + footnote URL) |

The `<markdown>` intrinsic in
[`element-contract.md`](./element-contract.md) §"`<markdown>` — formatted
text body" defers to this matrix; per-feature degradation is governed
by T11.

### TUI cadence sources

TUI deliberately has no `requestAnimationFrame` equivalent
(`animation.paint_loop = poll`). Primitives that animate (the
`<spinner>` intrinsic in [`element-contract.md`](./element-contract.md)
being the first) MUST be driven by an **app-owned cadence source** —
typically a `useEffect` + `setInterval` that bumps a state field
(`App.spinner_tick`), threaded into the primitive via its `tick` prop.

This keeps multi-spinner UIs phase-aligned and avoids per-primitive
animation timers fighting the poll loop. Web/desktop renderers MAY
ignore the cadence prop and animate natively; TUI renderers MUST honor
it.

### useTarget() hook

Cue components MAY query the active profile to gate optional UI:

```rust
let target = use_target();  // &'static TargetProfile
if target.is("tui") {
    // render keyboard-only path
}
if target.color_depth == ColorDepth::Rgba {
    // render the gradient version
}
```

| ID | Rule | Verifiable by |
|----|------|---------------|
| H1 | `use_target()` returns a `&'static TargetProfile` resolved at build time. The reference is stable across renders and re-renders MUST NOT cause the target to change. | Test: re-render a component 10x; profile pointer is unchanged. |
| H2 | The profile is read-only. Components MUST NOT attempt to mutate it. | Type system: the return is a `&'static`, not `&mut`. |
| H3 | The profile is fed by the build pipeline (#1239); the runtime cannot detect "current target" by other means. | Code search: no `cfg!(target = …)` direct reads in user component code. |

### Conformance vocabulary

Per element-contract §"Test strategy", the existing
`../../wasm-renderer/conformance.yaml` schema gains a `targets:` field:

```text
- id: S1_function_component_pure
  subset_rule: S1
  feature: Function component (pure render, no hooks)
  targets:
    web: verified
    desktop: inherits-web
    tui: verified
```

```text
- id: S6_inline_styles_color_rgba
  feature: Inline style color: rgba(...)
  targets:
    web: verified
    desktop: inherits-web
    tui: degraded                # quantized per T4
    tui_degradation_ref: T4
```

The `targets:` field schema lands in
`../../wasm-renderer/conformance.yaml.schema.json` as part of #1238.
The harness extension itself ships incrementally with #1241 (TUI
test entries) and #1242 (desktop confirmation).

### Out of scope

- Detecting target capability **at runtime** beyond the build-time
  profile (e.g., live capability negotiation with a terminal). The
  profile is static for the lifetime of a process.
- Defining new style props for TUI-specific concerns (e.g., a
  `tui-glyph` prop). The contract is designed to **subset** the
  existing style language, not extend it.
- Per-locale profile overrides (CJK width handling, etc.). Treated
  as a follow-up after the v1 contract stabilizes.
