// App-level hooks for the Cue UI. The single hook in Slice 2b-2 is
// `useBackend()` — the boundary the spec
// `cue-multi-target-slice.md` §"Command / query / event boundary"
// makes mandatory: app code never calls `fetch()` / `invoke()` /
// any target-specific IPC API. Per-target wiring (TUI direct call,
// desktop Tauri IPC, web HTTP) registers ITS `CueBackend` impl via
// `setBackend(...)` once at mount; every component then reads it
// via `useBackend()`.
//
// Slice 2b-2 ships a module-level singleton — the simplest binding
// that satisfies the contract on every target. A jet-side context
// provider (matching React's `<Context.Provider>`) is the natural
// upgrade path once jet hooks gain that primitive; the public
// shape of `useBackend()` does NOT change when that lands.

import type { CueBackend } from "./backend";

// Module-level slot. `null` means "no backend registered yet" —
// `useBackend()` will throw rather than return `null`, so the
// calling component's caller stack pinpoints the missing-mount bug
// loud-fast.
let registered: CueBackend | null = null;

/**
 * Register the active `CueBackend` implementation. Each target's
 * entry point (TUI bootstrap / Tauri main / web bundle entry)
 * calls this exactly once before any component mounts. Calling
 * `setBackend(...)` a second time replaces the previous instance
 * — supports test reset and live reload.
 */
export function setBackend(backend: CueBackend): void {
  registered = backend;
}

/**
 * Read the registered `CueBackend`. Throws when no backend has
 * been registered, because every other failure mode (silent
 * `null`, lazy-resolve, sentinel object) makes the missing-mount
 * bug surface only on the next async call — by which point the
 * stack is unrelated to the mount path.
 */
export function useBackend(): CueBackend {
  if (registered === null) {
    throw new Error(
      "useBackend(): no CueBackend registered. " +
        "Call setBackend(...) at the target entry point before mount.",
    );
  }
  return registered;
}

/**
 * Reset the module-level slot back to the unregistered state.
 * Test-only escape hatch: production targets register exactly once
 * and never tear down. Exported (rather than a `__test__` hidden
 * key) because `@cue/app` is a private package — every consumer
 * is in this repo and the surface stays small.
 */
export function clearBackend(): void {
  registered = null;
}
