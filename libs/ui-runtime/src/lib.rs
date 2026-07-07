// HANDWRITE-BEGIN gap="missing-generator:ui-runtime-core" tracker="pending-tracker" reason="Shared component runtime extracted from jet-wasm for desktop/WASM renderer reuse."
//! Renderer-neutral component runtime: fiber tree + hooks + mount/flush loop.
//!
//! The runtime owns React-like authoring semantics without depending on React
//! DOM, a browser, WASM, AppKit, or any concrete renderer. Components render
//! `cclab_surface::Element` trees; host adapters decide whether those trees are
//! painted by Jet WASM WebGPU, a native desktop backend, or a test recorder.
//!
//! This is the middle layer between the UI element model and renderer backends:
//!
//! ```text
//! Component/hooks -> Element tree -> layout/paint/backend
//! ```

use cclab_surface::{Callback, Component, Element};
use std::cell::RefCell;
use std::rc::Rc;

// ── Fiber + hook storage ────────────────────────────────────────────────────

/// Per-component hook storage. The transpiler compiles each
/// `useState` / `useEffect` into a positional slot lookup that
/// points into this `Vec`. React's "rules of hooks" (call in the
/// same order every render) enforce the positional contract.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
#[derive(Default)]
pub(crate) struct Fiber {
    pub id: FiberId,
    pub hooks: Vec<HookSlot>,
    /// Cursor incremented by each hook call during a single render.
    /// Reset to 0 at the start of every render.
    pub hook_cursor: usize,
    /// Marked dirty by a state setter; the scheduler picks dirty
    /// fibers and re-renders them on the next commit.
    pub dirty: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct FiberId(pub u64);

/// A single slot in the hook list. Type-erased box so different
/// state types coexist in one Vec. The transpiler emits a cast on
/// each access because it knows the type from the surrounding TSX.
///
/// Variant = hook kind. A rules-of-hooks violation (conditional
/// hook call that re-orders slots across renders) produces a
/// variant mismatch at access time and panics with a clear message.
// @spec hooks-runtime#H6
pub(crate) enum HookSlot {
    /// `use_state`: the cell value.
    State(Box<dyn std::any::Any>),
    /// `use_memo` / `use_callback`: memoised value + last-seen deps.
    Memo {
        value: Box<dyn std::any::Any>,
        deps: Vec<MemoDepHash>,
    },
    /// `use_ref`: an `Rc<RefCell<T>>` behind a shared handle so
    /// clones in closures see mutations.
    Ref(Rc<RefCell<Box<dyn std::any::Any>>>),
    /// `use_context`: records which context the hook is watching so
    /// the runtime (future) can re-render on provider change. For
    /// now this is a no-op placeholder — the value is read from the
    /// active provider stack at render time, not stored.
    #[allow(dead_code)]
    Context,
    /// `use_effect_once`: a narrow effect slot for empty-deps effects.
    /// The effect is scheduled exactly once for the owning fiber.
    EffectOnce { ran: bool },
}

/// Deps array element for `use_memo` / `use_callback`. We hash deps
/// into `u64` at the call site so the slot doesn't need to carry
/// arbitrary types. The transpiler emits `hash_dep(x)` for each
/// dep the TSX source passes.
pub type MemoDepHash = u64;

// ── Thread-local scheduler / runtime ────────────────────────────────────────
//
// Single-threaded by design — a React app is a single-threaded render
// loop. When this ships to WASM the browser's main thread is our
// thread, and there's no possibility of concurrent renders.

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::default());
    static UPDATE_SCHEDULER: RefCell<Option<Rc<dyn Fn()>>> = RefCell::new(None);
}

#[derive(Default)]
struct Runtime {
    fibers: Vec<Fiber>,
    current: Option<FiberId>,
    next_id: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
impl Runtime {
    fn new_fiber(&mut self) -> FiberId {
        let id = FiberId(self.next_id);
        self.next_id += 1;
        self.fibers.push(Fiber {
            id,
            ..Default::default()
        });
        id
    }

    fn fiber_mut(&mut self, id: FiberId) -> &mut Fiber {
        self.fibers
            .iter_mut()
            .find(|f| f.id == id)
            .expect("fiber id not found — scheduler bug")
    }

    fn begin_render(&mut self, id: FiberId) {
        let f = self.fiber_mut(id);
        f.hook_cursor = 0;
        self.current = Some(id);
    }

