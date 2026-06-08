// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
// CODEGEN-BEGIN
//! jet-wasm — framework-agnostic browser-runtime substrate.
//!
//! Two layers:
//!
//! - **Generic substrate** (this file + `renderer`):
//!   - `Element` tree, `Props`, `Callback`, `Component` types that
//!     any framework-specific runtime uses.
//!   - `renderer` module: layout + paint + canvas backend.
//!   - Nothing here is React-specific. Vue / Angular / Solid /
//!     vanilla-JS adapters would target the same `Element` shape.
//!
//! - **Framework-specific runtimes** (feature-gated modules):
//!   - `react` (default feature): fiber + hooks + mount/flush
//!     commit loop. Target of jet's TSX → Rust transpiler.
//!   - `vue` (future): reactivity + templates.
//!   - `angular` (future): signals + zones.
//!   - `solid` (future): fine-grained reactivity.
//!
//! The framework-specific modules are **adapters over the same
//! generic substrate** — they all produce `Element` trees the
//! renderer consumes. Adding a new framework means adding a new
//! module here + a new compiler front-end in `jet::*_to_rust`.
//!
//! For the transpiler view of the world, a TSX function component
//! like:
//!
//! ```tsx
//! function Counter() {
//!   const [n, setN] = useState(0);
//!   return <button onClick={() => setN(n + 1)}>{n}</button>;
//! }
//! ```
//!
//! lowers to Rust that uses `jet_wasm::react::use_state` + builds
//! a `jet_wasm::Element` tree. Vue code would use
//! `jet_wasm::vue::ref` + build the same `Element` shape. Only
//! the reactive plumbing differs; the paint pipeline is shared.
//!
//! Deferred (scoped to later cycles):
//! - `use_effect` (needs async executor).
//! - Reconciliation diffing — current commit rebuilds the tree.
//! - Context, Suspense, refs, memo, error boundaries, concurrent mode.
//! - Taffy flexbox, rustybuzz text shaping, a11y shadow tree.
//! - Vue / Angular / Solid adapters.

pub mod renderer;

/// React-compat binding manifest — `jet.declare.d.ts` parsing + defaults.
///
/// @spec .aw/tech-design/projects/jet/wasm-renderer/binding-manifest.md
///
/// Public surface for the module's import-resolver consumers (the
/// transpiler will import these types when `jet-tsx-to-rust` lands as
/// a Phase 1 deliverable). This crate owns parsing + overlay-merge
/// only; transpiler emit semantics live in `transpiler.md`.
pub mod manifest;

#[cfg(feature = "host-bridge")]
pub mod host;

/// Text shaping engine — rustybuzz integration.
///
/// @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md
///
/// Phase 6a: glyph shaping + per-paragraph cache + paint-runtime
/// integration boundary. Line breaking, bidi, selection, IME, and
/// clipboard are deferred to Phase 6b–6f follow-up issues.
pub mod text;

#[cfg(feature = "react")]
pub mod react;

#[cfg(feature = "debug")]
pub mod debug;

use std::rc::Rc;

// ── Element tree ────────────────────────────────────────────────────────────

/// A rendered element. Either an intrinsic host node (`<div>`, `<button>`,
/// etc. — eventually mapped to canvas paint ops by the renderer), a text
/// leaf, or a component invocation that the runtime will expand.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
#[derive(Clone)]
pub enum Element {
    /// An intrinsic HTML-shaped node — tag + props + children.
    Intrinsic {
        tag: &'static str,
        props: Props,
        children: Vec<Element>,
    },
    /// A text leaf.
    Text(String),
    /// A component invocation; the runtime will call `render` to get
    /// the actual sub-tree during mount / update.
    Component(Component),
    /// An empty / noop element. Used for conditional `null`/`undefined`
    /// in JSX.
    Empty,
    /// Transparent container for a dynamic list of Elements —
    /// produced by TSX `{arr.map(...)}`. Layout + paint treat it as
    /// if its children were spliced into the parent's child list.
    /// The renderer walks through without emitting a node of its own.
    Fragment(Vec<Element>),
}

