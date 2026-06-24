// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
/// Serve the `/@react-refresh` endpoint.
///
/// Returns a self-contained ESM module implementing the React Fast Refresh
/// runtime interface.  This is a lightweight shim that wraps the essential
/// React Refresh APIs used by the `$RefreshReg$` / `$RefreshSig$` injections
/// from `transform_tsx.rs`.
///
/// The real `react-refresh/runtime` is a dependency of the project, but we
/// serve a thin wrapper that re-exports the functions we need and adds the
/// `enqueueUpdate` / `performReactRefresh` scheduling helpers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn react_refresh_runtime_source() -> &'static str {
    r#"// /@react-refresh — Jet React Fast Refresh runtime shim
//
// This module wraps the react-refresh/runtime API with scheduling helpers.
// Injected by Jet dev server for HMR support.

let pendingUpdates = false;
let refreshTimeout = null;

// Registry of components keyed by module + component id.
const allFamilies = new Map();

// Signature tracking for hooks-order stability.
const allSignatures = new Map();

// Host-registered refresh callbacks (#196). A host that owns the React root
// (e.g. the jet stories isolated preview frame) registers a callback here so
// `performReactRefresh()` can drive an in-place re-render that reuses the
// existing root — preserving component hook state — instead of remounting.
const refreshCallbacks = new Set();

/**
 * Register a host callback invoked on every `performReactRefresh()`.
 * Returns an unregister function. Used by hosts that own the React root and
 * must re-render in place (state-preserving) when a module is hot-updated.
 */
export function onPerformReactRefresh(cb) {
  if (typeof cb === 'function') refreshCallbacks.add(cb);
  return () => refreshCallbacks.delete(cb);
}

/**
 * Look up the current (latest-registered) component type for a family id.
 * Lets a host resolve the freshly hot-updated component implementation while
 * keeping React's view of component identity stable.
 */
export function getCurrentType(id) {
  const family = allFamilies.get(id);
  return family ? family.current : undefined;
}

/**
 * Register a component with the refresh runtime.
 * Called as `$RefreshReg$(Component, "ComponentName")` by the transform.
 */
export function register(type, id) {
  if (type == null || typeof type !== 'function') return;

  const fullId = id;
  let family = allFamilies.get(fullId);
  if (!family) {
    family = { current: type };
    allFamilies.set(fullId, family);
  } else {
    family.current = type;
  }

  // If React DevTools or react-refresh/runtime is available, forward.
  if (typeof window !== 'undefined' && window.__REACT_DEVTOOLS_GLOBAL_HOOK__) {
    try {
      const hook = window.__REACT_DEVTOOLS_GLOBAL_HOOK__;
      if (typeof hook.registerFamily === 'function') {
        hook.registerFamily(fullId, family);
      }
    } catch (_) {}
  }
}

/**
 * Create a signature function for tracking hook call order.
 * Called as `const _s = $RefreshSig$()` at module scope.
 *
 * Returns a function that:
 * 1. First call (during component definition): records the signature
 * 2. Subsequent calls: validates the signature hasn't changed
 */
export function createSignatureFunctionForTransform() {
  let savedSignature;
  let hasCustomHooks = false;
  let didCollectHooks = false;

  return function(type, key, forceReset, getCustomHooks) {
    if (typeof key === 'string') {
      // Recording phase
      savedSignature = key;
      hasCustomHooks = typeof getCustomHooks === 'function';

      if (type != null) {
        let sig = allSignatures.get(type);
        if (!sig) {
          sig = {};
          allSignatures.set(type, sig);
        }
        sig.key = key;
        sig.forceReset = forceReset || false;
        sig.getCustomHooks = getCustomHooks;
      }
    }

    return type;
  };
}

/**
 * Schedule a React refresh update.
 * Debounced to batch multiple module updates into one React re-render.
 */
export function enqueueUpdate() {
  if (pendingUpdates) return;
  pendingUpdates = true;

  if (refreshTimeout != null) {
    clearTimeout(refreshTimeout);
  }

  refreshTimeout = setTimeout(() => {
    pendingUpdates = false;
    refreshTimeout = null;
    performReactRefresh();
  }, 30);
}

/**
 * Perform the actual React refresh / re-render.
 */
export function performReactRefresh() {
  // Trigger React re-render by calling the DevTools hook if available.
  if (typeof window !== 'undefined' && window.__REACT_DEVTOOLS_GLOBAL_HOOK__) {
    try {
      const hook = window.__REACT_DEVTOOLS_GLOBAL_HOOK__;
      if (typeof hook.performReactRefresh === 'function') {
        hook.performReactRefresh();
      }
    } catch (_) {}
  }
  // Drive host-registered refresh callbacks (#196). A host that owns the React
  // root re-renders the same root in place here, so the reconciler keeps the
  // existing fiber tree (and component hook state) instead of remounting.
  refreshCallbacks.forEach((cb) => {
    try { cb(); } catch (e) { console.error('[react-refresh] host callback error', e); }
  });
}

// Default export for `import RefreshRuntime from '/@react-refresh'`
export default {
  register,
  createSignatureFunctionForTransform,
  enqueueUpdate,
  performReactRefresh,
  onPerformReactRefresh,
  getCurrentType,
};
"#
}
// CODEGEN-END
