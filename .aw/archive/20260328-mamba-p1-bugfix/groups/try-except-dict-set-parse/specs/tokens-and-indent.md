---
id: tokens-and-indent
type: spec
title: "Tokens and Indentation"
version: 1
spec_type: algorithm
files:
  - lexer/mod.rs
  - lexer/token.rs
  - lexer/indent.rs
main_spec_ref: "crates/mamba/lexer/tokens-and-indent.md"
merge_strategy: extend
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Tokens and Indentation

## Overview

This change-spec reviews the Mamba lexer's handling of `{`/`}` tokens (LBrace/RBrace) in context of issue #1112: dict/set literals inside `try` blocks triggering parse errors.

### Analysis

| Aspect | Finding |
|--------|---------|
| IndentProcessor `{` at line start | Correct — computes indent and emits INDENT/DEDENT **before** incrementing `paren_depth` (R2 step 2) |
| Implicit line continuation for `{...}` | Correct — newlines inside braces suppressed per R2 step 2 |
| Token stream for `try:\n    d = {}['x']\nexcept:` | Correct — emits `INDENT` at column 4, pairs `{}`/`[]` with balanced paren_depth, `DEDENT` before `except` |
| Token stream for `try:\n    {}\nexcept:` | Correct — `{` at line start triggers indent calc first, then enters paren mode |

### Conclusion

The IndentProcessor (R2) correctly handles all brace scenarios. **No lexer code changes required.** The bug root cause is in the parser layer (`parse_block` / `parse_try` in `stmt.rs` / `stmt_compound.rs`), which misinterprets the correctly-produced token stream.

### Scope

- **Confirmed correct**: `IndentProcessor.process()` in `indent.rs` — brace handling order (indent check → paren_depth increment)
- **Clarification added**: R2 brace-at-line-start ordering guarantee (see Changes section)
- **No file modifications**: Lexer code is unchanged
## Requirements

### R1 - Tokenization of Python 3.12 Syntax

```yaml
id: R1
priority: high
```

The lexer must recognize all Python 3.12 token kinds via `TokenKind` (derived with `logos::Logos`):

