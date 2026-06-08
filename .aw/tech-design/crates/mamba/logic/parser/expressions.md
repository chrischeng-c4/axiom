---
id: parser-expressions
title: Parser — Expression Pratt Parser
crate: mamba
files:
  - crates/mamba/src/parser/expr.rs
  - crates/mamba/src/parser/expr_compound.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 61a87cb6a
---

# Parser — Expression Pratt Parser

`parser/expr.rs` (1257 LOC) and `parser/expr_compound.rs` (802 LOC)
together implement Pratt expression parsing producing `Spanned<Expr>`
nodes per `parser/ast.md`. `expr.rs` carries the precedence tables
(`prefix_bp`, `infix_bp`) plus f-string parsing helpers; `expr_compound.rs`
handles the compound expressions (lambda, comprehensions, walrus, ternary
if-expression, unary chains).

Three load-bearing invariants:

1. **Pratt binding-power tables match Python operator precedence
   exactly** — `infix_bp(op)` returns `(left_bp, right_bp)` per CPython's
   precedence table. Off-by-one would silently parse `a or b and c` as
   `(a or b) and c` instead of `a or (b and c)`.
2. **`is_comparison_op` chains via `Expr::Compare`, not nested
   `BinOp`** — `1 < x < 10` lowers to `Compare { ops: [Lt, Lt],
   comparators: [x, 10] }`, NOT `(1 < x) < 10`. CPython chains
   comparisons with short-circuit evaluation; nested BinOp would lose
   that.
3. **F-string parts are parsed at lex time, not parse time** — the
   lexer produces `FStr(String)` containing the raw inner text;
   `parse_fstring_parts` here re-parses the content into
   `Vec<FStringPart>` with literal/expr/format-spec splitting, then
   recursively parses each expr part as a full `Expr`.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: expr-parse-types
types:
  PrattCore:        { kind: struct, label: "expr.rs — prefix_bp / infix_bp tables + parse_expression entry" }
  CompoundExprs:    { kind: struct, label: "expr_compound.rs — lambda / comprehensions / walrus / if-expr / starred" }
  FStringHandlers:  { kind: struct, label: "parse_fstring_parts / parse_fstring_expr / strip_literal / split_expr_and_*" }
  ASTExpr:          { kind: enum,   label: "parser::ast::Expr (~50 variants)" }
  TokenStream:      { kind: struct, label: "from lexer (Vec<Token>)" }
  TypeExprParser:   { kind: struct, label: "parser/type_expr.rs (annotations within expressions)" }
edges:
  - { from: PrattCore,       to: ASTExpr,        kind: owns,       label: "produces" }
  - { from: PrattCore,       to: CompoundExprs,  kind: references, label: "delegates compound forms" }
  - { from: PrattCore,       to: FStringHandlers, kind: references, label: "FStr token handler" }
  - { from: PrattCore,       to: TokenStream,    kind: references, label: "consumes" }
  - { from: CompoundExprs,   to: PrattCore,      kind: references, label: "recurse into sub-expressions" }
  - { from: FStringHandlers, to: PrattCore,      kind: references, label: "recurse into expr parts" }
  - { from: PrattCore,       to: TypeExprParser, kind: references, label: "annotated assigns" }
---
classDiagram
    class PrattCore
    class CompoundExprs
    class FStringHandlers
    class ASTExpr
    class TokenStream
    class TypeExprParser
    PrattCore --> ASTExpr : produces
    PrattCore --> CompoundExprs : delegates
    PrattCore --> FStringHandlers : f-string
    PrattCore --> TokenStream : consumes
    CompoundExprs --> PrattCore : recurse
    FStringHandlers --> PrattCore : recurse parts
    PrattCore --> TypeExprParser : annotations
