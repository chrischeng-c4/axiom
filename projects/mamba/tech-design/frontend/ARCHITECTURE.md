# frontend — architecture (as-is, 2026-07-15)

Domain per `tech-design/README.md`: source text → tokens → AST → typed HIR. Source: `src/lexer/`, `src/parser/`, `src/hir/`, `src/source/`, `src/diagnostic/`.
Neighbors (cross-referenced, not restated): resolver assigns `SymbolId` (`../name-resolution/` — todo; `src/resolve/`); AST walls (`../type-system/ARCHITECTURE.md`); the **AST→HIR construction pass itself** lives in `src/lower/ast_to_hir.rs` and is owned by `../codegen/ARCHITECTURE.md` (this domain owns the HIR *data model* in `src/hir/mod.rs`, not the lowering walk); PEP 695 desugar is `lower::pep695` (codegen).

## Responsibilities

- Lexing: logos raw scan + synthetic INDENT/DEDENT/EOF injection (`lexer/mod.rs:lex`, `lexer/indent.rs:IndentProcessor`).
- Recursive-descent + Pratt expression parsing → span-annotated AST (`parser/mod.rs:Parser`, `parser/ast.rs`).
- Parse-level **semantic** rejects that CPython raises at compile time (bare walrus, solo `/`, compound-after-`;`, match-pattern shape) — these must fire at parse, not later.
- Post-parse PEP-classic private-name mangling (`parser/mangle.rs`, driven by `parser::parse`).
- The HIR type model (`src/hir/mod.rs`) — desugared, `SymbolId`/`TypeId`-resolved node set that lowering targets.
- Byte-span source model + single-error diagnostic rendering (`source/`, `diagnostic/mod.rs`).

## Key structures & invariants

| Structure | Where | Rule that must hold |
|---|---|---|
| `Span{file,start,end}` | `source/span.rs:9` | **byte** offsets (not char/col); `merge` = min-start/max-end; `line_col` via cached `line_starts` (`source/mod.rs:27`) |
| `Token{kind,start,end}` / `TokenKind` | `lexer/token.rs:737` | logos-derived; hard keywords are `#[token]` variants; INDENT/DEDENT/Eof are zero-width synthetic (`indent.rs:73,98`) |
| `Parser` state | `parser/mod.rs:16` | `pending_stmts` LIFO-drained before any new token (chained `a=b=c=e`); `stmt_expr_toplevel` = bare-walrus arm; `in_class_body` = walrus-in-class-body reject |
| `Param{kind,pos_only,kw_only}` | `parser/ast.rs:242` | `ParamKind∈{Regular,Star,DoubleStar}` only; `pos_only`/`kw_only` are **introspection metadata**, call binding unaffected (`ast.rs:248,252`) |
| `HirParamSig.kind:u8` | `hir/mod.rs:48` | CPython `inspect.Parameter` ordinal 0..4 (POSITIONAL_ONLY..VAR_KEYWORD); lowering derives it from `pos_only`/`kw_only`/`ParamKind` |
| `HirModule` | `hir/mod.rs:8` | every name is `SymbolId`, every type `TypeId`; synthetic lowerer locals use `SymbolId≥1_000_000` (codegen invariant, `../codegen`) |
| `MambaError::Syntax{span,message}` | `error.rs:8` | parser is **fail-fast**: first syntax error aborts (`Result<T>`); no recovery, no multi-diagnostic |

## Control flow

1. `parser::parse(src,file_id)` (`parser/mod.rs:360`) = `lexer::lex` → `Parser::new` → `parse_module` → `mangle_private_names`.
2. `lexer::lex` (`mod.rs:11`): `lex_raw` (logos; **unknown chars silently skipped**, `mod.rs:28`) → `IndentProcessor::process` (indent_stack, `paren_depth` suppresses newlines inside `()[]{}`, comments dropped, trailing DEDENTs+Eof).
3. `Parser::new`: if logos dropped non-ASCII idents (`tokens_need_unicode_ident_repair`), rebuild via `prepare_parser_tokens`→`repair_unicode_ident_tokens` (gap-fill Ident tokens + merge runs, `mod.rs:239-349`).
4. `parse_module` (`mod.rs:56`): loop `parse_stmt`; `;` separates simple stmts (`is_compound_start`→reject after `;`).
5. `parse_ident_stmt` sets `stmt_expr_toplevel=true` (`stmt.rs:379`) → `parse_expr` (`expr.rs:85`) captures-and-clears it, so only the statement top — not parenthesized/nested — treats `:=` as the bare-walrus SyntaxError (`expr.rs:121`).
6. `parse_expr_bp` (`expr.rs:144`): Pratt loop over `infix_bp`; postfix call/attr/index chained; comparison runs fold into `Expr::ChainedCompare` (`expr.rs:206`).
7. `parse_params` (`stmt.rs:641`): `seen_star` (bare `*` or `*args`) → later Regular params `kw_only=true`; `/` retro-sets `pos_only=true` on preceding Regular params, else "at least one argument must precede /" (`stmt.rs:701`).
8. Numeric/f-string literals: overflow ints lexed as `Int(BIG_INT_LITERAL_SENTINEL=i64::MIN)` then reconstructed to `Expr::BigIntLit(text)` (`token.rs:6`, `expr.rs:239`); `FStr`/`RawFStr` carry raw inner text, replacement fields re-parsed by `parse_fstring_parts` (`expr.rs:280`), adjacent string/f-string literals concatenated.
9. Output AST feeds `../type-system` (walls) then `../codegen` (`pep695` desugar → `lower_module`/`ast_to_hir`). Errors → `CompilerSession::render_error` → `diagnostic::render_error` (span→line/col + one source line, no caret) → `main.rs:994` eprintln.

