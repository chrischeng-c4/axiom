---
id: parser-statements
title: Parser — Statements
crate: mamba
files:
  - crates/mamba/src/parser/stmt.rs
  - crates/mamba/src/parser/stmt_compound.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 61a87cb6a
---

# Parser — Statements

`parser/stmt.rs` (1144 LOC) handles single-line statements
(import, return, raise, pass, break, continue, global, nonlocal,
assert, del, type-alias, simple/aug/var-decl assignments, expression
statements). `parser/stmt_compound.rs` (1220 LOC) handles compound
statements with bodies (def, async def, class, enum, if, while, for,
with, try, match).

`parse_stmt` is the dispatch entry: it inspects the leading token kind
to pick a branch, parses decorators if present, then delegates to a
specific parser. Indent / Dedent tokens come from the lexer and frame
each compound body.

Three load-bearing invariants:

1. **Decorators are parsed exactly once and attached to the next
   def / class / async def** — `@d1 \n @d2 \n def f(): ...` collects
   `[d1, d2]` into `FnDef.decorators`. A bare `@expr` followed by
   anything else is a syntax error.
2. **`async` token is a hard keyword in this language** — unlike
   CPython 3.5–3.6 which had `async` as a soft keyword, Mamba's
   lexer always tokenizes `async` as `Async`. So `async def`,
   `async for`, `async with` parse cleanly without lookahead.
3. **`elif` and `else` chains attach to the most recent open `if`** —
   `If { condition, body, elif_clauses, else_body }` — flat list of
   elif tuples + optional else, NOT recursive nested `If` in else.
   This matches CPython's AST.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: stmt-parse-types
types:
  StmtCore:        { kind: struct, label: "stmt.rs — single-line statements" }
  StmtCompound:    { kind: struct, label: "stmt_compound.rs — body-bearing statements" }
  ExprParser:      { kind: struct, label: "from parser/expr.rs (parse_expression)" }
  PatternParser:   { kind: struct, label: "from parser/pattern.rs (match-case patterns)" }
  TypeExprParser:  { kind: struct, label: "from parser/type_expr.rs (annotations)" }
  ASTStmt:         { kind: enum,   label: "parser::ast::Stmt (~30 variants)" }
  TokenStream:     { kind: struct, label: "Indent / Dedent / Newline + content tokens" }
edges:
  - { from: StmtCore,       to: ASTStmt,         kind: owns,       label: "produces simple Stmt variants" }
  - { from: StmtCompound,   to: ASTStmt,         kind: owns,       label: "produces compound Stmt variants" }
  - { from: StmtCore,       to: ExprParser,      kind: references, label: "expressions inside simple stmts" }
  - { from: StmtCompound,   to: ExprParser,      kind: references, label: "headers + decorators" }
  - { from: StmtCompound,   to: PatternParser,   kind: references, label: "match arms" }
  - { from: StmtCompound,   to: TypeExprParser,  kind: references, label: "param + return types" }
  - { from: StmtCompound,   to: TokenStream,     kind: references, label: "Indent..body..Dedent" }
  - { from: StmtCore,       to: TokenStream,     kind: references }
---
classDiagram
    class StmtCore
    class StmtCompound
    class ExprParser
    class PatternParser
    class TypeExprParser
    class ASTStmt
    class TokenStream
    StmtCore --> ASTStmt : simple variants
    StmtCompound --> ASTStmt : compound variants
    StmtCore --> ExprParser : refs
    StmtCompound --> ExprParser : refs
    StmtCompound --> PatternParser : match arms
    StmtCompound --> TypeExprParser : annotations
    StmtCompound --> TokenStream : Indent + Dedent
    StmtCore --> TokenStream : refs
```

## Statement classification shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "stmt-parse-types"
$defs:
  StatementClass:
    type: string
    enum: [Simple, Compound, Decorated]
    description: "Top-level dispatch tier"
  SimpleLeadingTokens:
    description: "Tokens that mark a single-line statement"
    type: array
    items: { type: string }
    examples:
      - [Pass, Break, Continue, Return, Yield, Raise, Import, From,
         Global, Nonlocal, Assert, Del, Type, Identifier]
  CompoundLeadingTokens:
    description: "Tokens that introduce a compound body"
    type: array
    items: { type: string }
    examples:
      - [Def, Async, Class, Enum, If, While, For, With, Try, Match]
  DecoratorTrigger:
    description: "@ token starts a decorator chain"
    type: string
    const: At
```

## Statement dispatch logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: stmt-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "parse_stmt" }
  is_at:        { kind: decision, label: "leading @?" }
  parse_decos:  { kind: process,  label: "consume @ expr Newline; collect decorator list" }
  is_def_class: { kind: decision, label: "next is def / async def / class?" }
  attach_decos: { kind: process,  label: "parse def/class; FnDef.decorators / ClassDef.decorators = list" }
  bare_at_err:  { kind: terminal, label: "syntax error: bare @" }
  is_compound:  { kind: decision, label: "leading is def/async/class/enum/if/while/for/with/try/match?" }
  compound:     { kind: process,  label: "delegate to stmt_compound — parse header + Indent body Dedent" }
  is_simple:    { kind: decision, label: "leading is import/global/return/raise/pass/break/continue/etc.?" }
  simple:       { kind: process,  label: "single-line statement; consume Newline" }
  is_assign:    { kind: decision, label: "expression followed by = or += etc.?" }
  parse_assign: { kind: process,  label: "VarDecl / Assign / AugAssign" }
  expr_stmt:    { kind: process,  label: "Stmt::Expr(expr)" }
  done:         { kind: terminal, label: "Spanned<Stmt>" }
