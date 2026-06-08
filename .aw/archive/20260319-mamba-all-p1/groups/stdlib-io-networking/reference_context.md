---
change: mamba-all-p1
group: stdlib-io-networking
date: 2026-03-19
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| stdlib/network.md | cclab-mamba/stdlib | HIGH | Socket primitives (AF_INET, SOCK_STREAM/DGRAM, bind, listen, accept, connect, send, recv, close); HTTP client foundation (HTTPConnection, HTTPSConnection, request, getresponse, read); urllib.request foundation (urlopen simplified) — used as base for #661 SSL wrapper and #662 urllib full module |
| stdlib/concurrency.md | cclab-mamba/stdlib | HIGH | Threading primitives (Thread, Lock, Event); subprocess foundation (Popen, run, PIPE) — used as base for #664 multiprocessing (Process/Pool) and #665 concurrent.futures (ThreadPoolExecutor) |
| stdlib/native-implementations.md | cclab-mamba/stdlib | MEDIUM | General pattern for native stdlib module integration; error mapping from Rust crates to Mamba exceptions; symbol registration and module lifecycle — provides integration pattern for #663 email and other native stdlib modules |
| runtime/async.md | cclab-mamba/runtime | HIGH | Async runtime integration with Orbit event loop (mb_coroutine_suspend/resume); GIL-safe scheduling; Tokio integration for non-blocking I/O; Future interoperability with Tokio — enables #658 selectors high-level I/O multiplexing on async runtime |
| all-mamba-p0.md | cclab-mamba | LOW | Module system (R1 imports for new stdlib modules); runtime architecture context; multi-file compilation — provides foundation for registering and importing new stdlib modules |

# Coverage Analysis

## Issues → Specs Mapping

- **#658** (selectors module): Covered by runtime/async.md (async runtime foundation for tokio-based I/O multiplexing) + stdlib/network.md (socket primitives foundation)
- **#661** (ssl module): Covered by stdlib/network.md (socket and HTTP foundation for SSL wrapper) + stdlib/native-implementations.md (native error mapping pattern for ssl exceptions)
- **#662** (urllib module): Covered by stdlib/network.md (socket, HTTP, and simplified urllib.request foundation) + stdlib/native-implementations.md (native integration pattern for URL parsing and error handling)
- **#663** (email module): Covered by stdlib/native-implementations.md (integration pattern for native stdlib modules, error mapping, symbol registration)
- **#664** (multiprocessing module): Covered by stdlib/concurrency.md (subprocess_mod.rs and threading_mod.rs foundation for Process/Pool implementation) + all-mamba-p0.md (module system for registration)
- **#665** (concurrent.futures module): Covered by stdlib/concurrency.md (threading_mod.rs foundation for ThreadPoolExecutor) + runtime/async.md (async runtime for Tokio-backed executor) + all-mamba-p0.md (module system)

## Pre-Clarifications Coverage

- **Q1 (Selectors async I/O)**: runtime/async.md covers async runtime integration with Orbit/Tokio event loop (mb_coroutine_suspend/resume, GIL-safe scheduling); stdlib/network.md provides socket foundation for tokio::net integration
- **Q2 (SSL with rustls)**: stdlib/network.md covers socket and HTTP client foundation; stdlib/native-implementations.md provides error mapping pattern for SSL exceptions from rustls
- **Q3 (Multiprocessing + concurrent.futures thread pools)**: stdlib/concurrency.md covers subprocess and threading foundation; runtime/async.md supports Tokio-based async executor for concurrent.futures.ProcessPoolExecutor
- **Q4 (urllib with blocking semantics)**: stdlib/network.md covers socket and HTTP foundation; stdlib/native-implementations.md provides pattern for tokio::task::spawn_blocking wrapper

