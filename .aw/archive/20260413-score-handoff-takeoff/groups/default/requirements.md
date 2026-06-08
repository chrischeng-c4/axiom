---
change: score-handoff-takeoff
group: default
date: 2026-04-13
source: structured-issue
---

# Requirements

## Problem

R1: Sessions break when switching context mid-flight — there is no structured, CLI-native way to capture what was done, what was found, and what comes next. The existing `/handoff` skill writes to `/tmp/` with 6 bespoke sections and has no CLI backing; documents are ephemeral, inconsistently located, and not auto-verifiable.

R2: Resuming work requires manually hunting for the last handoff document and re-reading it without any automated criteria verification, leading to wasted context and missed steps.

## Requirements

