---
change: auto-gen-python-cclab
date: 2026-01-30
---

# Clarifications

## Q1: Scan Method
- **Question**: How should multi-file scanning be implemented?
- **Answer**: Recursive directory scan - scan all .rs files recursively in crate, merge all PyO3 entities
- **Rationale**: This approach captures all PyO3 exports regardless of how they're organized, more robust than parsing register_module()

## Q2: Output Scope
- **Question**: What is the target output scope?
- **Answer**: Per-module stubs - generate __init__.pyi for each module (titan, nebula, etc.)
- **Rationale**: Matches Python module structure, easier to maintain and import

## Q3: Git Workflow
- **Question**: Which git workflow do you prefer?
- **Answer**: In place - stay on current branch genesis/auto-gen-python-cclab
- **Rationale**: Already on the correct branch, no need to create new branch or worktree

