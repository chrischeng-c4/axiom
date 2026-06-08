---
change: prism-init
date: 2026-01-27
---

# Clarifications

## Q1: Scope
- **Question**: What is the scope?
- **Answer**: Pre-initialize Prism handlers for registered projects when server starts
- **Rationale**: Issue #8 - improve first request response time

## Q2: Location
- **Question**: Where to add logic?
- **Answer**: In start_server() in http_server.rs
- **Rationale**: Before server accepts requests

## Q3: Error Handling
- **Question**: How to handle errors?
- **Answer**: Log warnings, dont fail startup
- **Rationale**: Server should still start

