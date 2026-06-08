---
change: mamba-p3
date: 2026-02-22
---

# Context Clarifications

## Issue #397 — subprocess module
- **Scope**: `subprocess.run()`, `subprocess.Popen()`, `CompletedProcess`, `PIPE`/`DEVNULL` constants, `check_output()`/`check_call()`
- **Backend**: Rust `std::process::Command`. Popen wraps `std::process::Child` with pid tracking.
- **Decision**: No shell=True support (security risk). `args` accepts list only.
- **Error handling**: `CalledProcessError` exception when check=True and returncode != 0.

## Issue #398 — csv module
- **Scope**: `csv.reader()`, `csv.writer()`, `csv.DictReader()`, `csv.DictWriter()` with delimiter/quotechar support.
- **Backend**: Simple Rust string parsing. No external `csv` crate needed for P3 scope.
- **Decision**: Support RFC 4180 quoting (double-quote escaping). Skip Sniffer/dialect registry.
- **Dependency**: Uses file I/O from #379 (already implemented in P2).

## Issue #399 — argparse module
- **Scope**: `ArgumentParser`, `add_argument()`, `parse_args()`, positional/optional args, `--help` auto-gen.
- **Backend**: Pure Mamba-runtime implementation (no clap wrapping). Namespace is Instance with field access.
- **Decision**: Skip subparsers for P3. Focus on core positional/optional/flag arguments with type/default/help/required/choices.
- **Note**: `sys.argv` already available from sys_mod (P2).

## Issue #400 — logging module
- **Scope**: `logging.debug/info/warning/error/critical()`, `getLogger()`, `basicConfig()`, StreamHandler/FileHandler, Formatter.
- **Backend**: Simple stderr/file output. No Rust `log` crate dependency — self-contained.
- **Decision**: Global logger registry via thread-local HashMap<String, Logger>. Default level WARNING. ISO 8601 timestamps.
- **Format**: Default `"%(levelname)s:%(name)s:%(message)s"`. Support `%(asctime)s`, `%(levelname)s`, `%(name)s`, `%(message)s`, `%(lineno)d`.

## Issue #401 — typing module runtime
- **Scope**: Make `from typing import X` work. All typing constructs are runtime no-ops / sentinel objects.
- **Backend**: Module attrs are MbValue sentinels. `Optional[T]` is `Union[T, None]`. TypeVar creates named placeholder.
- **Decision**: No runtime enforcement. `get_type_hints()` returns empty dict (annotations not stored at runtime). `TYPE_CHECKING = False`.
- **Note**: Type checker already handles these internally; this is purely for import compatibility.

## Issue #405 — bytes/bytearray (ALREADY CLOSED)
- **Status**: Closed. Was P1, already implemented in mamba-p1 batch.
- **Note**: Referenced as dependency by #418, #442, #445, #451. Those issues can proceed since bytes support exists.

## Issue #407 — metaclasses/ABC (ALREADY CLOSED)
- **Status**: Closed. Was P1, already implemented in mamba-p1 batch.
- **Note**: Referenced as dependency by #419. unittest can proceed since ABC/metaclass support exists.

## Issue #417 — threading/multiprocessing
- **Scope**: `threading.Thread`, `.start()/.join()/.is_alive()`, `Lock`, `Event`, `current_thread()`, `active_count()`.
- **Backend**: Rust `std::thread` for threads, `std::sync::Mutex` for Lock, `std::sync::Condvar` for Event.
- **Decision**: Skip multiprocessing (R6-R8) for P3 — too complex with NaN-boxed runtime. Focus on threading only.
- **Constraint**: Each thread gets its own runtime scope copy (thread-local MbValue). No shared mutable state across threads except through Lock-protected structures.
- **GIL**: Implement a global interpreter lock (simple Mutex) to serialize access to shared runtime state.

## Issue #418 — socket/http networking
- **Scope**: `socket.socket()`, `.connect()/.bind()/.listen()/.accept()/.send()/.recv()/.close()/.settimeout()`. `urllib.request.urlopen()`. Skip HTTPServer.
- **Backend**: Rust `std::net::TcpStream`/`TcpListener` for socket. Rust `ureq` or `std::net` for HTTP.
- **Decision**: TCP only (skip UDP for P3). `urlopen()` returns response with `.read()`, `.status`, `.headers`. Skip HTTPS for P3 (or use ureq which handles it).
- **Dependency**: #405 (bytes) is closed/done. Socket send/recv work with bytes.

## Issue #419 — unittest module
- **Scope**: `TestCase` base class, assert methods, setUp/tearDown, `unittest.main()` runner, skip decorators.
- **Backend**: TestCase is a regular class. Runner discovers TestCase subclasses via class registry, runs methods starting with `test_`.
- **Decision**: Test discovery scans current module's classes. `assertRaises` uses context manager pattern. Skip TestSuite/TestLoader complexity.
- **Dependency**: #407 (ABC/metaclasses) is closed/done.

