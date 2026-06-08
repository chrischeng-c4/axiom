---
id: mamba-test-coverage-remaining-spec
main_spec_ref: "crates/mamba/testing/test-coverage-remaining.md"
---

# Mamba Test Coverage Remaining Spec

## Overview
<!-- type: overview lang: markdown -->

Add inline `#[cfg(test)]` unit tests and integration tests to 23 source files in `crates/cclab-mamba`, raising each file from its current coverage to 100% line + 100% branch coverage measured by `cargo llvm-cov --branch`. No coverage-exclusion annotations permitted.

**Batch A — stdlib modules (12 files, 20–49%):**

| File | Current Coverage |
|------|------------------|
| `runtime/stdlib/argparse_mod.rs` | 20% |
| `runtime/stdlib/platform_mod.rs` | 25% |
| `runtime/stdlib/unittest_mod.rs` | 31% |
| `runtime/stdlib/socket_mod.rs` | 34% |
| `runtime/stdlib/array_mod.rs` | 35% |
| `runtime/stdlib/errno_mod.rs` | 37% |
| `runtime/stdlib/traceback_mod.rs` | 37% |
| `runtime/stdlib/codecs_mod.rs` | 46% |
| `runtime/stdlib/logging_mod.rs` | 46% |
| `runtime/stdlib/pickle_mod.rs` | 47% |
| `runtime/stdlib/threading_mod.rs` | 49% |
| `runtime/stdlib/sqlite3_mod.rs` | 49% |

**Batch B — core modules (3 files, 0–45%):**

| File | Current Coverage |
|------|------------------|
| `ffi/c_types.rs` | 0% |
| `driver/mod.rs` | 33% |
| `codegen/cranelift/mod.rs` | 45% |

**Batch C — compiler pipeline (8 files, 66–78%):**

| File | Current Coverage |
|------|------------------|
| `types/check_expr.rs` | 66% |
| `codegen/cranelift/aot.rs` | 67% |
| `codegen/cranelift/jit.rs` | 69% |
| `lexer/token.rs` | 70% |
| `lower/ast_to_hir.rs` | 75% |
| `driver/module_graph.rs` | 76% |
| `lower/hir_to_mir.rs` | 78% |
| `parser/expr_compound.rs` | 78% |

**Test strategy:**

- Inline `#[cfg(test)]` blocks inside each source file for per-function branch coverage
- Integration tests in `crates/mamba/tests/stdlib_coverage_remaining_tests.rs`
- No coverage exclusion annotations (`#[cfg(coverage_not)]`, `// coverage: off`)
- Measure with `cargo llvm-cov --branch` after each batch

**Constraints:**

- `socket_mod` tests: loopback address `127.0.0.1`, port 0 (OS-assigned); no live DNS; no mocks
- `threading_mod` tests: deterministic synchronization via `std::sync::Barrier` or channels; no racy sleeps
- `sqlite3_mod` tests: `:memory:` database only; no filesystem writes
- `c_types.rs` tests: pure Rust type assertions (display_name, size, equality); no C library required
- `driver/mod.rs` tests: use `compile_str`-style helpers or write `.py` temp files; test each `EmitMode` branch

## Requirements
<!-- type: overview lang: markdown -->

### R1: argparse_mod.rs (20%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str` | Str → Some(s); non-Str ObjData → None; null ptr (as_ptr() None) → None |
| `mb_argparse_new` | Str desc → stores description; non-Str desc → stores empty string |
| `mb_argparse_add_argument` | valid Dict parser → arg pushed to `_args` list; null parser (as_ptr() None) → no-op; parser with non-Dict ObjData → no-op |
| `mb_argparse_parse_args` | parser with args + matching env args (positional assign); parser with fewer env args than names (None inserted for remainder); parser with no registered names (empty namespace dict); null parser (as_ptr() None) → empty namespace |

### R2: platform_mod.rs (25%)

| Function | Branches to cover |
|----------|-------------------|
| `register` | happy path (inserts all entries into module) |
| `mb_platform_system` | returns `std::env::consts::OS` as MbValue |
| `mb_platform_node` | `HOSTNAME` env set → returns it; `HOSTNAME` unset + `HOST` set → returns HOST; neither set → returns "localhost" |
| `mb_platform_release` | always "0.0.0" |
| `mb_platform_machine` | returns `std::env::consts::ARCH` |
| `mb_platform_processor` | returns `std::env::consts::ARCH` |
| `mb_platform_python_version` | always "3.12.0" |
| `mb_platform_platform` | returns "OS-ARCH" formatted string |

### R3: unittest_mod.rs (31%)

| Function | Branches to cover |
|----------|-------------------|
| `to_snake` | uppercase char at i>0 → inserts `_`; uppercase at i==0 → no `_`; lowercase → unchanged; empty string → "" |
| `extract_str` | Str → Some(s); non-Str → None |
| `values_equal` | int==int (equal); int==int (unequal); float==float; bool==bool; str==str; neither (fallback `a==b`) |
| `mb_unittest_testcase` | creates dict with `__class__`, `_failures`, `_successes` |
| `mb_unittest_assert_equal` | values equal → None; values unequal → panic |
| `mb_unittest_assert_not_equal` | values unequal → None; values equal → panic |
| `mb_unittest_assert_true` | bool true → None; int non-zero → None; bool false → panic; int 0 → panic |
| `mb_unittest_assert_false` | bool false → None; int 0 → None; bool true → panic |
| `mb_unittest_assert_is` | same value (ptr equality) → None; different value → panic |
| `mb_unittest_assert_is_none` | None value → None; non-None → panic |
| `mb_unittest_assert_in` | item in List → None; item not in List → panic; item (str) in Str collection → None; item not in Str → panic; non-List/Str collection (ObjData other `_`) → panic (found=false); null collection (as_ptr() None) → None (no ptr branch skipped) |
| `mb_unittest_assert_raises` | creates dict with `expected` field |
| `mb_unittest_main` | emits to stderr → None |

