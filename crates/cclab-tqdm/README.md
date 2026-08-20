# Cclab Tqdm

## Brief

Cclab Tqdm is the Rust progress-tracking API surface for cclab crates.

It wraps `indicatif` behind a small public contract for progress bars,
spinners, multi-progress containers, style templates, rate/ETA reporting, and
typed progress errors. The current verification level is API behavior smoke;
terminal rendering and visual regression coverage remain product-readiness gaps.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Progress Tracking API | - | Progress bar, spinner, multi-progress, style, and error APIs with behavior smoke proof |

### Progress Tracking API

Cclab Tqdm exposes Rust progress tracking primitives for progress bars,
spinners, multi-progress rendering, style templates, rate/ETA calculation, and
typed progress errors.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API:
  `cclab_tqdm::{ProgressBar, MultiProgress, ProgressStyle, TqdmError}`
- Gate — behavior: `cargo test -p cclab-tqdm` - progress bar, spinner,
  multi-progress, style, rate, and typed-error behavior smoke
- Gate: `cargo test -p cclab-tqdm`
- Evidence: `cargo test -p cclab-tqdm`; crates/cclab-tqdm/src/bar.rs
