// CueRuntime — the adapter between the pure reducer in `state.ts`
// and the effectful `CueBackend` (TUI direct call / desktop IPC /
// web HTTP). State changes flow one way: components dispatch → the
// runtime reduces (pure) → effects run side-band → backend ack
// dispatches back. This is the piece called out in the spec
// `cue-multi-target-slice.md` §"Command / query / event boundary"
// and in the comment at the top of `state.ts` ("effects live in the
// runtime adapter") — Slice 2c-1 of #1240 lands the actual module.
//
// Per-target boots wire it the same way:
//
//   const backend = new InMemoryCueBackend(...);   // or HTTP / IPC
//   setBackend(backend);
//   const rt = create_runtime(backend);
//   rt.subscribe((state) => render(App({ state, dispatch: rt.dispatch, ... })));
//   await rt.start();
//
// The runtime is a plain object — no React, no fiber, no DOM. Every
// target uses the exact same instance and only differs in the
// `render(...)` function that paints the returned `Element` tree.

import type { CueBackend } from "./backend";
import {
  INITIAL_STATE,
  reduce,
  type CueAction,
  type CueState,
} from "./state";

export interface CueRuntime {
  /** Current state snapshot. Components should read via `subscribe`. */
  state(): CueState;
  /** Dispatch an action. Triggers reducer + matching side-effect. */
  dispatch(action: CueAction): void;
  /**
   * Subscribe to state changes. Listener fires once immediately with
   * the current state, then on every subsequent change. Returns an
   * unsubscribe handle.
   */
  subscribe(listener: (state: CueState) => void): () => void;
  /**
   * Bootstrap: load the initial issues list and start the runtime
   * event subscription loop. Resolves once the initial fetch dispatch
   * lands; the event loop runs until `stop()`.
   */
  start(): Promise<void>;
  /** Tear down the event loop. Idempotent. */
  stop(): void;
}

export function create_runtime(backend: CueBackend): CueRuntime {
  let state: CueState = INITIAL_STATE;
  const listeners = new Set<(s: CueState) => void>();
  let stopped = false;

  const notify = (): void => {
    for (const l of listeners) l(state);
  };

  // Pure reducer pass — narrowly scoped so the side-effect dispatcher
  // below can call it without re-triggering its own effects.
  const reduce_only = (action: CueAction): void => {
    const next = reduce(state, action);
    if (next === state) return;
    state = next;
    notify();
  };

  // Side-effects keyed off the action that just ran. `submit_command`
  // and `get_issue` are fire-and-forget; their results dispatch back
  // through `reduce_only` so we don't re-enter the effect dispatcher.
  const run_effects = (action: CueAction): void => {
    switch (action.type) {
      case "command_submitted": {
        void backend.submit_command(action.cmd).finally(() => {
          if (!stopped) reduce_only({ type: "command_settled" });
        });
        return;
      }
      case "select_issue": {
        void backend
          .get_issue(action.id)
          .then((detail) => {
            if (!stopped) reduce_only({ type: "issue_loaded", detail });
          })
          .catch(() => {
            // Detail-fetch failures are non-fatal: the panel falls
            // back to "issue not loaded yet" and the user can retry
            // via re-selection. Surfacing through a dedicated error
            // action lands once the reducer grows error state.
          });
        return;
      }
      // No effect for these — pure state mutations.
      case "issues_loaded":
      case "issue_loaded":
      case "input_changed":
      case "command_settled":
      case "runtime_event":
        return;
    }
  };

  const dispatch = (action: CueAction): void => {
    if (stopped) return;
    reduce_only(action);
    run_effects(action);
  };

  return {
    state: () => state,
    dispatch,

    subscribe(listener) {
      listeners.add(listener);
      listener(state);
      return () => {
        listeners.delete(listener);
      };
    },

    async start() {
      try {
        const issues = await backend.list_issues();
        if (!stopped) reduce_only({ type: "issues_loaded", issues });
      } catch {
        // Initial-fetch failure: the UI shows an empty list. Same
        // rationale as the get_issue catch above — explicit error
        // state lands when the reducer grows it.
      }
      // Event loop runs until stop(). Detached so `start()` resolves
      // as soon as the initial fetch dispatches.
      void (async () => {
        try {
          for await (const event of backend.subscribe_events()) {
            if (stopped) break;
            reduce_only({ type: "runtime_event", event });
          }
        } catch {
          // Event-stream termination is normal at shutdown; non-fatal.
        }
      })();
    },

    stop() {
      stopped = true;
    },
  };
}