### R4: socket_mod.rs (34%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str` | Str → Some(s); non-Str → None |
| `mb_socket_new` | family int + type int (uses them); family None → default 2; stype None → default 1 |
| `mb_socket_connect` | valid Dict sock → sets connected=true, stores addr; null sock → no-op |
| `mb_socket_send` | Str data → returns len; non-Str → returns 0 |
| `mb_socket_recv` | always returns empty string |
| `mb_socket_close` | valid Dict sock → closed=true, connected=false; null sock → no-op |
| `mb_socket_bind` | valid Dict sock → stores addr, bound=true; null sock → no-op |
| `mb_socket_listen` | valid Dict sock → listening=true; null sock → no-op |
| `mb_socket_gethostname` | HOSTNAME set → returns it; HOST set (HOSTNAME unset) → returns HOST; neither → "localhost" |
| `mb_socket_gethostbyname` | always "127.0.0.1" |

### R5: array_mod.rs (35%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str` | Str → Some(s); non-Str → None |
| `mb_array_new` | initializer None → empty items; initializer List → clones items; initializer non-List ObjData → empty items; typecode Str → uses it; typecode non-Str → uses "d" |
| `mb_array_append` | valid Dict arr with `data` list → item pushed; arr with no `data` key → no-op; null arr → no-op |
| `mb_array_extend` | valid arr + List iterable → items extended; valid arr + non-List iterable → no items added; null arr → no-op |
| `mb_array_tolist` | valid arr with `data` → list copy; null arr → empty list |
| `mb_array_tobytes` | arr with int items → bytes; arr with non-int items (filtered by as_int()); null arr → empty bytes |
| `mb_array_frombytes` | Bytes source → appends int MbValues; ByteArray source → appends int MbValues; non-Bytes/ByteArray → no items; null arr or bytes → no-op |

### R6: errno_mod.rs (37%)

| Function | Branches to cover |
|----------|-------------------|
| `mb_errno_errorcode` | creates dict; verifies known code (e.g. 2 → "ENOENT") |
| `mb_errno_strerror` | known codes: 1 (EPERM), 2 (ENOENT), 4 (EINTR), 5 (EIO), 9 (EBADF), 11 (EAGAIN), 13 (EACCES), 22 (EINVAL), 32 (EPIPE), 110 (ETIMEDOUT), 111 (ECONNREFUSED), 113 (EHOSTUNREACH); unknown code (e.g. 0, 999) → "Unknown error" |

### R7: traceback_mod.rs (37%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str` | Str → Some(s); non-Str → None |
| `mb_traceback_format_exc` | always "NoneType: None" |
| `mb_traceback_print_exc` | calls format_exc + eprintln → None |
| `format_exception_value` | None input → "NoneType: None"; Str → "Exception: {s}"; Instance with `message` field → "{class}: {msg}"; Instance with `msg` field (no `message`); Instance with no message fields → just class_name; Dict with `_type` + `message`; Dict with `_type` only (empty msg); int input → "Exception: {i}"; bool input true → "Exception: True"; bool input false → "Exception: False"; other (ptr but not Str/Instance/Dict) → "Exception: {:?}" |
| `mb_traceback_format_exception` | delegates to format_exception_value |
| `mb_traceback_extract_tb` | always empty list |

### R8: codecs_mod.rs (46%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str` | Str → Some; non-Str → None |
| `extract_bytes` | Bytes → Some; non-Bytes → None |
| `normalize_encoding` | "utf-8"/"utf_8"/"UTF-8" → "utf-8"; "ascii"/"ASCII" → "ascii"; "latin-1"/"latin_1"/"iso-8859-1" → "latin-1"; unknown → "utf-8" |
| `mb_codecs_encode` | Str + utf-8 → bytes; Str + ascii (all-ASCII → clean; non-ASCII char → '?'); Str + latin-1 (char ≤255 → ok; char >255 → '?'); non-Str → None; encoding None → defaults to "utf-8" |
| `mb_codecs_decode` | Bytes + utf-8; Bytes + ascii (byte <128 → char; byte ≥128 → '?'); Bytes + latin-1; Str passthrough (obj is Str not Bytes); neither Bytes/Str → None; encoding None → defaults to "utf-8" |
| `mb_codecs_lookup` | with encoding → tuple; missing encoding → defaults to "utf-8" |
| Stubs (register, register_error, lookup_error, open, getincremental*, getreader, getwriter) | all return None |
| Convenience (utf_8_encode/decode, ascii_encode/decode, latin_1_encode/decode) | delegate to encode/decode |

### R9: logging_mod.rs (46%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str` | Str → string; Int → formatted; Float → formatted; Bool true → "True"; Bool false → "False"; None → "None"; other (no ptr, not int/float/bool/none) → "" |
| `log_at_level` | level_num >= LOG_LEVEL → emits to stderr; level_num < LOG_LEVEL → suppressed |
| `mb_logging_debug` | delegates (level 10) |
| `mb_logging_info` | delegates (level 20) |
| `mb_logging_warning` | delegates (level 30) |
| `mb_logging_error` | delegates (level 40) |
| `mb_logging_critical` | delegates (level 50) |
| `mb_logging_getlogger` | None name → "root"; Str name → uses name |
| `mb_logging_basicconfig` | int level → sets LOG_LEVEL; non-int level → no-op |

### R10: pickle_mod.rs (47%)

| Function | Branches to cover |
|----------|-------------------|
| `serialize` | None; bool true → "B1"; bool false → "B0"; int; float; Str; List (empty + non-empty with nested values); Dict (with entries); Tuple; other ObjData variant → "N" |
| `deserialize` | 'N'; 'B' (b'1' → true; other → false); 'I' positive; 'I' negative; 'F'; 'S' with content; 'L' count=0 (empty); 'L' with items; 'T' with items; 'D' with key-value pairs; unknown byte → (None, 1); empty input → (None, 0) |
| `mb_pickle_dumps` | serialize → Bytes |
| `mb_pickle_loads` | Bytes input; ByteArray input; Str input; non-Bytes/ByteArray/Str ObjData → None; null (as_ptr() None) → None |
| `mb_pickle_dump` | serialize + discard → None |
| `mb_pickle_load` | always None |

