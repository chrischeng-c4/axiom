/// <reference lib="DOM" />

// Auto-invoke wrapper for the web target. This is the file the html
// shell references via `<script type="module" src="./bootstrap.ts">`
// (compiled to .js by `jet build --target web` — #1239). Splitting
// the auto-invoke from `start_web()` keeps `index.ts` callable from
// non-DOM hosts (SSR shim, jsdom-based unit tests, future web-worker
// bootstrap) while still giving the html shell a single side-effect-
// only entry point.
//
// Spec: `cue-multi-target-slice.md` §"Slice 2c-4 — html shell +
// bootstrap entry". Closes the consumer-side half of AC1 of #1240
// alongside the html shell sibling — the build pipeline (jet's
// target=web pass) walks `index.html` → `bootstrap.ts` → `start_web`
// → `boot` → `paint_to_dom`, end-to-end.

import { start_web } from "./index";

window.addEventListener("DOMContentLoaded", () => {
  const root = document.getElementById("cue-app");
  if (!(root instanceof HTMLElement)) {
    throw new Error(
      "[cue] missing #cue-app mount node — index.html must contain " +
        "<div id=\"cue-app\"></div>",
    );
  }
  void start_web(root);
});