/// Constructor-style helpers mirror the common TSX patterns. The
/// transpiler emits these calls directly.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
impl Element {
    pub fn intrinsic(tag: &'static str, props: Props, children: Vec<Element>) -> Self {
        Self::Intrinsic {
            tag,
            props,
            children,
        }
    }

    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    pub fn from_number<N: std::fmt::Display>(n: N) -> Self {
        Self::Text(n.to_string())
    }
}

/// Component = {render fn, props, stable type id for reconciliation}.
/// Props are type-erased here; the transpiler emits a typed shim per
/// component that reads props back out.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
#[derive(Clone)]
pub struct Component {
    pub name: &'static str,
    pub render: ComponentFn,
    pub props: Rc<dyn std::any::Any>,
}

pub type ComponentFn = fn(&Rc<dyn std::any::Any>) -> Element;

/// Props — a small typed bag. Limited to the subset real TSX code
/// emits so far; will grow. Events carry a `Callback` that captures
/// a `StateSetter` so the runtime can schedule re-renders.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
#[derive(Clone, Default, Debug)]
pub struct Props {
    pub class_name: Option<String>,
    pub style: Option<String>,
    pub on_click: Option<Callback<()>>,
    pub on_change: Option<Callback<String>>,
    pub on_checked_change: Option<Callback<bool>>,
    pub id: Option<String>,
    pub value: Option<String>,
    /// @spec .aw/tech-design/projects/jet/specs/4072.md#schema
    pub input_type: Option<String>,
    pub placeholder: Option<String>,
    /// @spec .aw/tech-design/projects/jet/specs/4072.md#schema
    pub checked: Option<bool>,
    /// @spec .aw/tech-design/projects/jet/specs/4072.md#schema
    pub aria_label: Option<String>,
    pub disabled: bool,
}

/// Event callback — typed by the payload it receives. Cloneable so
/// the runtime can hand it to the event dispatcher without moving.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
#[derive(Clone)]
pub struct Callback<P: Clone>(Rc<dyn Fn(P)>);

impl<P: Clone> std::fmt::Debug for Callback<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<callback>")
    }
}

impl<P: Clone> Callback<P> {
    pub fn new<F: Fn(P) + 'static>(f: F) -> Self {
        Self(Rc::new(f))
    }
    pub fn call(&self, payload: P) {
        (self.0)(payload);
    }
}

// ── Element-tree walking (for tests + the future renderer) ──────────────────

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
impl Element {
    /// Depth-first collect every `on_click` callback in the tree.
    /// The canvas renderer will do richer hit-testing; tests use
    /// this for round-trip verification.
    pub fn find_on_click(&self, target_id: &str) -> Option<Callback<()>> {
        match self {
            Element::Intrinsic {
                props, children, ..
            } => {
                if props.id.as_deref() == Some(target_id) {
                    return props.on_click.clone();
                }
                for c in children {
                    if let Some(cb) = c.find_on_click(target_id) {
                        return Some(cb);
                    }
                }
                None
            }
            Element::Fragment(children) => {
                for c in children {
                    if let Some(cb) = c.find_on_click(target_id) {
                        return Some(cb);
                    }
                }
                None
            }
            Element::Component(c) => {
                // Components produce a tree; for this walker we assume
                // already-rendered output. Component leaves in the
                // rendered tree mean we're inspecting the pre-render
                // output, which is a test-only path.
                let _ = c;
                None
            }
            _ => None,
        }
    }

    /// Concatenate all text descendants in order. Used by tests.
    pub fn text_content(&self) -> String {
        match self {
            Element::Text(s) => s.clone(),
            Element::Intrinsic { children, .. } => {
                children.iter().map(|c| c.text_content()).collect()
            }
            Element::Fragment(children) => children.iter().map(|c| c.text_content()).collect(),
            _ => String::new(),
        }
    }
}
// CODEGEN-END
