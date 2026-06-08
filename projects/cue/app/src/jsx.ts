// JSX runtime for the cue-app — lightweight `Element` type that
// mirrors `jet_wasm::Element` so TSX components in this package
// `tsc --noEmit` clean against the same vocabulary the Rust-side
// renderer resolves at build time.
//
// Spec: `.aw/tech-design/projects/cue/cue-multi-target-slice.md`
// §"Slice 2b" + the Jet element contract referenced from there
// (`projects/jet/multi-target/element-contract.md`).
//
// The TSX → Rust transpiler (`ts_to_rust`) consumes the shape of
// `Element` by name; the runtime body of `createElement` only matters
// for non-WASM TS unit tests + future web-target hot-path. Either way
// the public surface is a stable factory + a stable discriminated
// union — anything else is a refactor target inside this file.

export interface IntrinsicProps {
  readonly className?: string;
  readonly style?: string;
  readonly id?: string;
  readonly onClick?: () => void;
  readonly onChange?: (next: string) => void;
  // Open extension point. Per-component typed Props (StatusBarProps,
  // NavRailProps, …) live in `components.types.ts` and travel
  // through `createElement` as the second arg; the index signature
  // is the catch-all for one-off TSX attributes that haven't been
  // hoisted into a typed slot yet. Strict tag-by-tag prop maps land
  // once the renderer-contract gaps surfaced in #1246 are filed.
  readonly [extra: string]: unknown;
}

// Component invocation — `createElement(MyComponent, props, ...kids)`.
// Generic so component authors can declare typed props; the unknown
// fallback at the createElement signature keeps the factory itself
// monomorphic.
export type ComponentFn<P = unknown> = (props: P) => Element;

// Discriminated union mirroring the five Rust `Element` variants.
// `kind` is the discriminant; tagged for serde-style switch
// exhaustiveness in the renderer.
export type Element =
  | {
      readonly kind: "intrinsic";
      readonly tag: string;
      readonly props: IntrinsicProps;
      readonly children: ReadonlyArray<Element>;
    }
  | { readonly kind: "text"; readonly value: string }
  | {
      readonly kind: "component";
      readonly name: string;
      readonly render: ComponentFn<unknown>;
      readonly props: unknown;
    }
  | { readonly kind: "fragment"; readonly children: ReadonlyArray<Element> }
  | { readonly kind: "empty" };

export const EMPTY: Element = { kind: "empty" };

// Symbol-typed Fragment marker so JSX `<></>` round-trips through the
// transpiler's `jsxFragmentFactory: "Fragment"` setting. Symbol over
// string so user code can't shadow it accidentally.
export const Fragment: unique symbol = Symbol.for("jet.cue.fragment");
export type FragmentTag = typeof Fragment;

// Loose child input — TSX hands us strings, numbers, raw Elements,
// nested arrays from `.map()`, and the boolean / null / undefined
// noise that conditional render expressions emit. We normalize to a
// flat `Element[]` here so downstream consumers (Rust transpiler,
// renderer, snapshot tests) see exactly one shape.
export type ChildInput =
  | Element
  | string
  | number
  | boolean
  | null
  | undefined
  | ReadonlyArray<ChildInput>;

function flatten_children(input: ReadonlyArray<ChildInput>): Element[] {
  const out: Element[] = [];
  for (const c of input) {
    if (c === null || c === undefined || c === false || c === true) {
      // JSX renders `false`/`null`/`undefined` as nothing; explicit
      // `true` is the same here per the React convention.
      continue;
    }
    if (typeof c === "string") {
      out.push({ kind: "text", value: c });
    } else if (typeof c === "number") {
      out.push({ kind: "text", value: String(c) });
    } else if (Array.isArray(c)) {
      for (const sub of flatten_children(c)) {
        out.push(sub);
      }
    } else {
      out.push(c as Element);
    }
  }
  return out;
}

// Single TSX factory entry-point. Three call shapes:
//   - Intrinsic: `createElement("box", props, ...children)`
//   - Component: `createElement(MyComponent, props, ...children)`
//   - Fragment:  `createElement(Fragment, null, ...children)`
//
// The `props` arg is `IntrinsicProps | null | undefined` for intrinsic
// + fragment calls; component invocations carry their typed props bag
// and we re-attach the flattened children under the `children` key so
// component bodies can access them uniformly.
export function createElement(
  tag: string | FragmentTag | ComponentFn<unknown>,
  props: IntrinsicProps | Record<string, unknown> | null | undefined,
  ...children: ChildInput[]
): Element {
  const flat = flatten_children(children);
  if (tag === Fragment) {
    return { kind: "fragment", children: flat };
  }
  if (typeof tag === "function") {
    const merged: Record<string, unknown> = { ...(props ?? {}), children: flat };
    return {
      kind: "component",
      name: tag.name || "anonymous",
      render: tag,
      props: merged,
    };
  }
  return {
    kind: "intrinsic",
    tag,
    props: (props as IntrinsicProps | null | undefined) ?? {},
    children: flat,
  };
}

// Global JSX namespace so `<box>`, `<text>`, etc. typecheck without
// having to enumerate every Jet element here. `IntrinsicElements`
// uses an open index signature (the renderer-vocabulary still has
// gaps per #1246's contract-mapping spec); typed slots come once the
// four primitive-gap issues — Spinner / Modal+focus-trap /
// ActionRow / Markdown capability — are filed and resolved.
declare global {
  namespace JSX {
    type Element = import("./jsx").Element;
    interface IntrinsicElements {
      readonly [tag: string]: IntrinsicProps;
    }
  }
}
