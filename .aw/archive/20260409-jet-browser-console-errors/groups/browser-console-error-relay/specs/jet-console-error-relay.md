---
id: jet-console-error-relay
main_spec_ref: crates/cclab-jet/logic/console-error-relay.md
merge_strategy: new
fill_sections: [overview, schema, changes]
filled_sections: [overview, schema, changes]
create_complete: true
---

# Jet Console Error Relay

## Overview

<!-- type: overview lang: markdown -->

Relays browser runtime errors and warnings to the Jet dev server terminal via the existing `/__jet_hmr` WebSocket. Three components:

| Component | Location | Responsibility |
|-----------|----------|----------------|
| Browser capture | Injected HMR client JS (`hmr_client.rs`) | Hook `window.onerror`, `window.onunhandledrejection`, `console.error()`, `console.warn()`; serialize and send upstream |
| Message protocol | Client-to-server JSON on `/__jet_hmr` WS | New `console-message` type (level, message, stack, url, line, column); separate from existing server-to-client `HmrMessage` enum |
| Server reception | `mod.rs` recv_task (currently drops non-Close frames) | Parse `console-message` frames, print formatted output to terminal (red for errors, yellow for warnings) |

| Constraint | Detail |
|------------|--------|
| Always-on | No config toggle; active whenever `jet dev` runs |
| One-way | Browser to server terminal only; no re-broadcast to other clients |
| Preserves originals | Hooked methods call through to the original `console.error` / `console.warn` after capture |
| Capture scope | `console.error`, `console.warn`, uncaught exceptions, unhandled rejections only; excludes `console.log` / `console.info` / `console.debug` |
## Requirements
<!-- type: requirements lang: markdown -->

<!-- TODO -->

## Scenarios
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

<!-- type: changes lang: markdown -->

### 1. `hmr.rs` — Add `ClientMessage` enum

```rust
// New: client-to-server message type
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientMessage {
    ConsoleReport {
        level: ConsoleLevel,
        message: String,
        stack: Option<String>,
        url: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
        timestamp: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleLevel {
    Error,
    Warn,
}
```

### 2. `mod.rs` — Extend `recv_task` in `hmr_websocket()`

Currently (lines 485-492) the recv_task ignores all non-Close frames:

```rust
// BEFORE
_ => {} // ignores incoming messages

// AFTER
axum::extract::ws::Message::Text(text) => {
    if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
        match msg {
            ClientMessage::ConsoleReport { level, message, stack, url, line, .. } => {
                let prefix = match level {
                    ConsoleLevel::Error => "\x1b[31m[browser error]\x1b[0m",
                    ConsoleLevel::Warn  => "\x1b[33m[browser warn]\x1b[0m",
                };
                eprintln!("{} {}", prefix, message);
                if let Some(u) = url {
                    if let Some(l) = line {
                        eprintln!("  at {}:{}", u, l);
                    }
                }
                if let Some(s) = stack {
                    for frame in s.lines().take(10) {
                        eprintln!("  {}", frame);
                    }
                }
            }
        }
    }
}
```

### 3. `hmr_client.rs` — Add browser-side capture hooks

Add after WebSocket connection is established (inside `setupWebSocket()`):

```javascript
// --- Console Error Relay ---
function setupConsoleRelay(ws) {
  function send(level, message, stack, url, line, column) {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type: 'console-report',
        level: level,
        message: String(message),
        stack: stack || null,
        url: url || null,
        line: typeof line === 'number' ? line : null,
        column: typeof column === 'number' ? column : null,
        timestamp: Date.now()
      }));
    }
  }

  // Hook console.error
  const origError = console.error;
  console.error = function(...args) {
    send('error', args.map(String).join(' '), new Error().stack);
    origError.apply(console, args);
  };

  // Hook console.warn
  const origWarn = console.warn;
  console.warn = function(...args) {
    send('warn', args.map(String).join(' '), new Error().stack);
    origWarn.apply(console, args);
  };

  // Hook uncaught exceptions
  window.addEventListener('error', (e) => {
    send('error', e.message, e.error?.stack, e.filename, e.lineno, e.colno);
  });

  // Hook unhandled promise rejections
  window.addEventListener('unhandledrejection', (e) => {
    const msg = e.reason instanceof Error ? e.reason.message : String(e.reason);
    const stack = e.reason instanceof Error ? e.reason.stack : null;
    send('error', 'Unhandled rejection: ' + msg, stack);
  });
}
```

Call `setupConsoleRelay(ws)` right after `ws.onopen` fires.
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->


## Schema

<!-- type: schema lang: json -->

### ClientMessage (browser → server)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ClientMessage",
  "description": "Messages sent from the browser HMR client to the dev server via WebSocket",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "type": { "const": "console-report" },
        "level": { "type": "string", "enum": ["error", "warn"] },
        "message": { "type": "string" },
        "stack": { "type": ["string", "null"] },
        "url": { "type": ["string", "null"] },
        "line": { "type": ["integer", "null"] },
        "column": { "type": ["integer", "null"] },
        "timestamp": { "type": "integer", "description": "Date.now() from browser" }
      },
      "required": ["type", "level", "message", "timestamp"]
    }
  ]
}
```

### Rust Types

```rust
/// Client-to-server messages (browser → dev server)
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientMessage {
    ConsoleReport {
        level: ConsoleLevel,
        message: String,
        stack: Option<String>,
        url: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
        timestamp: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleLevel {
    Error,
    Warn,
}
```

# Reviews