## Issue #441 — eval/exec and globals/locals
- **Scope**: `eval()`, `exec()`, `compile()`, `globals()`, `locals()`.
- **Decision**: Implement `eval()` for simple expressions only (arithmetic, string ops, variable lookups). `exec()` supports simple statements (assignment, print, if/for). Both re-enter the parser+interpreter pipeline at runtime.
- **Backend**: Call into Mamba's own parser/HIR/MIR pipeline at runtime. Expensive but correct.
- **Constraint**: `compile()` returns an opaque code object (MbValue wrapping compiled MIR). `globals()`/`locals()` return dict snapshots of current scope.
- **Risk**: Circular dependency with parser crate. May need to extract a lightweight eval path.

## Issue #442 — pickle module
- **Scope**: `pickle.dumps()`, `pickle.loads()`, `pickle.dump()`, `pickle.load()`. Support `__getstate__`/`__setstate__`.
- **Backend**: Custom binary format (not CPython-compatible). Serialize MbValue tree to bytes using a simple type-tagged format.
- **Decision**: Skip `__reduce__`/`__reduce_ex__` (R6) for P3. Support: None, bool, int, float, str, bytes, list, tuple, dict, set, Instance with __getstate__.
- **Dependency**: #405 (bytes) is closed/done.

## Issue #444 — sqlite3 module
- **Scope**: `sqlite3.connect()`, cursor with `execute()`/`fetchone()`/`fetchall()`, `commit()`/`rollback()`/`close()`, context manager, parameterized queries.
- **Backend**: Rust `rusqlite` crate wrapping SQLite C library. Connection stored as ObjData::NativeHandle.
- **Decision**: Skip Row factory (R10) for P3. `cursor.description` returns column names. Parameter binding uses `?` placeholders.
- **Note**: Add `rusqlite` dependency to cclab-mamba Cargo.toml.

## Issue #445 — gzip/zipfile/tarfile compression
- **Scope**: `gzip.compress()`/`decompress()`/`open()`. `zipfile.ZipFile` with read/extractall/write. `tarfile.open()` with extractall/getmembers/add.
- **Backend**: Rust `flate2` for gzip, `zip` crate for zipfile, `tar` crate for tarfile.
- **Decision**: Focus on in-memory compress/decompress for gzip. ZipFile read-only for P3 (skip write). Tarfile read-only for P3.
- **Dependency**: #405 (bytes) is closed/done.

## Issue #446 — pprint module
- **Scope**: `pprint.pprint()`, `pprint.pformat()`, `PrettyPrinter(indent, width, depth)`.
- **Backend**: Recursive value-to-string with indentation tracking. Uses existing `value_to_string()` from string_ops.rs as base.
- **Decision**: Support dict, list, tuple, set, frozenset nesting. Depth limit truncates with `...`. Default width=80, indent=1.

## Issue #448 — textwrap module
- **Scope**: `textwrap.wrap()`, `fill()`, `dedent()`, `indent()`, `shorten()`.
- **Backend**: Pure string operations in Rust. Word-wrap by splitting on whitespace and tracking line width.
- **Decision**: Full scope — all 5 functions are simple and well-defined. `shorten()` uses `[...]` as default placeholder.

## Issue #449 — xml/html.parser modules
- **Scope**: `html.parser.HTMLParser` (event-based), `html.escape()`/`unescape()`. `xml.etree.ElementTree` with parse/fromstring/find/findall/tostring.
- **Backend**: Simple recursive-descent parser for XML/HTML (no external crate). Entity encoding via lookup table.
- **Decision**: HTMLParser: handle_starttag/endtag/data callbacks only. ET: basic XPath-like find (tag name only, no full XPath). Skip namespace support.
- **Note**: HTML parser is lenient (doesn't require well-formed). XML parser requires well-formed.

## Issue #451 — array module
- **Scope**: `array.array(typecode, initializer)` with typecodes b/h/i/l/q/f/d. Methods: append/extend/insert/pop/remove/reverse. tobytes()/frombytes()/tolist().
- **Backend**: Rust `Vec<u8>` backing store with typecode-aware access. Store raw bytes, interpret per typecode.
- **Decision**: Skip buffer protocol (R6) for P3. Focus on typed storage + conversion methods.
- **Dependency**: #405 (bytes) is closed/done.

## Issue #452 — string module
- **Scope**: All constants (ascii_lowercase, digits, etc.), `string.Template` with substitute/safe_substitute, `string.capwords()`.
- **Backend**: Constants are pre-built MbValue strings. Template uses simple `$var` / `${var}` substitution via regex-like matching.
- **Decision**: Skip `string.Formatter` (R4) for P3. Full scope on constants + Template + capwords.

## Issue #453 — complex number full operations
- **Scope**: `complex(real, imag)` constructor, arithmetic (+,-,*,/,**), `.real`/`.imag` properties, `abs()`, `.conjugate()`, mixed int/float arithmetic, `cmath` module.
- **Backend**: Store complex as ObjData::Complex(f64, f64). Arithmetic in Rust. cmath functions use `num::Complex` or manual formulas.
- **Decision**: Full scope including cmath. ComplexLit already parsed in AST. Need to add ObjData::Complex variant to rc.rs and wire through all match sites (similar to FrozenSet addition in P2).
- **Risk**: Adding ObjData::Complex requires updating ~7 match exhaustiveness sites across the codebase.
