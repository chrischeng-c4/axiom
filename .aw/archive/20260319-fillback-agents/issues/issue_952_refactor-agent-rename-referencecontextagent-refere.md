---
number: 952
title: "refactor(agent): Rename ReferenceContextAgent → ReferenceSpecContextAgent"
state: open
labels: [enhancement, crate:agent, P1]
group: "fillback-agents"
---

# #952 — refactor(agent): Rename ReferenceContextAgent → ReferenceSpecContextAgent

## Summary

Rename existing ReferenceContextAgent to ReferenceSpecContextAgent to distinguish from ReferenceCodebaseContextAgent.

## Changes
- Rename struct: ReferenceContextAgent → ReferenceSpecContextAgent
- Rename file: reference_context.rs → reference_spec_context.rs
- Update mod.rs, lib.rs exports
- Update PyO3 bindings
- Update specs and docs
