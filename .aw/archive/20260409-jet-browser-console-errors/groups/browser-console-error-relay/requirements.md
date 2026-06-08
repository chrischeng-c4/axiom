---
change: jet-browser-console-errors
group: browser-console-error-relay
date: 2026-04-09
---

# Requirements

Add browser runtime error collection to the Jet dev server via the existing HMR WebSocket channel.

1. **Browser-side capture** (injected HMR client JS):
   - Hook `window.onerror` to capture uncaught exceptions
   - Hook `window.onunhandledrejection` to capture unhandled promise rejections
   - Intercept `console.error()` and `console.warn()` calls
   - Send structured JSON messages upstream via the existing `/__jet_hmr` WebSocket

2. **Server-side reception** (Rust dev server):
   - Parse incoming `console-error` messages in the WebSocket `recv_task` (currently ignores all incoming messages)
   - Print captured errors to the terminal with colored formatting (red for errors, yellow for warnings)
   - Include source file, line number, and stack trace when available

3. **Message protocol**:
   - Define a client-to-server message type for console errors (level, message, stack, url, line, column)
   - Keep it separate from the existing server-to-client `HmrMessage` enum

**Scope constraints:**
- Only capture `console.error`, `console.warn`, uncaught exceptions, unhandled rejections
- Do NOT capture `console.log` / `console.info` / `console.debug` (too noisy)
- One-way relay: browser → server terminal only (no re-broadcast to other clients)
- Preserve original console behavior (call original methods after capture)