```

## Precedence shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "expr-parse-types"
$defs:
  PrecedenceLevel:
    type: object
    description: "Pratt binding power per CPython operator precedence"
    properties:
      level:        { type: integer, minimum: 1, maximum: 17 }
      operators:    { type: array, items: { type: string } }
      associativity: { type: string, enum: [left, right, chain] }
      description:  { type: string }
    required: [level, operators, associativity]
    examples:
      - { level: 1,  operators: [Or],         associativity: left,  description: "lowest binding — boolean or" }
      - { level: 2,  operators: [And],        associativity: left }
      - { level: 3,  operators: [Not],        associativity: right, description: "unary not — prefix" }
      - { level: 4,  operators: [Eq, Ne, Lt, Le, Gt, Ge, In, NotIn, Is, IsNot], associativity: chain, description: "comparison chains via Expr::Compare" }
      - { level: 5,  operators: [BitOr],      associativity: left }
      - { level: 6,  operators: [BitXor],     associativity: left }
      - { level: 7,  operators: [BitAnd],     associativity: left }
      - { level: 8,  operators: [LShift, RShift], associativity: left }
      - { level: 9,  operators: [Add, Sub],   associativity: left }
      - { level: 10, operators: [Mul, TrueDiv, FloorDiv, Mod, MatMul], associativity: left }
      - { level: 11, operators: [Neg, Pos, Invert], associativity: right, description: "unary arith — prefix" }
      - { level: 12, operators: [Pow],        associativity: right, description: "right-associative" }
      - { level: 13, operators: [Await],      associativity: right }
      - { level: 14, operators: [Subscript, Attribute, Call], associativity: left, description: "primary postfix" }
  FStringSplit:
    description: "How parse_fstring_parts splits a raw FStr"
    type: object
    properties:
      parts:
        type: array
        items:
          oneOf:
            - { title: Literal,    properties: { text: { type: string } } }
            - { title: Expr,       properties: { source: { type: string }, conversion: { type: string, enum: [s, r, a, ""] }, format_spec: { type: string } } }
    required: [parts]
```

## Pratt parse logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pratt-parse
entry: enter
nodes:
  enter:        { kind: start,    label: "parse_expression(min_bp = 0)" }
  parse_atom:   { kind: process,  label: "parse_atom — literal / Name / paren-group / list / dict / set / lambda / fstring / comprehension" }
  is_unary:     { kind: decision, label: "next token is prefix op?" }
  parse_unary:  { kind: process,  label: "rbp = prefix_bp(op); recurse parse_expression(rbp)" }
  loop_:        { kind: process,  label: "look at next token for infix op" }
  is_infix:     { kind: decision, label: "next is infix op AND lbp >= min_bp?" }
  is_compare:   { kind: decision, label: "is_comparison_op?" }
  build_compare: { kind: process, label: "Expr::Compare { ops, comparators } — chained" }
  build_binop:  { kind: process,  label: "Expr::BinOp { lhs, op, rhs }" }
  is_postfix:   { kind: decision, label: "Subscript / Attribute / Call?" }
  postfix:      { kind: process,  label: "wrap lhs in Subscript / Attribute / Call" }
  done:         { kind: terminal, label: "return Spanned<Expr>" }
edges:
  - { from: enter,        to: is_unary }
  - { from: is_unary,     to: parse_unary,    label: "yes" }
  - { from: is_unary,     to: parse_atom,     label: "no" }
  - { from: parse_unary,  to: loop_ }
  - { from: parse_atom,   to: loop_ }
  - { from: loop_,        to: is_infix }
  - { from: is_infix,     to: is_postfix,     label: "no infix" }
  - { from: is_infix,     to: is_compare,     label: "yes" }
  - { from: is_compare,   to: build_compare,  label: "yes — chain" }
  - { from: is_compare,   to: build_binop,    label: "no" }
  - { from: build_compare, to: loop_ }
  - { from: build_binop,  to: loop_ }
  - { from: is_postfix,   to: postfix,        label: "yes" }
  - { from: is_postfix,   to: done,           label: "no" }
  - { from: postfix,      to: loop_ }
---
flowchart TD
    enter([parse_expression min_bp]) --> is_unary{prefix op?}
    is_unary -->|yes| parse_unary[rbp = prefix_bp; recurse]
    is_unary -->|no| parse_atom[atom]
    parse_unary --> loop_[next token]
    parse_atom --> loop_
    loop_ --> is_infix{infix? lbp >= min_bp?}
    is_infix -->|no infix| is_postfix{postfix?}
    is_infix -->|yes| is_compare{comparison?}
    is_compare -->|yes — chain| build_compare[Compare ops list]
    is_compare -->|no| build_binop[BinOp]
    build_compare --> loop_
    build_binop --> loop_
    is_postfix -->|yes| postfix[Subscript/Attr/Call]
    is_postfix -->|no| done([Spanned Expr])
    postfix --> loop_
