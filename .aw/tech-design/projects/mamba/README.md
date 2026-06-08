# Mamba Spec Index

Specifications for the Mamba Python compiler and runtime. Directory structure mirrors `crates/mamba/src/` — each spec maps to 1-3 closely related `.rs` files.

## source/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [source-and-diagnostics](source/source-and-diagnostics.md) | source/, diagnostic/, error.rs | Source file management, spans, error reporting |

## lexer/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [tokens-and-indent](lexer/tokens-and-indent.md) | lexer/ | Tokenization, INDENT/DEDENT, f-string lexing |

## parser/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [ast](logic/parser/ast.md) | ast.rs, type_expr.rs | AST node definitions, type expression nodes |
| [expressions](logic/parser/expressions.md) | mod.rs, expr.rs, expr_compound.rs | Expression parsing, precedence climbing |
| [statements](logic/parser/statements.md) | stmt.rs, stmt_compound.rs | Statement parsing, PEP 695 generics |
| [patterns](logic/parser/patterns.md) | pattern.rs | Pattern matching (PEP 634) |

## resolve/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [name-resolution](resolve/name-resolution.md) | pass.rs, scope.rs | Variable classification, scope chains |

## types/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [type-checker](types/type-checker.md) | check.rs, check_expr.rs, check_stmt.rs, context.rs | Bidirectional type inference |
| [type-representations](types/type-representations.md) | ty.rs, builtins.rs | Ty enum, built-in type registry |
| [generics-and-protocols](types/generics-and-protocols.md) | generic.rs, protocol.rs | PEP 695 generics, structural subtyping |

## hir/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [hir](hir/hir.md) | mod.rs | High-level IR after desugaring |

## lower/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [ast-to-hir](lower/ast-to-hir.md) | ast_to_hir.rs | AST → HIR desugaring |
| [hir-to-mir](lower/hir-to-mir.md) | hir_to_mir.rs | HIR → MIR lowering, comprehensions, pattern match |

## mir/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [mir](mir/mir.md) | mod.rs | Mid-level IR instruction set |

## codegen/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [cranelift](generate/codegen/cranelift.md) | cranelift/mod.rs, cranelift/marshal.rs | Cranelift backend core, value marshaling |
| [cranelift-jit](generate/codegen/cranelift-jit.md) | cranelift/jit.rs | JIT execution, symbol wiring |
| [cranelift-aot](generate/codegen/cranelift-aot.md) | cranelift/aot.rs | AOT object file emission |
| [llvm](generate/codegen/llvm.md) | llvm.rs | LLVM AOT backend |

## runtime/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [value-and-rc](runtime/value-and-rc.md) | value.rs, rc.rs | NaN-boxed MbValue, reference counting |
| [gc](runtime/gc.md) | gc.rs | Cycle-detecting garbage collector |
| [symbols](runtime/symbols.md) | symbols.rs | Runtime symbol registry (mb_*) |
| [class](runtime/class.md) | class.rs | Class system, MRO, dispatch, magic methods |
| [builtins](runtime/builtins.md) | builtins.rs | Built-in functions (enumerate, zip, sorted, ...) |
| [string-ops](runtime/string-ops.md) | string_ops.rs | String methods, f-string formatting |
| [list-ops](runtime/list-ops.md) | list_ops.rs | List methods (append, sort, slice, ...) |
| [dict-ops](runtime/dict-ops.md) | dict_ops.rs | Dict methods (get, keys, values, ...) |
| [set-ops](runtime/set-ops.md) | set_ops.rs | Set operations (union, intersection, ...) |
| [tuple-ops](runtime/tuple-ops.md) | tuple_ops.rs | Tuple operations |
| [bytes-ops](runtime/bytes-ops.md) | bytes_ops.rs | Bytes/ByteArray operations |
| [exception](runtime/exception.md) | exception.rs | Exception hierarchy, ExceptionGroup |
| [iter](runtime/iter.md) | iter.rs | Iteration protocol, built-in iterators |
| [generator](runtime/generator.md) | generator.rs | Generator state machines |
| [closure](runtime/closure.md) | closure.rs | Closures, free variable capture |
| [module](runtime/module.md) | module.rs | Import system, sys.modules |
| [file-io](runtime/file-io.md) | file_io.rs | File I/O (open, read, write, close) |
| [async](runtime/async.md) | async_rt.rs, async_task.rs | Async/await, coroutine scheduling |

## stdlib/
<!-- type: doc lang: markdown -->

Standard library modules in `runtime/stdlib/`.

