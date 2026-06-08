---
number: 400
title: "feat(mamba): stdlib logging"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #400 — feat(mamba): stdlib logging

## Summary
Implement `logging` module for structured log output.

## Required
- `logging.debug/info/warning/error/critical(msg)`
- `logging.getLogger(name)` → Logger
- Logger methods: `.debug()`, `.info()`, `.warning()`, `.error()`, `.critical()`, `.setLevel()`
- `logging.basicConfig(level, format, filename)`
- Log levels: DEBUG, INFO, WARNING, ERROR, CRITICAL
- Handlers: StreamHandler, FileHandler (basic)
- Formatters (basic)

## Implementation Notes
- Use Rust `log` + `env_logger` crates or simple stderr output
- Global logger registry via HashMap<String, Logger>
