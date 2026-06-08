---
number: 389
title: "feat(mamba): stdlib re (regular expressions)"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #389 — feat(mamba): stdlib re (regular expressions)

## Summary
Implement Python `re` module for regular expression support.

## Required Functions
- `re.match(pattern, string)`, `re.search(pattern, string)`
- `re.findall(pattern, string)`, `re.finditer(pattern, string)`
- `re.sub(pattern, repl, string)`, `re.subn()`
- `re.split(pattern, string)`
- `re.compile(pattern)` → Pattern object
- Match object: `.group()`, `.groups()`, `.start()`, `.end()`, `.span()`
- Flags: `re.IGNORECASE`, `re.MULTILINE`, `re.DOTALL`

## Implementation Notes
- Use Rust `regex` crate as backend
- Match objects as MbObject with captured groups
