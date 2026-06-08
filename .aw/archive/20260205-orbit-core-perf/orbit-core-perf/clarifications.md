---
change: orbit-core-perf
date: 2026-02-05
---

# Clarifications

## Q1: Issue Scope
- **Question**: Which 5 orbit issues do you want to work on together?
- **Answer**: Core Performance (#58-61, #103) - GIL fixes, timer wheel, MPSC queue, io_uring
- **Rationale**: These are foundational optimizations that address the core performance bottlenecks in orbit. GIL contention is the most critical issue affecting Python integration.

## Q2: Git Workflow
- **Question**: What git workflow do you prefer for this change?
- **Answer**: In place - stay on current branch (pylibs)
- **Rationale**: Simpler workflow for iterative development, no branch switching overhead.

## Q3: Dependencies
- **Question**: How should we handle dependencies between these 5 issues?
- **Answer**: Sequential specs: #58 (GIL) → #60 (MPSC) → #59 (Timer) → #61 (Batching) → #103 (io_uring)
- **Rationale**: Sequential ordering ensures each optimization builds on the previous. GIL fix is foundational, MPSC queue enables efficient task dispatch, timer wheel improves scheduling, batching optimizes GIL acquisition patterns, and io_uring leverages all previous work.

## Q4: Platform
- **Question**: What's the target platform priority for io_uring (#103)?
- **Answer**: Linux-first with fallback to epoll on older kernels
- **Rationale**: io_uring provides significant performance gains on modern Linux (5.1+), but fallback ensures compatibility. macOS/Windows can use existing kqueue/IOCP backends.