```

## Compound + f-string interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: expr-recursive-flow
actors:
  - { id: PrattCore,    kind: system, label: "expr.rs parse_expression" }
  - { id: Compound,     kind: system, label: "expr_compound.rs" }
  - { id: FStringMod,   kind: system, label: "parse_fstring_parts" }
  - { id: AST,          kind: system, label: "Expr / FStringPart nodes" }
messages:
  - { from: PrattCore,  to: Compound,   name: "lambda token observed → parse_lambda" }
  - { from: Compound,   to: PrattCore,  name: "recurse for body expression" }
  - { from: PrattCore,  to: Compound,   name: "Expr::Lambda" }
  - { from: PrattCore,  to: Compound,   name: "for-clause inside [] → parse_comprehension" }
  - { from: Compound,   to: PrattCore,  name: "iter / ifs sub-expressions" }
  - { from: Compound,   to: AST,        name: "Expr::ListComp / SetComp / DictComp / GenExpr" }
  - { from: PrattCore,  to: FStringMod, name: "FStr token → parse_fstring_parts" }
  - { from: FStringMod, to: FStringMod, name: "split into Literal / Expr parts via { } scan" }
  - { from: FStringMod, to: PrattCore,  name: "for each Expr part: recurse parse_expression on inner source" }
  - { from: FStringMod, to: AST,        name: "Vec<FStringPart>" }
---
sequenceDiagram
    participant PrattCore
    participant Compound
    participant FStringMod
    participant AST
    PrattCore->>Compound: lambda
    Compound->>PrattCore: recurse body
    Compound->>AST: Lambda
    PrattCore->>Compound: comprehension
    Compound->>PrattCore: iter / ifs
    Compound->>AST: ListComp / etc
    PrattCore->>FStringMod: FStr
    FStringMod->>FStringMod: split parts
    FStringMod->>PrattCore: recurse expr parts
    FStringMod->>AST: FStringPart vec
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: chained-comparison
    given: chained_compare/deep_broad.py contains `1 < x < 10`
    when: parse_expression consumes the comparison chain
    then: the AST is Expr::Compare with Lt and Lt operators plus x and 10 comparators
  - id: walrus-precedence
    given: language/walrus_basic.py contains `(n := f()) + 1`
    when: Pratt parsing applies binding powers
    then: the expression is BinOp(Walrus(n, Call f), Add, Int 1)
  - id: fstring-nested-format
    given: fstring/format_spec_broad.py contains `f'{x:{width}.2f}'`
    when: parse_fstring_parts splits the raw f-string token
    then: the expression part preserves a nested format_spec expression
  - id: lambda-immediate-call
    given: language/lambda_call.py contains `(lambda x: x*2)(5)`
    when: postfix call parsing runs after the lambda atom
    then: the AST is a Call whose func is Lambda and args contains Int 5
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
runner: "cargo test -p mamba --test parser_tests --release -- {name} --test-threads=1"
fixtures:
  - id: chain_compare
    name: "test_chained_comparison"
    description: "1 < x < 10 → Expr::Compare with two ops"
  - id: walrus
    name: "test_walrus_in_expression"
    description: "(n := f()) + 1 produces nested Walrus"
  - id: fstring_nested
    name: "test_fstring_nested_format_spec"
    description: "f'{x:{w}}' splits nested {} correctly"
  - id: lambda_call
    name: "test_lambda_immediately_called"
    description: "(lambda x: x*2)(5) yields Call(Lambda)"
  - id: precedence_pow_right_assoc
    name: "test_pow_right_associative"
    description: "2 ** 3 ** 2 parses as 2 ** (3 ** 2)"
  - id: comprehensions
    name: "test_comprehensions_all_forms"
    description: "[..], {..}, {k: v for ..}, generator expr"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/parser/expr.rs
    action: modify
    impl_mode: hand-written
    description: "Pratt parser core — prefix_bp / infix_bp / parse_expression / parse_atom; f-string parts splitter; comparison chain detector. Hand-written; precedence tables are the contract."
  - file: crates/mamba/src/parser/expr_compound.rs
    action: modify
    impl_mode: hand-written
    description: "Compound expressions — lambda / comprehensions / walrus / if-expr / starred / yield / await. Hand-written."
```
