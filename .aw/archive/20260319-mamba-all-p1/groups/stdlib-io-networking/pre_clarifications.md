---
change: mamba-all-p1
group: stdlib-io-networking
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: No selectors module exists. socket_mod.rs and http_mod.rs exist as closest equivalents. Recommend selectors run on top of tokio I/O driver with async integration, since Mamba already has tokio runtime. Use tokio::net for non-blocking I/O rather than blocking the tokio thread.

### Q2: General
- **Answer**: No ssl module exists. Recommend rustls (pure Rust, no system dependency) for portability. No specific platform targets or certificate handling requirements identified. rustls integrates well with tokio via tokio-rustls.

### Q3: General
- **Answer**: Neither multiprocessing nor concurrent.futures exists. subprocess_mod.rs exists for process spawning. Recommend shared thread pool backend for both (rayon for CPU-bound, tokio for IO-bound). Implement most commonly used classes first: ThreadPoolExecutor, ProcessPoolExecutor, Future, as_completed(), wait() for concurrent.futures; Process, Pool for multiprocessing.

### Q4: General
- **Answer**: No urllib module exists. http_mod.rs is closest. Recommend urllib.request.urlopen as truly blocking (spawn OS thread via tokio::task::spawn_blocking) to match CPython semantics. Keep synchronous API; async HTTP should use asyncio/aiohttp pattern instead.