    fn end_render(&mut self) {
        self.current = None;
    }
}

#[allow(dead_code)]
fn with_current_fiber<R>(f: impl FnOnce(&mut Fiber) -> R) -> R {
    // Kept around — `use_effect` (deferred) will use this instead of
    // inlining the RUNTIME.with dance. Silence dead_code until then.
    RUNTIME.with(|r| {
        let mut rt = r.borrow_mut();
        let id = rt
            .current
            .expect("hook called outside a render — rules-of-hooks violation");
        let fiber = rt.fiber_mut(id);
        f(fiber)
    })
}

// ── Public hooks ─────────────────────────────────────────────────────────────

/// `useState` — positional per-fiber state cell. Returns the current
/// value and a setter. The setter, when invoked, marks the owning
/// fiber dirty; the next commit re-renders it.
///
/// Matches React's semantics: setter accepts a new value OR a
/// function that receives the current value and returns the new
/// value. For v0 we only support the direct-value form — the
/// functional form lands when we implement `useReducer` (same
/// shape).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn use_state<T: Clone + 'static>(initial: T) -> (T, StateSetter<T>) {
    RUNTIME.with(|r| {
        let mut rt = r.borrow_mut();
        let id = rt
            .current
            .expect("hook called outside a render — rules-of-hooks violation");
        let fiber = rt.fiber_mut(id);
        let idx = fiber.hook_cursor;
        fiber.hook_cursor += 1;
        if fiber.hooks.len() <= idx {
            fiber.hooks.push(HookSlot::State(Box::new(initial)));
        }
        let HookSlot::State(any_value) = &fiber.hooks[idx] else {
            panic!(
                "hook slot {idx} type mismatch: expected State, got another kind — \
                 rules-of-hooks violation OR transpiler bug"
            );
        };
        let value: &T = any_value
            .downcast_ref()
            .expect("hook slot type mismatch — rules-of-hooks violation OR transpiler bug");
        let value = value.clone();
        let setter = StateSetter {
            fiber_id: id,
            idx,
            _marker: std::marker::PhantomData,
        };
        (value, setter)
    })
}

/// Setter handle returned from `use_state`. Clone-friendly so it can
/// be moved into event-handler closures; updating schedules a
/// re-render of the owning fiber.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub struct StateSetter<T: Clone + 'static> {
    fiber_id: FiberId,
    idx: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Clone + 'static> Clone for StateSetter<T> {
    fn clone(&self) -> Self {
        Self {
            fiber_id: self.fiber_id,
            idx: self.idx,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Clone + 'static> StateSetter<T> {
    /// Write a new value. Marks the owning fiber dirty. Re-render
    /// happens on the next `flush_updates` call.
    pub fn set(&self, new_value: T) {
        RUNTIME.with(|r| {
            let mut rt = r.borrow_mut();
            let fiber = rt.fiber_mut(self.fiber_id);
            fiber.hooks[self.idx] = HookSlot::State(Box::new(new_value));
            fiber.dirty = true;
        });
        notify_update_scheduled();
    }
}

/// Narrow `useEffect(..., [])` primitive for generated WASM code.
///
/// This intentionally handles only the empty-deps shape. The
/// transpiler uses it for side effects that start browser host
/// capabilities such as `fetch`; dependency-aware reruns will use a
/// richer effect slot later.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn use_effect_once<F: FnOnce() + 'static>(effect: F) {
    let should_run = RUNTIME.with(|r| {
        let mut rt = r.borrow_mut();
        let id = rt
            .current
            .expect("hook called outside a render — rules-of-hooks violation");
        let fiber = rt.fiber_mut(id);
        let idx = fiber.hook_cursor;
        fiber.hook_cursor += 1;
        if fiber.hooks.len() <= idx {
            fiber.hooks.push(HookSlot::EffectOnce { ran: false });
        }
        let HookSlot::EffectOnce { ran } = &mut fiber.hooks[idx] else {
            panic!(
                "hook slot {idx} type mismatch: expected EffectOnce, got another kind — \
                 rules-of-hooks violation OR transpiler bug"
            );
        };
        if *ran {
            false
        } else {
            *ran = true;
            true
        }
    });

    if should_run {
        effect();
    }
}

// ── use_reducer ────────────────────────────────────────────────────────────

/// Dispatch handle returned from `use_reducer`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub struct DispatchHandle<S: Clone + 'static, A: 'static> {
    fiber_id: FiberId,
    idx: usize,
    reducer: Rc<dyn Fn(&S, A) -> S>,
    _marker: std::marker::PhantomData<(S, A)>,
}

