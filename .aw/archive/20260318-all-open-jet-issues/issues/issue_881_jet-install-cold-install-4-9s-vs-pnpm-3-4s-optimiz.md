---
number: 881
title: "jet-install: cold install 4.9s vs pnpm 3.4s — optimize metadata resolution"
state: open
labels: [enhancement, P1, crate:jet]
group: "jet-install-optimizations"
---

# #881 — jet-install: cold install 4.9s vs pnpm 3.4s — optimize metadata resolution

## Context

After parallel prefetch + symlink optimizations, jet install benchmarks:

| Scenario | jet | pnpm | npm |
|----------|-----|------|-----|
| Cold (no lockfile) | **4.9s** | 3.4s | 9.7s |
| Warm (lockfile, no nm) | **0.11s** | 1.1s | 1.7s |
| Hot (everything exists) | **0.03s** | 1.2s | 1.9s |

Warm/hot already wins. Cold install gap is **1.5s** vs pnpm.

## Analysis

Profile breakdown for cold 4.9s:
- `0.84s user` — Rust CPU (resolution + version matching)
- `0.48s system` — I/O (symlinks)
- **~3.6s network** — HTTP metadata fetches (~200 requests to registry.npmjs.org)

pnpm is faster because:
1. **Persistent metadata cache on disk** — pnpm caches registry metadata to disk (`~/.local/share/pnpm/metadata/`), so repeated installs across projects skip HTTP entirely
2. **HTTP/2 multiplexing** — pnpm uses undici with HTTP/2, multiplexing many requests over a single TCP connection
3. **Abbreviated metadata** — jet already uses `application/vnd.npm.install-v1+json` but responses are still ~50-200KB per package

## Proposed Optimizations

1. **Disk metadata cache** — Cache registry JSON responses to `~/.jet-store/.metadata/{name}.json` with TTL (e.g., 5 min). Cold install after first project = warm metadata.
2. **HTTP/2 with connection reuse** — Ensure reqwest uses `http2_prior_knowledge` or `http2_adaptive_window` for registry connections
3. **Speculative prefetch** — When resolving level N, speculatively prefetch metadata for common transitive deps (react → react-dom, scheduler)
4. **Parallel downloads during resolution** — Start downloading tarballs for already-resolved packages while still resolving remaining deps (pipeline overlap)

## Target

Cold install ≤ 3.0s (beat pnpm)