edges:
  - { from: enter,        to: is_at }
  - { from: is_at,        to: parse_decos,  label: "yes" }
  - { from: is_at,        to: is_compound,  label: "no" }
  - { from: parse_decos,  to: is_def_class }
  - { from: is_def_class, to: attach_decos, label: "yes" }
  - { from: is_def_class, to: bare_at_err,  label: "no" }
  - { from: is_compound,  to: compound,     label: "yes" }
  - { from: is_compound,  to: is_simple,    label: "no" }
  - { from: is_simple,    to: simple,       label: "yes" }
  - { from: is_simple,    to: is_assign,    label: "no" }
  - { from: is_assign,    to: parse_assign, label: "yes" }
  - { from: is_assign,    to: expr_stmt,    label: "no" }
  - { from: attach_decos, to: done }
  - { from: compound,     to: done }
  - { from: simple,       to: done }
  - { from: parse_assign, to: done }
  - { from: expr_stmt,    to: done }
---
flowchart TD
    enter([parse_stmt]) --> is_at{leading @?}
    is_at -->|yes| parse_decos[collect decorators]
    is_at -->|no| is_compound{compound?}
    parse_decos --> is_def_class{def/async/class next?}
    is_def_class -->|yes| attach_decos[FnDef/ClassDef.decorators]
    is_def_class -->|no| bare_at_err([syntax error])
    is_compound -->|yes| compound[stmt_compound delegate]
    is_compound -->|no| is_simple{simple?}
    is_simple -->|yes| simple[single-line]
    is_simple -->|no| is_assign{expr = / +=?}
    is_assign -->|yes| parse_assign[VarDecl/Assign/AugAssign]
    is_assign -->|no| expr_stmt[Stmt::Expr]
    attach_decos --> done([Spanned Stmt])
    compound --> done
    simple --> done
    parse_assign --> done
    expr_stmt --> done
```

## Compound body interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: stmt-compound-flow
actors:
  - { id: Stmt,        kind: system, label: "stmt.rs parse_stmt" }
  - { id: Compound,    kind: system, label: "stmt_compound.rs" }
  - { id: TokenStream, kind: system, label: "Indent..Dedent body framing" }
  - { id: ExprParser,  kind: system, label: "parser/expr.rs" }
messages:
  - { from: Stmt,        to: Compound,    name: "delegate (e.g., parse_if)" }
  - { from: Compound,    to: ExprParser,  name: "parse condition expression" }
  - { from: ExprParser,  to: Compound,    name: cond_expr }
  - { from: Compound,    to: TokenStream, name: "expect Colon Newline Indent" }
  - { from: Compound,    to: Stmt,        name: "recurse parse_stmt for body until Dedent" }
  - { from: TokenStream, to: Compound,    name: "Dedent" }
  - { from: Compound,    to: Compound,    name: "loop elif clauses; optional else_body" }
  - { from: Compound,    to: Stmt,        name: "Stmt::If with elif_clauses + else_body" }
---
sequenceDiagram
    participant Stmt
    participant Compound
    participant TokenStream
    participant ExprParser
    Stmt->>Compound: delegate parse_if
    Compound->>ExprParser: condition
    ExprParser-->>Compound: cond_expr
    Compound->>TokenStream: Colon Newline Indent
    Compound->>Stmt: recurse body until Dedent
    TokenStream-->>Compound: Dedent
    Compound->>Compound: elif chain + else
    Compound-->>Stmt: Stmt::If
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: elif-flat
    given: language/elif_chain.py contains if, elif, and else clauses
    when: parse_if builds the statement
    then: Stmt::If stores elif_clauses as a flat list and else_body separately
  - id: decorators-source-order
    given: decorator_full/decorator_full.py stacks multiple decorators before a function
    when: parse_stmt consumes the decorator chain
    then: FnDef.decorators preserves source order for later right-to-left application
  - id: async-def
    given: async_await/gather.py defines async functions with await expressions
    when: compound statement parsing runs
    then: AsyncFnDef statements are produced and their bodies contain Await expressions
  - id: match-arms
    given: language/match_basic.py contains match and case clauses
    when: parse_match runs
    then: Stmt::Match contains ordered MatchArm entries with patterns and optional guards
  - id: augmented-assign
    given: augmented_assign/sequences.py uses augmented assignment targets
    when: simple statement parsing sees += or related operators
    then: AugAssign statements capture target, op, and value
```

## Tests
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: parser-statements-test-plan
title: Parser Statements Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test parser_tests --release -- {name} --test-threads=1"]
    Runner --> Elif["test_elif_clauses_flat"]
    Runner --> Decorators["test_decorators_in_source_order"]
    Runner --> Async["test_async_def_basic"]
    Runner --> Match["test_match_arms_with_guard"]
    Runner --> AugAssign["test_augmented_assign_all_ops"]
    Runner --> Try["test_try_finally_except_star"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/parser/stmt.rs
    action: modify
    impl_mode: hand-written
    description: "parse_stmt dispatch + simple-statement parsers (import / return / raise / pass / break / continue / global / nonlocal / assert / del / type / assignments). Hand-written."
  - file: crates/mamba/src/parser/stmt_compound.rs
    action: modify
    impl_mode: hand-written
    description: "Compound-statement parsers (def / async def / class / enum / if / while / for / with / try / match). Hand-written; Indent/Dedent framing is the contract."
```