### R11: threading_mod.rs (49%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str` | Str → Some; non-Str → None |
| `mb_threading_current_thread` | THREAD_NAME is None → name="MainThread"; THREAD_NAME is Some(s) → uses s |
| `mb_threading_active_count` | always 1 |
| `mb_threading_thread` | name is Str → uses it; name non-Str → uses "Thread" |
| `mb_threading_thread_start` | valid Dict thread → started=true, alive=true; null → no-op |
| `mb_threading_thread_join` | valid Dict thread → alive=false; null → no-op |
| `mb_threading_lock` | creates Dict with __class__="Lock", locked=false |
| `mb_threading_lock_acquire` | valid Dict lock → locked=true; null → no-op |
| `mb_threading_lock_release` | valid Dict lock → locked=false; null → no-op |
| `mb_threading_rlock` | delegates to mb_threading_lock |
| `mb_threading_event` | creates Dict with is_set=false |
| `mb_threading_event_set` | valid Dict event → is_set=true; null → no-op |
| `mb_threading_event_clear` | valid Dict event → is_set=false; null → no-op |
| `mb_threading_event_is_set` | valid Dict event with is_set → returns it; null → false |

### R12: sqlite3_mod.rs (49%)

| Function | Branches to cover |
|----------|-------------------|
| `extract_str` | Str → Some; non-Str → None |
| `mb_sqlite3_connect` | Str db_path → uses it; non-Str → ":memory:" |
| `mb_sqlite3_cursor` | returns conn unchanged |
| `extract_table_name` | TABLE token + next not "IF" → returns table name; TABLE + IF NOT EXISTS at i+4 → returns table name; no TABLE → None |
| `mb_sqlite3_execute` | CREATE TABLE → extracts and stores table name; CREATE TABLE IF NOT EXISTS → extracts name; non-CREATE sql → stores last_sql only; null conn → no-op |
| `mb_sqlite3_fetchall` | conn with `_results` → returns list; conn without `_results` → empty list; null → empty list |
| `mb_sqlite3_fetchone` | conn with non-empty `_results` → first item; conn with empty `_results` → None; null → None |
| `mb_sqlite3_commit` | always None |
| `mb_sqlite3_close` | valid Dict conn → closed=true; null → no-op |
| `mb_sqlite3_executemany` | delegates to mb_sqlite3_execute |

---

### R13: ffi/c_types.rs (0%)

| Function / Type | Branches to cover |
|-----------------|-------------------|
| `CType::display_name` | Void → "void"; Int8 → "int8_t"; Int16 → "int16_t"; Int32 → "int32_t"; Int64 → "int64_t"; UInt8 → "uint8_t"; UInt16 → "uint16_t"; UInt32 → "uint32_t"; UInt64 → "uint64_t"; Float → "float"; Double → "double"; Bool → "bool"; ConstChar → "const char*"; MutChar → "char*"; Pointer(inner) → "{inner.display_name()}*"; ConstPointer(inner) → "const {inner.display_name()}*"; Named(s) → s.clone() |
| `CType` derived traits | Debug: format a Pointer variant; Clone: clone a CEnum; PartialEq: eq on Int32 vs Int32, Int32 vs Int64 |
| `CFunction` | construct with params + return_type; clone; eq |
| `CParam` | construct; clone; eq |
| `CStruct` | construct with fields; clone; eq |
| `CField` | construct; clone |
| `CEnum` | construct with variants; clone |
| `CEnumVariant` | value Some(i64) → set; value None → unset |
| `CHeader` | Default::default(); push to functions/structs/enums |

### R14: driver/mod.rs (33%)

| Function | Branches to cover |
|----------|-------------------|
| `CompilerSession::new` | creates session with config |
| `CompilerSession::new_from_project` | with mamba.toml in start_dir → loads project_config; without mamba.toml → base_config unchanged |
| `CompilerSession::load_file` | valid path → reads + adds to source_map; invalid path (no such file) → Err |
| `CompilerSession::check` | valid source → Ok; source with type error → first error returned; EmitMode::Ast → prints AST and returns Ok(()) |
| `CompilerSession::build` | EmitMode::Ast → prints + returns Ok(vec![]); EmitMode::Hir → prints + returns; EmitMode::Mir → prints + returns; type error → Err; Backend::Cranelift → CraneliftBackend; Backend::Llvm + OptLevel::O0/O1/O2/O3; CodegenOutput::ObjectFile; CodegenOutput::LlvmIr; other output |
| `CompilerSession::run` | JIT → executes main_fn + prints; non-JIT output → Err; project_config present → check_native_imports called; codegen error → Err |
| `check_native_imports` | no project_config → skip (called via run with None); native module + exposed symbol → Ok; native module + unexposed symbol → Err; star import → skip; no expose entry for crate → allow all; non-native import (not in MAMBA_MODULES) → skip; empty module path → skip |
| `check_dependencies` | valid abs_path; add_root error → prints warning; topo_sort cycle error → prints + returns; entry module skipped; dep errors printed |

### R15: codegen/cranelift/mod.rs (45%)

| Function | Branches to cover |
|----------|-------------------|
| `collect_used_externs` | empty module → empty set; CallExtern → inserts name; MakeList → inserts mb_list_new + mb_list_append; MakeDict → inserts dict fns; GetAttr → mb_getattr; SetAttr → mb_setattr; GetItem → mb_list_getitem + mb_obj_getitem; SetItem → mb_list_setitem + mb_obj_setitem; MakeTuple → mb_list_new + mb_list_append + mb_list_to_tuple; BinOp → mb_dispatch_binop + mb_obj_contains; UnaryOp → mb_dispatch_unaryop; other instructions → no insertion |
| `VarAlloc::new` | creates empty map, next=0 |
| `VarAlloc::get` | VReg already in map → returns existing Variable; VReg new → inserts + returns new Variable |

---

### R16: types/check_expr.rs (66%)

| Function | Branches to cover |
|----------|-------------------|
| `check_expr` | IntLit, FloatLit, ComplexLit, StrLit, FString, BytesLit, BoolLit, NoneLit, Ellipsis; Ident found in scope; Ident undefined → error; BinOp on various type combos; UnaryOp Pos on numeric (ok); Pos on non-numeric → error; Neg on numeric; Neg on non-numeric → error; Not on bool; Not on non-Bool/Error/Any → error; BitNot on int; BitNot on non-int → error; Tuple/List/Set/Dict literal; Call with positional args; Attribute access on known type; Index access; IfExpr; Lambda; Comprehension; Await; Yield; YieldFrom |
| `check_binop` | arithmetic ops on int+int; float+float; int+float coercion; comparison ops returning bool; `in` / `not in`; `is` / `is not`; bitwise ops on int; type mismatch → error |

