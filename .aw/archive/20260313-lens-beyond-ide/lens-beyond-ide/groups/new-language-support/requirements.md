---
change: lens-beyond-ide
group: new-language-support
date: 2026-03-13
---

# Requirements

Add Go, Java, and C++ language support with tree-sitter grammars, lint checkers, and symbol builders.

1. Add tree-sitter grammars: tree-sitter-go, tree-sitter-java, tree-sitter-cpp to Cargo.toml
2. Extend Language enum and MultiParser with Go, Java, Cpp variants
3. Go lint checker: error handling (unchecked err), unused imports, shadowed variables, context.Background misuse, naked returns
4. Java lint checker: null checks, resource leaks (unclosed streams), raw types, empty catch blocks, string concatenation in loops
5. C++ lint checker: raw pointer usage, missing virtual destructors, include guard style, magic numbers, unchecked array access
6. Go symbol builder: functions, types, interfaces, methods, constants, packages
7. Java symbol builder: classes, interfaces, methods, fields, enums, annotations, packages
8. C++ symbol builder: classes, functions, namespaces, templates, macros, typedefs
9. Register all new checkers and symbol builders
10. Each language should have 8-10 initial lint rules