impl<S: Clone + 'static, A: 'static> Clone for DispatchHandle<S, A> {
    fn clone(&self) -> Self {
        Self {
            fiber_id: self.fiber_id,
            idx: self.idx,
            reducer: self.reducer.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S: Clone + 'static, A: 'static> DispatchHandle<S, A> {
    pub fn dispatch(&self, action: A) {
        RUNTIME.with(|r| {
            let mut rt = r.borrow_mut();
            let fiber = rt.fiber_mut(self.fiber_id);
            let HookSlot::State(any_value) = &fiber.hooks[self.idx] else {
                panic!(
                    "hook slot {} type mismatch: expected State (reducer), got another kind",
                    self.idx
                );
            };
            let current: &S = any_value
                .downcast_ref()
                .expect("reducer slot type mismatch — rules-of-hooks OR transpiler bug");
            let new_state = (self.reducer)(current, action);
            fiber.hooks[self.idx] = HookSlot::State(Box::new(new_state));
            fiber.dirty = true;
        });
        notify_update_scheduled();
    }
}

/// Register the renderer-side async update scheduler. State setters
/// are framework-level primitives, but only the mounted renderer knows
/// how to coalesce dirty fibers into an actual frame.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn set_update_scheduler(scheduler: Option<Rc<dyn Fn()>>) {
    UPDATE_SCHEDULER.with(|slot| {
        *slot.borrow_mut() = scheduler;
    });
}

fn notify_update_scheduled() {
    UPDATE_SCHEDULER.with(|slot| {
        if let Some(schedule) = slot.borrow().as_ref().cloned() {
            schedule();
        }
    });
}

/// `useReducer` — same slot as `useState`, but transitions driven
/// by a pure reducer. Reducer is stable across renders.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn use_reducer<S: Clone + 'static, A: 'static, F: Fn(&S, A) -> S + 'static>(
    reducer: F,
    initial: S,
) -> (S, DispatchHandle<S, A>) {
    RUNTIME.with(|r| {
        let mut rt = r.borrow_mut();
        let id = rt
            .current
            .expect("hook called outside a render — rules-of-hooks violation");
        let fiber = rt.fiber_mut(id);
        let idx = fiber.hook_cursor;
        fiber.hook_cursor += 1;
        if fiber.hooks.len() <= idx {
            fiber.hooks.push(HookSlot::State(Box::new(initial)));
        }
        let HookSlot::State(any_value) = &fiber.hooks[idx] else {
            panic!("hook slot {idx} type mismatch: expected State (reducer), got another kind");
        };
        let state: &S = any_value
            .downcast_ref()
            .expect("reducer slot type mismatch — rules-of-hooks OR transpiler bug");
        (
            state.clone(),
            DispatchHandle {
                fiber_id: id,
                idx,
                reducer: Rc::new(reducer),
                _marker: std::marker::PhantomData,
            },
        )
    })
}

// ── use_ref ────────────────────────────────────────────────────────────────

/// Persistent mutable cell that survives re-renders. Mutating a
/// ref does NOT trigger a re-render.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub struct RefHandle<T: 'static> {
    cell: Rc<RefCell<Box<dyn std::any::Any>>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> Clone for RefHandle<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Clone + 'static> RefHandle<T> {
    pub fn current(&self) -> T {
        let b = self.cell.borrow();
        b.downcast_ref::<T>()
            .expect("RefHandle type mismatch — transpiler bug")
            .clone()
    }

    pub fn set(&self, new_value: T) {
        let mut b = self.cell.borrow_mut();
        *b = Box::new(new_value);
    }
}

impl<T: 'static> RefHandle<T> {
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut b = self.cell.borrow_mut();
        let t = b
            .downcast_mut::<T>()
            .expect("RefHandle type mismatch — transpiler bug");
        f(t)
    }
}

/// `useRef` — stable mutable container across renders.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn use_ref<T: 'static>(initial: T) -> RefHandle<T> {
    RUNTIME.with(|r| {
        let mut rt = r.borrow_mut();
        let id = rt
            .current
            .expect("hook called outside a render — rules-of-hooks violation");
        let fiber = rt.fiber_mut(id);
        let idx = fiber.hook_cursor;
        fiber.hook_cursor += 1;
        if fiber.hooks.len() <= idx {
            let cell: Rc<RefCell<Box<dyn std::any::Any>>> =
                Rc::new(RefCell::new(Box::new(initial)));
            fiber.hooks.push(HookSlot::Ref(cell));
        }
        let HookSlot::Ref(cell) = &fiber.hooks[idx] else {
            panic!(
                "hook slot {idx} type mismatch: expected Ref, got another kind — \
                 rules-of-hooks OR transpiler bug"
            );
        };
        RefHandle {
            cell: cell.clone(),
            _marker: std::marker::PhantomData,
        }
    })
}

