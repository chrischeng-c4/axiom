---
change: 1136
group: platform-config-and-merge-git
date: 2026-04-03
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should auto_commit stage only the 3 known directories (specs/, changes/, archive/) or detect all dirty paths under cclab/?
- **Answer**: Detect all dirty paths under cclab/. Stage everything that changed within the cclab/ directory.

### Q2: General
- **Question**: For auto_pr, should the PR body be auto-generated from user_input.md and spec summaries, or just a minimal template?
- **Answer**: Dispatch an agent to write the PR body. Most flexible, can produce high-quality PR descriptions from the change context.

### Q3: General
- **Question**: Should repo_platform.repo be required or default to the same value as issue_platform.repo when omitted?
- **Answer**: Required. repo_platform.repo must be explicitly set — no fallback to issue_platform.repo. Keeps the config explicit even if it means writing the repo twice.