| Category      | Tokens                                                                 |
|---------------|------------------------------------------------------------------------|
| Keywords      | `def`, `return`, `if`, `elif`, `else`, `while`, `for`, `in`, `class`, `enum`, `match`, `case`, `import`, `from`, `as`, `and`, `or`, `not`, `True`, `False`, `None`, `pass`, `break`, `continue`, `self`, `try`, `except`, `finally`, `raise`, `with`, `async`, `await`, `yield`, `lambda`, `del`, `assert`, `global`, `nonlocal`, `is`, `type` |
| Type keywords | `int`, `float`, `bool`, `str`, `list`, `dict`, `tuple`                |
| Literals      | `Int(i64)`, `Float(f64)`, `Complex(f64)`, `Str`, `TripleStr`, `FStr`, `RawStr`, `ByteStr`, `Ellipsis` |
| Operators     | `+`, `-`, `*`, `/`, `//`, `%`, `**`, `=`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `->`, `\|`, `?`, `+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `**=`, `&`, `^`, `~`, `<<`, `>>`, `&=`, `\|=`, `^=`, `<<=`, `>>=`, `:=`, `@`, `@=` |
| Delimiters    | `(`, `)`, `[`, `]`, `{`, `}`, `:`, `,`, `.`, `;`                      |
| Synthetic     | `Indent`, `Dedent`, `Eof`, `Newline`, `Comment`, `Ident`              |

The `Token` struct stores `kind: TokenKind`, `start: u32`, `end: u32` (byte offsets). Integer literals support hex (`0x`), octal (`0o`), binary (`0b`), and underscore separators. Float literals support scientific notation (`1e10`).

### R2 - INDENT/DEDENT Generation

```yaml
id: R2
priority: high
```

The `IndentProcessor` converts raw tokens into an indentation-aware stream:

1. Maintains an `indent_stack` (initialized to `[0]`) and `paren_depth` counter.
2. Newlines inside parentheses/brackets/braces are suppressed (implicit line continuation).
3. At each line start (first non-whitespace token after a newline), computes the indent level as the column offset from the preceding newline.
4. If indent > current top: push level, emit `INDENT`.
5. If indent < current top: pop levels and emit `DEDENT` for each until the stack matches.
6. At EOF, emit remaining `DEDENT` tokens to close all open blocks, then emit `EOF`.
7. Comments are stripped entirely from the output.

### R3 - PEP 701 f-string Lexing

```yaml
id: R3
priority: medium
```

F-strings are lexed as a single `FStr(String)` token containing the raw content between `f"` and `"` (or `f'` and `'`). The lexer captures the template text including `{expr}` placeholders as-is. The parser is responsible for splitting the content into literal and expression parts. Both single and double quote delimiters are supported.

### R4 - String Literal Lexing

```yaml
id: R4
priority: high
```

The lexer supports the full range of Python string literal forms:

| Form         | Prefix | Token       | Escape Processing |
|--------------|--------|-------------|-------------------|
| Regular      | none   | `Str`       | Yes               |
| Triple-quoted| none   | `TripleStr` | Yes (multi-line)  |
| F-string     | `f`    | `FStr`      | Deferred to parser|
| Raw          | `r`    | `RawStr`    | No                |
| Byte         | `b`    | `ByteStr`   | Yes               |

Triple-quoted strings use custom callback functions (`lex_triple_dquote`, `lex_triple_squote`) that scan for the closing `"""` or `'''`, respecting backslash escapes within the body.

## Acceptance Criteria

### Scenario: Tokenize Simple Expression

- **WHEN** `lex_raw("x + 42", file_id)` is called.
- **THEN** the result contains tokens `[Ident, Plus, Int(42)]` with correct byte offsets.

### Scenario: INDENT/DEDENT from Nested Blocks

- **WHEN** the following source is lexed:
  ```
  if True:
      x = 1
      if True:
          y = 2
  ```
- **THEN** the token stream contains two `INDENT` tokens (at column 4 and column 8), and two `DEDENT` tokens at EOF to close both blocks.

### Scenario: Implicit Line Continuation

- **WHEN** the following source is lexed:
  ```
  x = (1 +
       2)
  ```
- **THEN** the newline between `1 +` and `2` does NOT produce a `Newline` token (suppressed by paren depth).

### Scenario: F-string Token

- **WHEN** `lex_raw("f\"hello {name}\"", file_id)` is called.
- **THEN** a single `FStr("hello {name}")` token is produced.

### Scenario: Triple-quoted String

- **WHEN** source containing `"""multi\nline"""` is lexed.
- **THEN** a single `TripleStr("multi\nline")` token is produced spanning the full range.

## Diagrams

### Lexer Pipeline

```mermaid
flowchart LR
    Source["Source text"]
    Logos["logos::Lexer<TokenKind>"]
    Raw["Raw token stream"]
    IP["IndentProcessor"]
    Final["Final token stream with INDENT/DEDENT/EOF"]
    Source --> Logos --> Raw --> IP --> Final
```

### IndentProcessor State Machine

```mermaid
stateDiagram-v2
    [*] --> AtLineStart
    AtLineStart --> InLine: non-whitespace token (compute indent)
    InLine --> InLine: any token (not newline)
    InLine --> AtLineStart: Newline (paren_depth == 0)
    InLine --> InLine: Newline (paren_depth > 0, suppress)
    InLine --> InParen: open bracket
    InParen --> InParen: any token
    InParen --> InLine: close bracket (depth == 0)
    InLine --> EmitDedents: EOF
    EmitDedents --> [*]: all DEDENT + EOF emitted
```


## Changes

### R2 Clarification: Brace Token Processing Order

The existing R2 implementation handles `LBrace` at line start with a specific ordering guarantee that the parser depends on:

```yaml
brace_at_line_start_invariant:
  trigger: "LBrace token when at_line_start=true AND paren_depth=0"
  steps:
    1: "Compute indent from preceding Newline"
    2: "Emit INDENT or DEDENT(s) as needed"
    3: "Set at_line_start = false"
    4: "Increment paren_depth"
    5: "Push LBrace to output"
  guarantee: |
    Dict/set literals starting a line inside a block receive correct
    INDENT/DEDENT tokens BEFORE entering implicit-line-continuation mode.
```

### File Changes

```yaml
files: []
```

No lexer file modifications required. The IndentProcessor correctly produces INDENT/DEDENT tokens for all brace scenarios. The fix is entirely in the parser layer (see `statements` change-spec).

### Acceptance Criteria (Verification)

| ID | Input | Expected Token Stream (relevant tokens) |
|----|-------|-----------------------------------------|
| L-BRACE-1 | `try:\n    d = {}['x']\nexcept KeyError:\n    pass` | `... Newline INDENT Ident Eq LBrace RBrace LBracket Str RBracket Newline DEDENT ...` |
| L-BRACE-2 | `try:\n    {}\nexcept:\n    pass` | `... Newline INDENT LBrace RBrace Newline DEDENT ...` |
| L-BRACE-3 | `if True:\n    d = {1: 2}\n    print(d)` | `... Newline INDENT Ident Eq LBrace Int Colon Int RBrace Newline Ident LParen Ident RParen Newline DEDENT ...` |

# Reviews