| Spec | Files | Description |
|------|-------|-------------|
| [sys](stdlib/sys.md) | sys_mod.rs | argv, path, exit, version |
| [os](stdlib/os.md) | os_mod.rs | File system ops, environ, os.path |
| [math](stdlib/math.md) | math_mod.rs, cmath_mod.rs | Math functions, complex math |
| [json](stdlib/json.md) | json_mod.rs | JSON encode/decode |
| [time](stdlib/time.md) | time_mod.rs | Monotonic clock, sleep, strftime |
| [re](stdlib/re.md) | re_mod.rs | Regular expressions |
| [datetime](stdlib/datetime.md) | datetime_mod.rs | Date/time types, timedelta |
| [collections](stdlib/collections.md) | collections_mod.rs | defaultdict, Counter, deque |
| [itertools](stdlib/itertools.md) | itertools_mod.rs | chain, product, permutations, ... |
| [functools](stdlib/functools.md) | functools_mod.rs | reduce, partial, lru_cache |
| [pathlib](stdlib/pathlib.md) | pathlib_mod.rs | Path class |
| [random](stdlib/random.md) | random_mod.rs | PRNG, randint, choice, shuffle |
| [io](stdlib/io.md) | io_mod.rs, csv_mod.rs, pprint_mod.rs | StringIO/BytesIO, CSV, pprint |
| [struct-and-binary](stdlib/struct-and-binary.md) | struct_mod.rs, base64_mod.rs, pickle_mod.rs | Binary packing, base64, pickle |
| [hashlib](stdlib/hashlib.md) | hashlib_mod.rs | MD5, SHA-256, SHA-512 |
| [fs-utils](stdlib/fs-utils.md) | shutil_mod.rs, tempfile_mod.rs, glob_mod.rs | File utils, temp files, glob |
| [numeric](stdlib/numeric.md) | decimal_mod.rs, fractions_mod.rs, array_mod.rs | Decimal, Fraction, array |
| [enum-and-dataclasses](stdlib/enum-and-dataclasses.md) | enum_mod.rs, dataclasses_mod.rs | Enum, @dataclass |
| [operator-and-copy](stdlib/operator-and-copy.md) | operator_mod.rs, copy_mod.rs | Operator wrappers, copy/deepcopy |
| [text-processing](stdlib/text-processing.md) | textwrap_mod.rs, string_constants_mod.rs | textwrap, string constants |
| [typing-and-inspect](stdlib/typing-and-inspect.md) | typing_mod.rs, inspect_mod.rs | Type hints, introspection |
| [diagnostics-utils](stdlib/diagnostics-utils.md) | contextlib_mod.rs, weakref_mod.rs, traceback_mod.rs, warnings_mod.rs | Context managers, tracebacks, warnings |
| [concurrency](stdlib/concurrency.md) | threading_mod.rs, subprocess_mod.rs | Threading, subprocess |
| [network](stdlib/network.md) | socket_mod.rs, http_mod.rs | Sockets, HTTP client |
| [testing](stdlib/testing.md) | unittest_mod.rs | unittest framework |
| [archive-and-compression](stdlib/archive-and-compression.md) | gzip_mod.rs, zipfile_mod.rs, tarfile_mod.rs | gzip, zip, tar |
| [database](stdlib/database.md) | sqlite3_mod.rs | SQLite3 interface |
| [logging](stdlib/logging.md) | logging_mod.rs | Logging framework |
| [argparse](stdlib/argparse.md) | argparse_mod.rs | Argument parsing |
| [markup](stdlib/markup.md) | xml_mod.rs, html_parser_mod.rs | XML, HTML parsing |

## ffi/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [c-parser-and-types](ffi/c-parser-and-types.md) | c_parser.rs, c_types.rs, type_map.rs | C header parsing, type mapping |
| [bindings-and-stubs](ffi/bindings-and-stubs.md) | cbindgen.rs, stub_gen.rs | Binding and stub generation |
| [memory-and-safety](ffi/memory-and-safety.md) | memory.rs, safety.rs | FFI memory management, safety |

## driver/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [compiler-driver](driver/compiler-driver.md) | driver/mod.rs, driver/config.rs | Pipeline orchestration, backend selection |
| [repl](driver/repl.md) | driver/repl.rs | Interactive REPL |

## config/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [config-schema](config/config-schema.md) | config/schema.rs | MambaConfig, TOML/CLI parsing |

## testing/
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [test-harness](testing/test-harness.md) | tests/fixture_tests.rs | Fixture-based testing, CPython integration |