### R17: codegen/cranelift/aot.rs (67%)

All uncovered MIR instruction paths in `CraneliftAotCompiler`:

| Path | Branches to cover |
|------|-------------------|
| Function with no params | compiles + emits |
| Function with params | compiles + emits |
| `MirInst::Const` variants | int/float/bool/none/string |
| `MirInst::BinOp` | arithmetic + comparison ops |
| `MirInst::UnaryOp` | neg + not |
| `MirInst::Call` | direct call |
| `MirInst::CallExtern` | extern call |
| `MirInst::Return` | with value; with None |
| Terminator::Jump | unconditional |
| Terminator::Branch | true/false branches |

### R18: codegen/cranelift/jit.rs (69%)

| Path | Branches to cover |
|------|-------------------|
| `CraneliftJitBackend::new` | creates JIT module |
| `CraneliftJitBackend::new_with_externals` | registers external symbols |
| JIT compilation of function returning constant | works end-to-end |
| JIT compilation with conditional branch | true + false arms |
| JIT compilation with function call | call site wired |
| `codegen` returning `CodegenOutput::Jit` | entry point callable |

### R19: lexer/token.rs (70%)

| Function | Branches to cover |
|----------|-------------------|
| `unicode_name_to_char` | known name (e.g. "LATIN SMALL LETTER A" → 'a'); unknown name → None |
| `apply_escape_sequences` | `\n` `\t` `\r` `\\` `\'` `\"` `\0` `\a` `\b` `\f` `\v`; `\xNN` valid hex; `\xNN` invalid hex (skipped); `\N{name}` valid unicode name; `\N{name}` unknown name (kept); `\uNNNN` 4-digit hex; `\UNNNNNNNN` 8-digit hex; non-escape char after `\` (kept literally) |
| `lex_triple_dquote` | valid `"""..."""` content; unterminated (None) |
| `lex_triple_squote` | valid `'''...'''` content; unterminated (None) |
| `lex_fstr_dquote` | f-string double quote; delegates to lex_fstr_inner |
| `lex_fstr_squote` | f-string single quote; delegates to lex_fstr_inner |
| `lex_fstr_inner` | plain literal only; `{expr}` expression; `{{` escaped brace; `}}` escaped brace; close quote reached; unterminated (None) |

### R20: lower/ast_to_hir.rs (75%)

Uncovered AST lowering branches (read existing test coverage gaps via grep on `fn test_` names and cross-reference with match arms):

| Path | Branches to cover |
|------|-------------------|
| `Stmt::ClassDef` | class with methods; class with base class; class body with pass |
| `Stmt::With` | single context manager; multiple context managers |
| `Stmt::Delete` | delete variable; delete attribute |
| `Stmt::Global` / `Stmt::Nonlocal` | global decl; nonlocal decl |
| `Expr::Walrus` | walrus operator (`:=`) |
| `Expr::Starred` | starred in assignment target |
| `Expr::DictComp` | dict comprehension |
| `Expr::SetComp` | set comprehension |
| Augmented assignment with all operators not yet tested (`//=`, `**=`, `^=`, `|=`, `&=`, `<<=`, `>>=`) |

### R21: driver/module_graph.rs (76%)

| Function | Branches to cover |
|----------|-------------------|
| `ModuleGraph::add_root` | absolute import (stdlib) → silently skipped; relative import level 1 (.foo); relative import level 2 (..foo); import with parse error → error collected; circular import detected by visited set |
| `ModuleGraph::topo_sort` | DAG (no cycle) → correct order; cycle present → Err |
| `path_to_module_name` | nested path (a/b/c.py → a.b.c) |
| `infer_import_level` | absolute (0 dots); relative 1 dot; relative 2 dots |

### R22: lower/hir_to_mir.rs (78%)

Uncovered HIR→MIR lowering paths:

| Path | Branches to cover |
|------|-------------------|
| `HirExpr::Await` | await lowering |
| `HirExpr::YieldFrom` | yield-from lowering |
| `HirStmt::With` | with-block lowering |
| `HirStmt::Delete` | delete lowering |
| `HirStmt::Raise` | raise with value; raise bare |
| `HirStmt::Assert` | assert true (no-op); assert false (RuntimeError) |
| `HirStmt::Global` / `HirStmt::Nonlocal` | lowered to no-op |
| Complex `BinOp` not yet covered | floor division `//`; power `**`; bitwise `^` `|` `&` |

### R23: parser/expr_compound.rs (78%)

| Function | Branches to cover |
|----------|-------------------|
| `parse_ternary` | well-formed `x if cond else y`; complex condition with precedence |
| `parse_lambda` | no params (`lambda: body`); one regular param; multiple params; param with type annotation; `*args` param; `**kwargs` param; param with default value; mixed typed + default |
| `parse_yield` | bare `yield`; `yield expr`; `yield from expr` |
| `parse_await` | `await expr` |
| `parse_comprehension` | list comprehension with filter (`if` clause); nested for; generator expression |
| `parse_dict_comp` | `{k: v for ...}` |
| `parse_set_comp` | `{v for ...}` |

## Scenarios
<!-- type: overview lang: markdown -->

## Batch A — stdlib scenarios
<!-- type: overview lang: markdown -->

### argparse_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| new parser with description | `mb_argparse_new(MbValue::from_ptr(MbObject::new_str("desc")))` | dict with `description == "desc"` |
| new parser with non-str desc | `mb_argparse_new(MbValue::from_int(0))` | dict with `description == ""` |
| add_argument valid | parser dict + name MbValue | name appended to `_args` list |
| add_argument null parser | `MbValue::none()` as parser | no panic, returns None |
| parse_args no names | parser with empty `_args` | empty namespace dict |
| parse_args with names, env empty | 2 registered names, `args = []` | both keys map to None |

### platform_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| system | — | non-empty string (OS name) |
| node — HOSTNAME set | env HOSTNAME="testhost" | "testhost" |
| node — neither set | env unset | "localhost" |
| machine | — | non-empty string (arch) |
| platform | — | string contains "-" |

