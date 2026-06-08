---
number: 840
title: "Error diagnostics quality — rich compiler messages with suggestions"
state: open
labels: [enhancement, P1, crate:mamba]
group: "compiler-infrastructure"
---

# #840 — Error diagnostics quality — rich compiler messages with suggestions

## Summary

Improve Mamba's error diagnostics to match the quality of Rust compiler messages. Current diagnostics have spans and basic messages but lack the polish needed for developer adoption.

## Proposed Improvements

### Rich context
```
error[E0308]: type mismatch
  --> src/main.py:12:5
   |
12 |     x: int = "hello"
   |              ^^^^^^^ expected int, found str
   |
help: did you mean to parse the string?
   |
12 |     x: int = int("hello")
   |              +++        +
```

### Specific features
- **Color output**: ANSI colored terminal output (error in red, warning in yellow)
- **Underline spans**: Wavy underlines for error locations
- **Fix suggestions**: "did you mean X?" for common mistakes
- **Related information**: Show where a type was inferred or where a variable was defined
- **Error codes**: Categorized error codes (E0001, E0002...) with `--explain` flag
- **Multi-span errors**: Show both sides of a type mismatch

## Implementation

Consider using the `ariadne` or `miette` crate for rendering, or enhance the existing `diagnostic` module.
