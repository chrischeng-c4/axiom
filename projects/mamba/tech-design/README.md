# mamba tech-design

Condensed implementation designs, authored by the orchestrating agent (root-cause
analysis, mechanism, fix pattern, verification contract — no code dumps).
Implementation agents (mamba-dev) implement from these documents; a dispatch
references one TD by path and the agent follows it instead of re-deriving the
analysis.

Layout:

- `logic/` — per-change design docs (one bounded WI's mechanism + contract).
- Filename: `<issue>-<slug>.md`, briefed by the name alone.

Each TD carries: **Mechanism** (why it breaks, 2-5 lines), **Invariant** (the
rule the fix must establish), **Fix pattern** (where + how, file:symbol level),
**Out of scope**, **Verification contract** (exact fixtures/probes + sweeps
that prove it). Keep it under one page — the value is density, not coverage.