### unittest_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| to_snake camelCase | `"assertEqual"` | `"assert_equal"` |
| to_snake already snake | `"assert_true"` | `"assert_true"` |
| to_snake empty | `""` | `""` |
| assert_equal pass | `from_int(1), from_int(1)` | `None` |
| assert_equal fail | `from_int(1), from_int(2)` | `should_panic` |
| assert_not_equal pass | `from_int(1), from_int(2)` | `None` |
| assert_not_equal fail | `from_int(1), from_int(1)` | `should_panic` |
| assert_true bool | `from_bool(true)` | `None` |
| assert_true int nonzero | `from_int(5)` | `None` |
| assert_true fail | `from_bool(false)` | `should_panic` |
| assert_false pass | `from_bool(false)` | `None` |
| assert_false fail | `from_bool(true)` | `should_panic` |
| assert_is same value | same MbValue ptr | `None` |
| assert_is different | two distinct ptrs | `should_panic` |
| assert_is_none None | `MbValue::none()` | `None` |
| assert_is_none non-None | `from_int(1)` | `should_panic` |
| assert_in list found | item in list | `None` |
| assert_in list missing | item not in list | `should_panic` |
| assert_in str found | `"x"` in `"xyz"` | `None` |
| assert_in str missing | `"z"` in `"abc"` | `should_panic` |
| assert_raises | `from_ptr(new_str("ValueError"))` | returns dict with `expected` field |
| testcase | — | dict with `__class__ == "TestCase"`, `_failures == 0` |
| main | — | returns None, no panic |

### socket_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| socket new with explicit types | `from_int(2), from_int(1)` | family=2, type=1 |
| socket new with None types | `none(), none()` | family=2, type=1 (defaults) |
| connect sets connected | sock dict + addr | connected=true |
| connect null sock | `none()` | no panic |
| send Str data | sock + `new_str("hello")` | returns 5 |
| send non-Str data | sock + `from_int(0)` | returns 0 |
| close sets closed | sock dict | closed=true, connected=false |
| bind sets bound | sock + addr | bound=true |
| listen sets listening | sock | listening=true |
| gethostbyname | any | "127.0.0.1" |

### array_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| new with none initializer | `tc="i", none()` | empty data list |
| new with list initializer | `tc="i", list([1,2])` | data has 2 items |
| new non-str typecode | `from_int(0), none()` | typecode stored as "d" |
| append item | arr + `from_int(5)` | tolist()[0] == 5 |
| extend with list | arr + list([1,2,3]) | 3 items added |
| extend non-list | arr + `from_int(0)` | no items added |
| tolist empty | arr with no items | empty list |
| tobytes int items | arr with `[1,2,3]` | bytes == [1,2,3] |
| frombytes Bytes | arr + `new_bytes(vec![10,20])` | 2 ints appended |

### errno_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| strerror EPERM (1) | `from_int(1)` | "Operation not permitted" |
| strerror ENOENT (2) | `from_int(2)` | "No such file or directory" |
| strerror EINVAL (22) | `from_int(22)` | "Invalid argument" |
| strerror ECONNREFUSED (111) | `from_int(111)` | "Connection refused" |
| strerror unknown (999) | `from_int(999)` | "Unknown error" |
| errorcode dict | — | dict containing key "2" → str |

### traceback_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| format_exc | — | "NoneType: None" |
| print_exc | — | None (no panic) |
| format_exception None | `MbValue::none()` | "NoneType: None" |
| format_exception Str | `new_str("boom")` | "Exception: boom" |
| format_exception Instance with message | `ObjData::Instance` + fields `{message: "oops"}` | "SomeError: oops" |
| format_exception Instance no fields | `ObjData::Instance` + empty fields | just class_name |
| format_exception Dict _type+message | dict `{_type: "TypeError", message: "bad"}` | "TypeError: bad" |
| format_exception Dict _type only | dict `{_type: "ValueError"}` | "ValueError" |
| format_exception int | `from_int(42)` | "Exception: 42" |
| format_exception bool true | `from_bool(true)` | "Exception: True" |
| extract_tb | any | empty list |

### codecs_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| encode utf-8 ASCII | `"hello", "utf-8"` | bytes == b"hello" |
| encode ascii non-ASCII | `"héllo", "ascii"` | 'é' replaced with '?' |
| encode latin-1 in range | `"café", "latin-1"` | 'é' (0xe9) as u8 |
| encode latin-1 out of range | char > 255 (e.g. "\u{1F600}"), "latin-1" | replaced with '?' |
| encode non-Str obj | `from_int(5), enc` | None |
| encode default encoding | `str_val, none()` | utf-8 bytes |
| decode utf-8 bytes | `new_bytes(b"hello"), "utf-8"` | "hello" |
| decode ascii bad byte | `new_bytes(vec![200]), "ascii"` | "?" |
| decode latin-1 | `new_bytes(vec![0xe9]), "latin-1"` | "é" |
| decode Str passthrough | `new_str("x"), enc` | "x" |
| decode neither | `from_int(0), enc` | None |
| normalize utf_8 | `"utf_8"` → normalize → "utf-8" |
| lookup | `"ascii"` | tuple with name |
| stubs | register/register_error/lookup_error/open/getincremental* | None |

### logging_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| debug below level | set level to WARNING (30), call debug | suppressed (no panic) |
| warning at level | set level to WARNING (30), call warning | emitted |
| critical above level | any level, call critical | emitted |
| getlogger None | `MbValue::none()` | dict with name="root" |
| getlogger str | `new_str("mylogger")` | dict with name="mylogger" |
| basicconfig sets level | `from_int(10)` | LOG_LEVEL thread-local = 10, debug now emits |
| basicconfig non-int | `none()` | no change to level |
| extract_str int | `from_int(42)` | "42" |
| extract_str float | `from_float(3.14)` | "3.14" |
| extract_str bool true | `from_bool(true)` | "True" |
| extract_str None | `none()` | "None" |

