---
change: mamba-features-305-316
date: 2026-02-14
---

# Clarifications

## Q1: Implementation Priority
- **Question**: What's the implementation priority order? P1-first, bottom-up, or all at once?
- **Answer**: All at once — design and implement all 12 features together as one cohesive system.
- **Rationale**: The features are interdependent (e.g., for-loops need iteration protocol, OOP needs import system, comprehensions need for-loops). A unified design avoids rework and ensures consistent architecture.

## Q2: Target Scope
- **Question**: What's the target scope — MVP, production-ready, or spec only?
- **Answer**: Production-ready — complete, tested, documented implementation of all 12 features.
- **Rationale**: User wants a fully functional Mamba compiler with all features working together, not partial implementations.

## Q3: Git Workflow
- **Question**: Which git workflow: new_branch, in_place, or worktree?
- **Answer**: in_place — work on the current branch (cclab-mamba).
- **Rationale**: Development is already on the cclab-mamba branch which is the dedicated feature branch for Mamba work.