// ── use_memo / use_callback ────────────────────────────────────────────────

/// `useMemo` — recompute `compute()` only when `deps` change.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn use_memo<T: Clone + 'static, F: FnOnce() -> T>(compute: F, deps: Vec<MemoDepHash>) -> T {
    RUNTIME.with(|r| {
        let mut rt = r.borrow_mut();
        let id = rt
            .current
            .expect("hook called outside a render — rules-of-hooks violation");
        let fiber = rt.fiber_mut(id);
        let idx = fiber.hook_cursor;
        fiber.hook_cursor += 1;
        let needs_recompute = match fiber.hooks.get(idx) {
            None => true,
            Some(HookSlot::Memo { deps: prev, .. }) => prev != &deps,
            Some(_) => panic!("hook slot {idx} type mismatch: expected Memo, got another kind"),
        };
        if needs_recompute {
            let value = compute();
            let slot = HookSlot::Memo {
                value: Box::new(value),
                deps,
            };
            if fiber.hooks.len() <= idx {
                fiber.hooks.push(slot);
            } else {
                fiber.hooks[idx] = slot;
            }
        }
        let HookSlot::Memo { value, .. } = &fiber.hooks[idx] else {
            unreachable!("just populated the slot above");
        };
        value
            .downcast_ref::<T>()
            .expect("memo slot type mismatch — transpiler bug")
            .clone()
    })
}

/// `useCallback` — stable-identity callback that rebinds iff deps
/// change. Sugar over `use_memo`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn use_callback<P: Clone + 'static, F: Fn(P) + 'static>(
    f: F,
    deps: Vec<MemoDepHash>,
) -> Callback<P> {
    use_memo(move || Callback::new(f), deps)
}

/// Hash any `Hash + ?Sized` value into a `MemoDepHash`. The
/// transpiler emits `hash_dep(x)` per dep expression.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn hash_dep<H: std::hash::Hash + ?Sized>(v: &H) -> MemoDepHash {
    use std::hash::{DefaultHasher, Hasher};
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

// ── Public mount / event / flush API ────────────────────────────────────────

/// Mount point — runs a component once and returns its initial
/// rendered tree + a handle for subsequent event dispatch and
/// updates.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn mount(component: Component) -> MountHandle {
    let fiber_id = RUNTIME.with(|r| r.borrow_mut().new_fiber());
    let tree = render_fiber(fiber_id, component.clone());
    MountHandle {
        fiber_id,
        component,
        tree: RefCell::new(tree),
    }
}

/// Returned from `mount`. Holds the live fiber + the last rendered
/// tree so tests / the WebGPU renderer can inspect it.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub struct MountHandle {
    pub fiber_id: FiberId,
    pub component: Component,
    pub tree: RefCell<Element>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
impl MountHandle {
    /// Returns a clone of the currently-mounted element tree.
    pub fn snapshot(&self) -> Element {
        self.tree.borrow().clone()
    }