### pickle_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| roundtrip None | `none()` | none() |
| roundtrip bool true | `from_bool(true)` | true |
| roundtrip bool false | `from_bool(false)` | false |
| roundtrip float | `from_float(3.14)` | ≈3.14 |
| roundtrip tuple | `new_tuple([1,2,3])` | 3-item tuple |
| roundtrip dict | `{"k": from_int(5)}` | dict with key "k" → 5 |
| roundtrip nested list | `[[1,2],[3,4]]` | nested structure preserved |
| loads ByteArray | `new_bytearray(b"I42")` | `from_int(42)` |
| loads Str | `new_str("I42")` | `from_int(42)` |
| loads non-bytes | `from_int(0)` | None |
| dump | serialize + discard → None |
| load | always None |
| deserialize unknown byte | `"X123"` | (None, 1) |
| deserialize empty | `""` | (None, 0) |

### threading_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| current_thread main | (default THREAD_NAME=None) | name="MainThread" |
| thread with str name | `target=none(), name=new_str("worker")` | dict name="worker" |
| thread with non-str name | `target=none(), name=from_int(0)` | dict name="Thread" |
| start/join lifecycle | thread dict | started=true → alive=false after join |
| lock acquire/release | lock dict | locked true → false |
| lock acquire null | `none()` | no panic |
| rlock | — | same as lock |
| event set/clear/is_set cycle | event dict | false → true → false |
| event_is_set null | `none()` | false |
| active_count | — | ≥ 1 |

### sqlite3_mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| connect with str path | `new_str(":memory:")` | dict class="Connection" |
| connect non-str | `from_int(0)` | database=":memory:" |
| cursor | conn | same value |
| execute CREATE TABLE | `"CREATE TABLE users (id INT)"` | table "users" in `_tables` dict |
| execute CREATE TABLE IF NOT EXISTS | `"CREATE TABLE IF NOT EXISTS t (x INT)"` | table "t" in `_tables` |
| execute non-CREATE | `"SELECT 1"` | stores `_last_sql`, no table created |
| fetchall empty _results | fresh conn | empty list |
| fetchone empty _results | fresh conn | None |
| commit | conn | None |
| close | conn | closed=true |
| executemany | sql + none() | delegates to execute |

---

## Batch B — core module scenarios
<!-- type: overview lang: markdown -->

### ffi/c_types.rs

| Scenario | Expected |
|----------|----------|
| `CType::Void.display_name()` | "void" |
| `CType::Int32.display_name()` | "int32_t" |
| `CType::ConstChar.display_name()` | "const char*" |
| `CType::MutChar.display_name()` | "char*" |
| `CType::Pointer(Box::new(CType::Int32)).display_name()` | "int32_t*" |
| `CType::ConstPointer(Box::new(CType::UInt8)).display_name()` | "const uint8_t*" |
| `CType::Named("MyStruct".into()).display_name()` | "MyStruct" |
| All 17 CType variants with display_name | correct strings |
| CType::Int32 == CType::Int32 | true |
| CType::Int32 == CType::Int64 | false |
| CFunction clone | equal to original |
| CHeader default + push function | functions.len() == 1 |
| CEnumVariant value Some(42) | value == Some(42) |
| CEnumVariant value None | value == None |

### driver/mod.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| CompilerSession::new | default config | session created |
| load_file valid | temp .py file | Ok(file_id) |
| load_file invalid | "/no/such/file.py" | Err |
| check EmitMode::Ast | `x = 1` with Ast emit | Ok(()) |
| check type error | `x: int = "str"` | Err |
| build EmitMode::Hir | valid source | Ok(vec![]) |
| build EmitMode::Mir | valid source | Ok(vec![]) |
| check_native_imports no project | session without project_config | Ok(()) skipped |
| check_dependencies no imports | source with no imports | no panics |

### codegen/cranelift/mod.rs

| Scenario | Expected |
|----------|----------|
| collect_used_externs empty module | empty HashSet |
| collect_used_externs with CallExtern | contains the extern name |
| collect_used_externs with MakeList | contains "mb_list_new" + "mb_list_append" |
| collect_used_externs with MakeDict | contains "mb_dict_new" + "mb_dict_setitem" |
| collect_used_externs with BinOp | contains "mb_dispatch_binop" |
| VarAlloc::new | map.is_empty() == true |
| VarAlloc::get new vreg | returns Variable with idx 0 |
| VarAlloc::get existing vreg | same Variable returned |

---

## Batch C — compiler pipeline scenarios
<!-- type: overview lang: markdown -->

### types/check_expr.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| IntLit | `x = 1` | type int |
| FloatLit | `x = 1.0` | type float |
| BoolLit | `x = True` | type bool |
| StrLit | `x = "s"` | type str |
| NoneLit | `x = None` | type none |
| Ident undefined | `y` (no prior def) | error emitted |
| UnaryOp Neg int | `-1` | type int |
| UnaryOp Neg non-numeric | `-True` | type error emitted |
| UnaryOp Not bool | `not True` | type bool |
| UnaryOp Not non-bool | `not 42` | type error emitted |
| UnaryOp BitNot int | `~5` | type int |
| BinOp int+int | `1 + 2` | type int |
| BinOp type mismatch | `1 + "s"` | type error emitted |

### lexer/token.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| unicode_name known | `"LATIN SMALL LETTER A"` | Some('a') |
| unicode_name unknown | `"NOT A NAME"` | None |
| escape \n | `"\\n"` | newline char |
| escape \t | `"\\t"` | tab char |
| escape \xNN valid | `"\\x41"` | 'A' |
| escape \N{name} known | `"\\N{LATIN SMALL LETTER A}"` | 'a' |
| escape \u4 | `"\\u0041"` | 'A' |
| escape \U8 | `"\\U00000041"` | 'A' |
| escape non-escape char | `"\\q"` | `\\q` kept |
| lex triple dquote | `"\"\"\"hello\"\"\""` | "hello" |
| lex fstr inner plain | `"f\"hello\""` | no exprs |
| lex fstr inner with expr | `"f\"{x}\""` | has expr braces |

### lower/ast_to_hir.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| ClassDef with method | `class Foo:\n  def bar(self): pass` | HIR class + method |
| With single ctx | `with open("f") as f: pass` | HIR with-block |
| Delete var | `del x` | HIR delete |
| Walrus | `if (n := 10) > 5: pass` | HIR assign + compare |
| AugAssign `//=` | `x //= 2` | HIR augassign floor-div |
| DictComp | `{k: v for k, v in items}` | HIR dict comprehension |

