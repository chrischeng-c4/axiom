/** @jsx createElement */
// CostFooter — optional turn-cost summary anchored at the App's
// bottom edge.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "CostFooter"). Tag composition:
//   `Box(direction=horizontal){ Text(in/out tokens) + Text(usd, dim) }`
//
// `summary` is `?` because the protocol field on the Rust side
// (`App.cost_summary`) is planned but not yet shipped. Renderer skips
// this surface when summary is undefined — matches the spec's
// "optional, plays a no-op until the field lands" contract.

import { createElement, type Element, EMPTY } from "../jsx";
import type { CostFooterProps } from "../components.types";

export function CostFooter(props: CostFooterProps): Element {
  const { summary } = props;
  if (!summary) {
    return EMPTY;
  }
  return (
    <box className="cost-footer" style="direction: horizontal">
      <text className="tokens">
        {`in: ${summary.in_tokens} / out: ${summary.out_tokens}`}
      </text>
      <text className="usd dim">{`$${summary.usd.toFixed(4)}`}</text>
    </box>
  );
}
