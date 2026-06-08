---
change: orbit-task-primitives
date: 2026-01-31
---

# Clarifications

## Q1: Task References
- **Question**: How should task registry handle weak vs strong references to tasks?
- **Answer**: Weak references
- **Rationale**: Tasks can be garbage collected when no longer referenced externally, preventing memory leaks from abandoned tasks

## Q2: Notification Type
- **Question**: What completion notification mechanism should we use?
- **Answer**: mpsc channel
- **Rationale**: Simple and efficient, supports multiple consumers via cloning the sender, integrates well with Tokio async patterns

## Q3: WaitSet Dynamics
- **Question**: Should WaitSet support dynamic task addition after creation?
- **Answer**: Static only
- **Rationale**: Tasks fixed at creation leads to simpler implementation, matches Python asyncio.wait() semantics

