---
change: mamba-all-p1
group: stdlib-introspection
date: 2026-03-19
---

# Requirements

Implement 3 native stdlib modules that expose Mamba's compiler internals to userspace:
- #667 dis: `dis()`, `disassemble()`, `Instruction`, `get_instructions()` — wrap Mamba's MIR/bytecode IR as Python-accessible objects
- #668 ast: `parse()`, `dump()`, `literal_eval()`, `NodeVisitor`, `NodeTransformer` — wrap Mamba's parser output as a Python AST tree
- #669 tokenize: `generate_tokens()`, `TokenInfo`, token type constants — wrap Mamba's lexer output as Python token stream
All three require public Rust API surface from the parser/lexer/IR layers to be accessible from the stdlib native module boundary.
