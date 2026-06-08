/** @jsx createElement */
// First TSX component on the cue-app data layer — proves the JSX
// runtime + per-component Props type + element vocabulary all
// typecheck end-to-end against the Slice 2 contract.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "StatusBar"). Tag composition:
//   `Box(direction=horizontal){ Text(lifecycle_glyph) +
//      Text(status) + Text(warning?, accent=warn) }`
//
// Slice 2b-1 anchor: this component intentionally renders the
// minimum that exercises `IntrinsicProps` + conditional children +
// component prop typing. App.tsx / NavRail.tsx / DetailPane.tsx and
// the rest of the catalog land in follow-up sub-slices on top of the
// JSX runtime.

import { createElement, type Element } from "../jsx";
import type { LifecycleState } from "../protocol";
import type { StatusBarProps } from "../components.types";

// Stable per-state glyph table — matches the existing
// `cue-tui-lifecycle-ux.md` design tokens. Inlined here (vs. imported
// from a tokens module) so the component compiles in isolation; the
// shared tokens module lands once it has more than one consumer.
const LIFECYCLE_GLYPH: { readonly [K in LifecycleState]: string } = {
  idle: "○",
  running: "●",
  done: "✔",
  error: "✗",
};

export function StatusBar(props: StatusBarProps): Element {
  return (
    <box className="status-bar" style="direction: horizontal">
      <text className="lifecycle-glyph">{LIFECYCLE_GLYPH[props.lifecycle]}</text>
      <text className="status">{props.status}</text>
      {props.config_warning ? (
        <text className="warn">{props.config_warning}</text>
      ) : null}
    </box>
  );
}
