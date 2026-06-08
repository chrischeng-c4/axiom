// Target-agnostic boot scaffold. Every per-target entry (web /
// desktop / TUI) calls `boot(...)` with two things only: a
// `CueBackend` impl that knows how to talk to its transport, and a
// `paint(element)` function that knows how to put the returned
// `Element` tree on screen. Everything in between — `setBackend`,
// `create_runtime`, the render-on-change subscription, the initial
// `start()` fetch — is identical across targets and lives here.
//
// Spec: `cue-multi-target-slice.md` §"Command / query / event
// boundary" + the per-target boot pattern documented in
// `runtime.ts`. Slice 2c-2 of #1240 — closes the structural half of
// AC1 ("built against ≥1 target") by giving every target a single
// canonical boot entry; the actual paint impls are per-target
// follow-up slices that swap their `paint` argument for a real DOM
// walker / ratatui frame / Tauri webview hook without touching this
// file or `App.tsx`.

import { App } from "./components/App";
import { create_runtime, type CueRuntime } from "./runtime";
import { setBackend } from "./hooks";
import type { CueBackend } from "./backend";
import type { CueState } from "./state";
import type { LifecycleState } from "./protocol";
import type { Element } from "./jsx";

export interface BootConfig {
  /** Per-target CueBackend implementation. Registered before mount. */
  readonly backend: CueBackend;
  /**
   * Per-target paint function. Receives the `Element` tree from
   * `App({ state, dispatch, lifecycle })` on every state change.
   * Implementations: web → DOM walker; desktop → Tauri webview hook;
   * TUI → ratatui frame buffer. Pure projection — must not mutate
   * `state` or call `dispatch` (`App` already wires interactions).
   */
   readonly paint: (element: Element) => void;
  /**
   * Optional warning shown on the StatusBar (e.g. config validation
   * hint surfaced from the backend at boot). Forwarded into
   * `AppProps.status_warning`.
   */
  readonly status_warning?: string;
}

/**
 * Boot the Cue UI on a target. Resolves with the live `CueRuntime`
 * once the initial `list_issues` fetch dispatches; the runtime keeps
 * pumping events until the caller invokes `runtime.stop()`.
 *
 * Convention: per-target entry files are tiny. They construct the
 * concrete `CueBackend`, supply the matching `paint` impl, and call
 * `boot(...)`. They do NOT import `App` or `create_runtime`
 * directly — those are routed through here so the wiring stays
 * uniform across targets and any future change (e.g. adding a
 * config-warning surface or a global key handler) lands in one
 * place.
 */
export async function boot(config: BootConfig): Promise<CueRuntime> {
  const { backend, paint, status_warning } = config;

  setBackend(backend);
  const rt = create_runtime(backend);

  rt.subscribe((state: CueState) => {
    paint(
      App({
        state,
        dispatch: rt.dispatch,
        lifecycle: derive_lifecycle(state),
        status_warning,
      }),
    );
  });

  await rt.start();
  return rt;
}

// Lifecycle is a derived view over reducer state — kept here (not
// in the reducer) because it's a render-time concern and the
// reducer stays minimal. "running" while a command is in flight,
// otherwise "idle". `done` / `error` lifecycles arrive when the
// reducer grows the corresponding fields.
function derive_lifecycle(state: CueState): LifecycleState {
  return state.pending_cmd === null ? "idle" : "running";
}