### driver/module_graph.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| relative import level 1 | `from . import foo` | resolves relative to parent_dir |
| stdlib import skipped | `import os` | silently skipped |
| cycle detection | A imports B, B imports A | topo_sort returns Err |
| path_to_module_name nested | `a/b/c.py` | `"a.b.c"` |

### lower/hir_to_mir.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| Raise with value | `raise ValueError("x")` | MIR raise inst |
| Assert true | `assert True` | no RuntimeError MIR |
| Assert false | `assert False` | RuntimeError MIR call |
| BinOp floor div | `a // b` | MIR BinOp FloorDiv |
| BinOp power | `a ** b` | MIR BinOp Pow |
| With block | `with ctx as v: body` | enter/exit calls in MIR |

### parser/expr_compound.rs

| Scenario | Input | Expected |
|----------|-------|----------|
| ternary basic | `"1 if True else 2"` | IfExpr AST node |
| ternary complex cond | `"x if a and b else y"` | nested BinOp in condition |
| lambda no params | `"lambda: 42"` | Lambda with empty params |
| lambda one param | `"lambda x: x"` | Lambda with 1 Regular param |
| lambda multiple | `"lambda x, y: x+y"` | Lambda with 2 params |
| lambda *args | `"lambda *args: args"` | Lambda with Star param |
| lambda **kwargs | `"lambda **kw: kw"` | Lambda with DoubleStar param |
| lambda with default | `"lambda x=1: x"` | param with default expr |
| yield bare | `"yield"` (in function) | Yield{value: None} |
| yield expr | `"yield x"` | Yield{value: Some(x)} |
| yield from | `"yield from gen"` | YieldFrom node |
| await | `"await coro"` | Await node |
| list comp with filter | `"[x for x in xs if x > 0]"` | ListComp with filter |
| dict comp | `"{k: v for k, v in items}"` | DictComp node |
| set comp | `"{x for x in xs}"` | SetComp node |

## Diagrams
<!-- type: overview lang: markdown -->

### Interaction
<!-- type: overview lang: markdown -->
<!-- score-td-placeholder -->

### Logic
<!-- type: overview lang: markdown -->
<!-- score-td-placeholder -->

### Dependencies
<!-- type: overview lang: markdown -->
<!-- score-td-placeholder -->

### State Machine
<!-- type: overview lang: markdown -->
<!-- score-td-placeholder -->

### Data Model
<!-- type: overview lang: markdown -->
<!-- score-td-placeholder -->

## API Spec
<!-- type: overview lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->

### RPC API
<!-- type: rpc-api lang: yaml -->
<!-- score-td-placeholder -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->

### Schema
<!-- type: schema lang: yaml -->
<!-- score-td-placeholder -->

### Config
<!-- type: config lang: yaml -->
<!-- score-td-placeholder -->

## Test Plan
<!-- type: overview lang: markdown -->

## Coverage Measurement
<!-- type: overview lang: markdown -->

```bash
cargo llvm-cov --branch --package cclab-mamba 2>&1 | tail -20
```

Run after each batch to confirm incremental progress. Measure per-file coverage with:

```bash
cargo llvm-cov --branch --package cclab-mamba report --lcov | lcov --list -
```

## Acceptance Criteria
<!-- type: overview lang: markdown -->

| Batch | Files | Line target | Branch target |
|-------|-------|-------------|---------------|
| A — stdlib | argparse, platform, unittest, socket, array, errno, traceback, codecs, logging, pickle, threading, sqlite3 | 100% each | 100% each |
| B — core | ffi/c_types, driver/mod, codegen/cranelift/mod | 100% each | 100% each |
| C — pipeline | types/check_expr, cranelift/aot, cranelift/jit, lexer/token, lower/ast_to_hir, driver/module_graph, lower/hir_to_mir, parser/expr_compound | 100% each | 100% each |

## Test Constraints
<!-- type: overview lang: markdown -->

| Module | Constraint |
|--------|------------|
| `socket_mod` | Use `127.0.0.1:0` (port=0 → OS-assigned); no mocks; no live DNS |
| `threading_mod` | Deterministic sync via `std::sync::Barrier` or channels; no `std::thread::sleep` |
| `sqlite3_mod` | `:memory:` database only; no temp file I/O |
| `ffi/c_types` | Pure Rust type assertions; no C headers or FFI calls |
| `driver/mod` | Write `.py` source to `tempfile::NamedTempFile` for file-path tests |
| `codegen/cranelift/*` | Build minimal MIR fixtures inline; no external `.py` files needed |

## Test Placement Rules
<!-- type: overview lang: markdown -->

- Inline `#[cfg(test)] mod tests { ... }` at the bottom of each source file
- Integration tests (cross-module scenarios) → `crates/mamba/tests/stdlib_coverage_remaining_tests.rs`
- No `#[cfg(coverage_not)]` or `// coverage: off` annotations
- All tests must be deterministic and pass under `cargo test` without side effects

## Verification Steps
<!-- type: overview lang: markdown -->

