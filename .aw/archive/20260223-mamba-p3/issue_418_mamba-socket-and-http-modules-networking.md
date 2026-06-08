---
number: 418
title: "mamba: socket and http modules (networking)"
state: open
labels: [enhancement, crate:mamba, P3]
dependencies: [405]
---

# #418 — mamba: socket and http modules (networking)

## Description

Implement networking modules for HTTP requests and socket communication.

## Requirements

### socket
- R1: `socket.socket(AF_INET, SOCK_STREAM)` — TCP socket
- R2: `.connect()`, `.bind()`, `.listen()`, `.accept()`
- R3: `.send()`, `.recv()`, `.close()`
- R4: `.settimeout()`

### http (urllib)
- R5: `urllib.request.urlopen(url)` — basic HTTP GET
- R6: `http.server.HTTPServer` — simple HTTP server (lower priority)
- R7: `http.client.HTTPConnection` / `HTTPSConnection`

## Dependencies

Depends on #405 (bytes type).

## Priority

P3 — needed for networked applications.
