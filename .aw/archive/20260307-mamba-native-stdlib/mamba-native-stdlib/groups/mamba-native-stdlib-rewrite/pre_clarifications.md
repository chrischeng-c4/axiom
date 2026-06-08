---
change: mamba-native-stdlib
group: mamba-native-stdlib-rewrite
date: 2026-03-07
status: answered
---

# Pre-Clarifications

### Q1: Priority
- **Answer**: Priority should be given to the most commonly used modules: json, re, sqlite3, datetime, os, sys, math, and time. These are currently either stubs or have very basic manual implementations that benefit most from a robust rewrite.

### Q2: Scope
- **Answer**: We will focus on the most commonly used modules first. While we should audit the entire stdlib/mod.rs list, the primary goal is to replace stubs and highly inefficient implementations with robust Rust crate-based versions.

### Q3: Crate Preferences
- **Answer**: We will use standard, production-grade crates: serde_json for JSON, regex for regular expressions, chrono for datetime/time, rusqlite for sqlite3, and standard Rust crates for other utilities (e.g., base64, sha2, rand).

### Q4: JIT Integration
- **Answer**: Yes, improving the JIT integration is part of this rewrite. We should aim to refactor symbols.rs to use a more automated registration system (e.g., via a macro) to reduce the manual overhead and potential for errors in the current string-based lookup system.

### Q5: Completeness
- **Answer**: For complex modules, we will aim for a functional subset that covers the majority (approx. 80%) of common use cases, ensuring stability and performance while avoiding the extreme complexity of 100% Python compatibility.