1. Run `cargo test -p mamba` — all tests pass
2. Run `cargo llvm-cov --branch -p mamba` — all 23 target files at 100% line + 100% branch
3. Run `cargo clippy -p mamba` — no new warnings introduced
4. Run `cargo build -p mamba` — clean build

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  # Batch A — stdlib modules
  - path: crates/mamba/src/runtime/stdlib/argparse_mod.rs
    action: MODIFY
    desc: Expand #[cfg(test)] block — add tests for extract_str (non-Str/null), mb_argparse_new (non-Str desc), mb_argparse_add_argument (null parser), mb_argparse_parse_args (empty names, fewer env args than names)

  - path: crates/mamba/src/runtime/stdlib/platform_mod.rs
    action: MODIFY
    desc: Replace stub test with full coverage — test all mb_platform_* functions, mb_platform_node HOSTNAME/HOST/fallback branches

  - path: crates/mamba/src/runtime/stdlib/unittest_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for to_snake, values_equal (all type combos), mb_unittest_assert_not_equal, assert_true/false (both branches), assert_is, assert_is_none, assert_in (list/str/missing), assert_raises, mb_unittest_testcase, mb_unittest_main

  - path: crates/mamba/src/runtime/stdlib/socket_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for mb_socket_new (None family/type), connect/bind/listen/close (null sock), send (non-Str), recv, gethostname (HOSTNAME/HOST/fallback), gethostbyname

  - path: crates/mamba/src/runtime/stdlib/array_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for mb_array_new (list initializer, non-Str typecode), mb_array_extend (list + non-list), mb_array_tobytes, mb_array_frombytes (Bytes + ByteArray)

  - path: crates/mamba/src/runtime/stdlib/errno_mod.rs
    action: MODIFY
    desc: Expand test block — test mb_errno_strerror for all matched codes (1,2,4,5,9,11,13,22,32,110,111,113) and unknown code; test mb_errno_errorcode dict entry

  - path: crates/mamba/src/runtime/stdlib/traceback_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for format_exception_value (Instance with/without message, Dict with _type+message, Dict _type only, int, bool true/false), mb_traceback_print_exc, mb_traceback_extract_tb

  - path: crates/mamba/src/runtime/stdlib/codecs_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for extract_bytes, normalize_encoding (ascii/latin-1/unknown), encode (ascii non-ASCII, latin-1 in/out-of-range), decode (ascii bad byte, latin-1, Str passthrough, neither), stub functions (register/register_error/lookup_error/open/getincremental*/getreader/getwriter), convenience encode/decode functions

  - path: crates/mamba/src/runtime/stdlib/logging_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for extract_str (int/float/bool/None/other), log_at_level suppression, mb_logging_error, mb_logging_critical, mb_logging_getlogger (None name), mb_logging_basicconfig (non-int level)

  - path: crates/mamba/src/runtime/stdlib/pickle_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for serialize (None/bool/float/dict/tuple/other), deserialize (all byte prefixes, empty input, unknown byte), mb_pickle_loads (ByteArray/Str/non-bytes), mb_pickle_dump, mb_pickle_load

  - path: crates/mamba/src/runtime/stdlib/threading_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for mb_threading_current_thread (None vs Some THREAD_NAME), mb_threading_thread (non-Str name), start/join null, lock_acquire/release null, rlock, event_set/clear null, event_is_set null, active_count

  - path: crates/mamba/src/runtime/stdlib/sqlite3_mod.rs
    action: MODIFY
    desc: Expand test block — add tests for mb_sqlite3_connect (non-Str), cursor, extract_table_name (IF NOT EXISTS, no TABLE), execute (non-CREATE, null conn), fetchall/fetchone (empty _results, null), commit, executemany

  # Batch B — core modules
  - path: crates/mamba/src/ffi/c_types.rs
    action: MODIFY
    desc: Add full #[cfg(test)] block — test display_name for all 17 CType variants (including Pointer/ConstPointer nesting, Named), test Clone/PartialEq on CFunction/CParam/CStruct/CField/CEnum/CEnumVariant/CHeader, test CHeader::default()

  - path: crates/mamba/src/driver/mod.rs
    action: MODIFY
    desc: Expand test block — add tests for load_file (invalid path), check (EmitMode::Ast, type error), build (EmitMode::Hir, EmitMode::Mir), check_native_imports (no project, star import, no expose entry), check_dependencies (no imports)

  - path: crates/mamba/src/codegen/cranelift/mod.rs
    action: MODIFY
    desc: Add #[cfg(test)] block — test collect_used_externs (empty, CallExtern, MakeList, MakeDict, GetAttr, SetAttr, GetItem, SetItem, MakeTuple, BinOp, UnaryOp, other), test VarAlloc::new and get (new vreg, existing vreg)

  # Batch C — compiler pipeline
  - path: crates/mamba/src/types/check_expr.rs
    action: MODIFY
    desc: Add #[cfg(test)] block — test check_expr for Ellipsis, BytesLit, ComplexLit, undefined Ident, UnaryOp Neg/Not/BitNot (error branches on wrong types), BinOp type mismatch

  - path: crates/mamba/src/codegen/cranelift/aot.rs
    action: MODIFY
    desc: Add #[cfg(test)] block — test CraneliftAotCompiler::new, codegen with minimal MirModule (empty function, function with Const int return)

  - path: crates/mamba/src/codegen/cranelift/jit.rs
    action: MODIFY
    desc: Add #[cfg(test)] block — test CraneliftJitBackend::new, new_with_externals (empty externals), codegen with minimal function returning constant

  - path: crates/mamba/src/lexer/token.rs
    action: MODIFY
    desc: Expand existing test block — add tests for unicode_name_to_char (known, unknown), apply_escape_sequences (all escape variants including \xNN, \N{name}, \u, \U, non-escape), lex_triple_dquote/squote, lex_fstr_inner (plain + with expr)

  - path: crates/mamba/src/lower/ast_to_hir.rs
    action: MODIFY
    desc: Expand existing test block — add tests for ClassDef (with method, with base), Stmt::With, Stmt::Delete, Stmt::Global/Nonlocal, Expr::Walrus, AugAssign (//=, **=, ^=, |=, &=), DictComp, SetComp

  - path: crates/mamba/src/driver/module_graph.rs
    action: MODIFY
    desc: Expand existing test block — add tests for relative import level 1/2, cycle detection, path_to_module_name nested (a/b/c.py)

  - path: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    desc: Expand existing test block — add tests for Raise (with value, bare), Assert (true/false), BinOp FloorDiv/Pow/bitwise ops, Stmt::With lowering, Stmt::Delete lowering

  - path: crates/mamba/src/parser/expr_compound.rs
    action: MODIFY
    desc: Expand existing test block — add tests for lambda *args/**kwargs/default, yield (bare/expr/from), await, list-comp with filter, DictComp, SetComp

  # New integration test file
  - path: crates/mamba/tests/stdlib_coverage_remaining_tests.rs
    action: CREATE
    desc: Integration tests for cross-module scenarios — socket loopback connect/send/recv, threading concurrent put/get with Barrier synchronization, sqlite3 :memory: table create/insert/fetchone/fetchall pipeline
```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
# score-td-placeholder
```

## Component
<!-- type: component lang: yaml -->

```yaml
# score-td-placeholder
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
# score-td-placeholder
```

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->
