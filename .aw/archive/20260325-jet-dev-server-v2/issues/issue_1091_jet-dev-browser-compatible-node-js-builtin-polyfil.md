---
number: 1091
title: "jet dev: browser-compatible Node.js builtin polyfills"
state: open
labels: [type:enhancement, priority:p2, crate:jet]
group: "jet-dev-server-v2"
---

# #1091 — jet dev: browser-compatible Node.js builtin polyfills

## Problem

Some npm packages import Node.js built-in modules (crypto, url, path, buffer, stream, etc.) which don't exist in browsers. When Jet dev server serves these, the browser fails with:

```
Failed to resolve module specifier "crypto"
```

Current workaround: empty stub files (`export default {};`) in `node_modules/.jet/`. These prevent the resolution error but any actual usage fails silently.

## Success Criteria

1. Common Node.js builtins have browser-compatible polyfills
2. Packages that import builtins for feature detection (not actual usage) work without errors
3. Packages that truly need Node APIs (fs, child_process) get a clear warning, not silent failure

## Boundary Conditions

**Should polyfill (browser equivalents exist):**
- `crypto` → `window.crypto` + `crypto.subtle` (Web Crypto API)
- `url` → `URL` + `URLSearchParams` (native browser APIs)
- `buffer` → `Uint8Array` or `buffer` polyfill
- `path` → `path-browserify` equivalent (pure JS)
- `events` → `EventEmitter` polyfill (pure JS)
- `util` → partial: `util.inspect`, `util.promisify`
- `querystring` → `URLSearchParams`
- `process` → `{ env: { NODE_ENV: 'development' }, browser: true }`
- `stream` → web streams API wrapper

**Should stub with warning (no browser equivalent):**
- `fs`, `child_process`, `cluster`, `net`, `tls`, `dgram`
- `worker_threads`, `v8`, `vm`, `os` (partial), `dns`

**Should not polyfill (never imported by browser packages):**
- `http`, `https`, `http2` — unless through packages like axios (which has browser adapter)

## Implementation

Generate polyfills at pre-bundle time in `node_modules/.jet/`:
1. On startup, check which builtins are actually imported by pre-bundled deps
2. Only generate polyfills for builtins that are referenced
3. Use minimal implementations (not full `browserify` polyfills)
4. Log warning for stub-only builtins: `[jet] Warning: 'fs' imported by 'some-package' — stubbed (no browser equivalent)`

## Test Cases

```
# crypto polyfill works
import crypto from 'crypto'
crypto.randomUUID() // → valid UUID string

# url polyfill works  
import { URL } from 'url'
new URL('https://example.com').hostname // → 'example.com'

# fs stub logs warning
import fs from 'fs'
fs.readFileSync // → undefined (stub)
```