## CPython-parity semantics

- **Bare walrus** `a:=5` as an expression statement is a SyntaxError (must be `(a:=5)`); enforced structurally at parse (`expr.rs:116-126`), plus walrus-in-comprehension-iterable and walrus-in-class-body rejects (`expr_compound.rs`).
- **Solo `/`** and `/` after only `*`/`**` are SyntaxErrors (`stmt.rs:698-706`); `pos_only`/`kw_only` markers are introspection-only here — positional-only/keyword-only *enforcement* is runtime binding (`../calling-convention/` — todo).
- **Private-name mangling** `__x`→`_Class__x` inside class bodies, dunders `__x__` exempt; runs post-parse over the whole module (`parser/mangle.rs`) before checker/lowering observe the tree.
- **Soft keywords** `match`/`case`/`type`/`enum` + builtin type names `int/float/bool/str/list/dict/tuple` are usable as identifiers (`Parser::is_name_token`, `mod.rs:172`).
- **Chained comparison** `a<b<c` becomes one `ChainedCompare` (short-circuit preserved downstream); chained assignment desugars to per-target `Assign`s via `pending_stmts`.
- Contract deviations: indentation is **byte width from last newline** (`indent.rs:90`) — tabs count as 1, no tabstop=8, no `TabError`; unknown source bytes are dropped rather than raising SyntaxError.

## Known hazards

- Unknown chars silently skipped (`lexer/mod.rs:28`) — a malformed byte vanishes, quietly changing the program instead of erroring.
- Byte-width indentation (`indent.rs:90`) — mixed tabs/spaces mis-indent silently; diverges from CPython tabstop-8 / TabError.
- Unicode-ident repair is a post-hoc gap reconstruction (`parser/mod.rs:255-349`) — relies on `is_identifier_text` heuristic + adjacent-run merge; fragile for non-ASCII near operators.
- `stmt_expr_toplevel` must be set only at expr-statement entry and cleared at `parse_expr` top — any new expr-stmt path that forgets to set it drops the bare-walrus check; any that leaks it into a sub-parse over-fires.
- Reading `pos_only`/`kw_only` as binding-authoritative is wrong — they are metadata only (`ast.rs:248`); binding lives in the runtime.
- `Param.ty` doc says "mandatory type annotation" (`ast.rs:240`) but code defaults omitted annotations to `TypeExpr::Named("Any")` (`stmt.rs:750`) — comment is stale; unannotated params are legal.
- `expect` compares `std::mem::discriminant` only (`parser/mod.rs:117`) — token payloads (e.g. `Int(v)`) are not matched; fine for keywords, a trap if a check needs the value.
- Fail-fast single error — the user sees at most one syntax diagnostic per compile; there is no error recovery.
- ASTs built by non-`parse` entrypoints (exec/compile) must call `mangle_private_names` explicitly or dunder mangling silently won't run (`parser/mod.rs:372`).

## Extension points

- New token: add `TokenKind` variant (`lexer/token.rs`); if a soft keyword, also extend `is_name_token` (`parser/mod.rs:172`).
- New stmt/expr: AST variant (`parser/ast.rs`) + parse fn (`parser/{stmt,stmt_compound,expr,expr_compound,pattern,type_expr}.rs`) + HIR variant (`hir/mod.rs`) + lowering arm (`lower/ast_to_hir.rs`, codegen domain).
- New compile-time SyntaxError: raise `MambaError::syntax(span,msg)` in the owning parse fn (must be here when CPython rejects at compile, cf. bare walrus).
- Param-marker semantics: `parse_params` (`stmt.rs:641`); diagnostic layout: `diagnostic/mod.rs:render_error`.

## EC surface

| Dimension / dir | Proves |
|---|---|
| `tests/cpython/_regression/core/grammar/*.py` via `tests/harness/cpython/grammar.rs` | parser-**acceptance** of Python 3.12 grammar (`# RUN: parse`, `# XFAIL`); never lowered/executed (`name_mangling.py`, `numeric_literals.py`, `operators_precedence.py`, `errors.py`, `subscript_slicing.py`) |
| `errors/` dimension | SyntaxError taxonomy — bare walrus, solo `/`, compound-after-`;` (a missing raise is a failure) |
| `surface/` dimension | `inspect.signature` reflecting `pos_only`/`kw_only`/param kinds via `HirParamSig` ordinals |
| `type/` dimension | parser output feeding the checker walls (`../type-system`) |
| Rust `#[cfg(test)] mod tests` in `mod.rs`/`span.rs`/`expr.rs` + `parser/tests/core.rs`, `lexer/tests/core.rs` | unit coverage of tokenizer, span math, param/walrus parsing |

Fixture layout + dimensions: `tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`. Harness verdict semantics: `../external-contracts/HARNESS.md`.
