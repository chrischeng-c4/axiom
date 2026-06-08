/// <reference lib="DOM" />

// Real `paint(element)` for the web target — a minimal DOM walker
// that materializes the Cue `Element` tree under a caller-supplied
// root node and replaces it on every state change. Naive replace-
// children strategy is intentional: this is the foundation slice
// (Slice 2c-3 of #1240, AC1) — diffing / keyed reconciliation lands
// only when a measured render becomes the bottleneck. The walker is
// pure projection: it never reads `state` or calls `dispatch`; the
// boot scaffold (`boot.ts`) hands a fresh tree on every change.
//
// Triple-slash `lib="DOM"` is scoped to this file (and the web
// target entry) — the rest of `@cue/app` keeps `lib: ES2020` only
// so DOM access cannot leak into target-agnostic app code. Spec:
// `cue-multi-target-slice.md` §"Per-target boot pattern" + the
// element-contract reference therein.

import type { Element as CueElement } from "../../jsx";

export function paint_to_dom(root: HTMLElement, tree: CueElement): void {
  root.replaceChildren(...materialize(tree));
}

function materialize(tree: CueElement): Node[] {
  switch (tree.kind) {
    case "empty":
      return [];
    case "text":
      return [document.createTextNode(tree.value)];
    case "fragment":
      return tree.children.flatMap(materialize);
    case "component":
      return materialize(tree.render(tree.props));
    case "intrinsic":
      return [build_intrinsic(tree.tag, tree.props, tree.children)];
  }
}

function build_intrinsic(
  tag: string,
  props: { readonly [k: string]: unknown },
  children: ReadonlyArray<CueElement>,
): HTMLElement {
  const el = document.createElement(tag);

  for (const [key, value] of Object.entries(props)) {
    apply_prop(el, key, value);
  }

  for (const node of children.flatMap(materialize)) {
    el.appendChild(node);
  }

  return el;
}

// Prop application is deliberately small and explicit — the catalog
// in `IntrinsicProps` is itself a short closed-ish list, and the
// only "magic" surface (event handlers) is two names. New typed
// slots from the renderer-contract gaps (#1246) will land as
// branches here, not as a generic dispatch table.
function apply_prop(el: HTMLElement, key: string, value: unknown): void {
  if (value === undefined || value === null || value === false) return;

  switch (key) {
    case "className":
      if (typeof value === "string") el.className = value;
      return;
    case "style":
      if (typeof value === "string") el.setAttribute("style", value);
      return;
    case "id":
      if (typeof value === "string") el.id = value;
      return;
    case "onClick":
      if (typeof value === "function") {
        el.addEventListener("click", value as () => void);
      }
      return;
    case "onChange":
      if (typeof value === "function") {
        const handler = value as (next: string) => void;
        el.addEventListener("input", (ev) => {
          const target = ev.target as HTMLInputElement | null;
          if (target && typeof target.value === "string") handler(target.value);
        });
      }
      return;
    default:
      if (typeof value === "string") el.setAttribute(key, value);
      else if (value === true) el.setAttribute(key, "");
      return;
  }
}
