---
change: jet-hmr-validation
group: jet-hmr-validation
date: 2026-03-26
---

# Requirements

Add JavaScript module HMR to Jet dev server and validate with Conductor FE:

1. JS Module HMR (#1118): Extend existing /__jet_hmr WebSocket with JS update messages. Track module import graph server-side. On file change, send update message with module URL. Inject HMR client runtime into index.html that re-imports changed modules. Integrate React Fast Refresh ($RefreshReg$/$RefreshSig$ injection in transform_tsx). Support import.meta.hot API. HMR boundary detection — propagate to parent or full-reload if no boundary. Error overlay for syntax errors.

2. Conductor FE Validation (#1119): cclab jet install + cclab jet dev on projects/conductor/fe must work end-to-end. Dashboard renders, @cclab/ui components display, API proxy works, CSS/Tailwind renders, no console errors. Fix any remaining gaps found during validation.

Acceptance: Edit a React component in Conductor FE → component updates without full page reload, state preserved. CSS HMR continues to work. No Unexpected token errors in browser.