    /// Synchronously flush any pending state updates. Re-runs the
    /// component function for each dirty fiber and replaces this
    /// handle's tree if the root was affected. Returns whether a
    /// re-render happened.
    pub fn flush(&self) -> bool {
        let re_render = RUNTIME.with(|r| {
            let rt = r.borrow();
            rt.fibers.iter().any(|f| f.id == self.fiber_id && f.dirty)
        });
        if !re_render {
            return false;
        }
        let new_tree = render_fiber(self.fiber_id, self.component.clone());
        *self.tree.borrow_mut() = new_tree;
        RUNTIME.with(|r| {
            let mut rt = r.borrow_mut();
            rt.fiber_mut(self.fiber_id).dirty = false;
        });
        true
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
impl MountHandle {
    /// Debug-only: force the root fiber dirty so the next `flush()`
    /// re-renders even when no state changed. Used by `JetDebug::force_rerender`.
    #[cfg(feature = "debug")]
    pub fn mark_root_dirty(&self) {
        RUNTIME.with(|r| {
            let mut rt = r.borrow_mut();
            rt.fiber_mut(self.fiber_id).dirty = true;
        });
    }
}

/// Debug-only summary of a fiber's storage. Feature-gated to keep
/// the `pub(crate)` visibility of `Fiber` intact — we expose just
/// enough shape for `JetDebug` to serialize.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
#[cfg(feature = "debug")]
pub struct DebugFiberMeta {
    pub id: u64,
    pub hook_count: usize,
    pub dirty: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
#[cfg(feature = "debug")]
pub fn debug_snapshot_fibers() -> Vec<DebugFiberMeta> {
    RUNTIME.with(|r| {
        let rt = r.borrow();
        rt.fibers
            .iter()
            .map(|f| DebugFiberMeta {
                id: f.id.0,
                hook_count: f.hooks.len(),
                dirty: f.dirty,
            })
            .collect()
    })
}

/// One hook slot's value rendered for debug. `value_json` is `None`
/// when the runtime can't cheaply read it (non-primitive `State`,
/// `Memo` / `Ref` body, or `Context` placeholder).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
#[cfg(feature = "debug")]
pub struct DebugHookSummary {
    pub kind: &'static str,
    pub type_name: Option<&'static str>,
    pub value_json: Option<serde_json::Value>,
}

/// Read every hook slot for `fiber_id` and render a debug summary.
/// Returns an empty Vec if the fiber doesn't exist (rather than
/// panicking — `jet browser hooks <bogus-id>` should be a gentle
/// error, not a crash).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
#[cfg(feature = "debug")]
pub fn debug_snapshot_hooks(fiber_id: u64) -> Vec<DebugHookSummary> {
    RUNTIME.with(|r| {
        let rt = r.borrow();
        let Some(fiber) = rt.fibers.iter().find(|f| f.id.0 == fiber_id) else {
            return Vec::new();
        };
        fiber
            .hooks
            .iter()
            .map(|slot| match slot {
                HookSlot::State(any) => {
                    let (type_name, value_json) = summarize_any(any.as_ref());
                    DebugHookSummary {
                        kind: "State",
                        type_name,
                        value_json,
                    }
                }
                HookSlot::Memo { value, .. } => {
                    let (type_name, value_json) = summarize_any(value.as_ref());
                    DebugHookSummary {
                        kind: "Memo",
                        type_name,
                        value_json,
                    }
                }
                HookSlot::Ref(_) => DebugHookSummary {
                    kind: "Ref",
                    type_name: None,
                    value_json: None,
                },
                HookSlot::Context => DebugHookSummary {
                    kind: "Context",
                    type_name: None,
                    value_json: None,
                },
                HookSlot::EffectOnce { ran } => DebugHookSummary {
                    kind: "EffectOnce",
                    type_name: Some("bool"),
                    value_json: Some(serde_json::json!(*ran)),
                },
            })
            .collect()
    })
}

/// Try a small chain of primitive downcasts. Returns (type_name,
/// value_json) — `type_name` is `None` only when Any's concrete type
/// can't even be named (shouldn't happen for boxed Anys).
#[cfg(feature = "debug")]
fn summarize_any(v: &dyn std::any::Any) -> (Option<&'static str>, Option<serde_json::Value>) {
    use serde_json::json;
    if let Some(x) = v.downcast_ref::<i64>() {
        return (Some("i64"), Some(json!(*x)));
    }
    if let Some(x) = v.downcast_ref::<i32>() {
        return (Some("i32"), Some(json!(*x)));
    }
    if let Some(x) = v.downcast_ref::<u64>() {
        return (Some("u64"), Some(json!(*x)));
    }
    if let Some(x) = v.downcast_ref::<u32>() {
        return (Some("u32"), Some(json!(*x)));
    }
    if let Some(x) = v.downcast_ref::<f64>() {
        return (Some("f64"), Some(json!(*x)));
    }
    if let Some(x) = v.downcast_ref::<f32>() {
        return (Some("f32"), Some(json!(*x)));
    }
    if let Some(x) = v.downcast_ref::<bool>() {
        return (Some("bool"), Some(json!(*x)));
    }
    if let Some(x) = v.downcast_ref::<String>() {
        return (Some("String"), Some(json!(x.clone())));
    }
    if let Some(x) = v.downcast_ref::<&'static str>() {
        return (Some("&'static str"), Some(json!(*x)));
    }
    if v.downcast_ref::<()>().is_some() {
        return (Some("()"), Some(json!(null)));
    }
    // Unknown concrete type — return just the type_id form so the UI
    // can at least tell the user something. std::any::type_name::<T>()
    // requires knowing T at compile time, which we don't here; best
    // we can do without type reflection is "unknown".
    (Some("<unknown primitive chain missed>"), None)
}

fn render_fiber(fiber_id: FiberId, component: Component) -> Element {
    RUNTIME.with(|r| r.borrow_mut().begin_render(fiber_id));
    let tree = (component.render)(&component.props);
    RUNTIME.with(|r| r.borrow_mut().end_render());
    tree
}
// HANDWRITE-END
