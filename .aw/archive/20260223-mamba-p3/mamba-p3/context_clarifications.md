---
change: mamba-p3
date: 2026-02-23
---

# Context Clarifications

## Q1: General
- **Question**: What is the scope and implementation strategy for the subprocess module?
- **Answer**: subprocess.run(), subprocess.Popen(), CompletedProcess, PIPE/DEVNULL constants, check_output()/check_call(). No shell=True support (security risk). args accepts list only. Rust std::process::Command backend. Popen wraps std::process::Child with pid tracking. CalledProcessError exception when check=True and returncode != 0.
- **Rationale**: 

## Q2: General
- **Question**: What is the scope and implementation strategy for the csv module?
- **Answer**: csv.reader(), csv.writer(), csv.DictReader(), csv.DictWriter() with delimiter/quotechar support. Simple Rust string parsing backend (no external csv crate). Support RFC 4180 quoting (double-quote escaping). Skip Sniffer/dialect registry. Dependency: uses file I/O from #379 (already implemented in P2).
- **Rationale**: 

## Q3: General
- **Question**: What is the scope and implementation strategy for the argparse module?
- **Answer**: ArgumentParser, add_argument(), parse_args(), positional/optional args, --help auto-gen. Pure Mamba-runtime implementation (no clap wrapping). Namespace is Instance with field access. Skip subparsers for P3. Focus on core positional/optional/flag arguments with type/default/help/required/choices. sys.argv already available from sys_mod (P2).
- **Rationale**: 

## Q4: General
- **Question**: What is the scope and implementation strategy for the logging module?
- **Answer**: logging.debug/info/warning/error/critical(), getLogger(), basicConfig(), StreamHandler/FileHandler, Formatter. Simple stderr/file output (no Rust log crate). Global logger registry via thread-local HashMap<String, Logger>. Default level WARNING. ISO 8601 timestamps. Default format "%(levelname)s:%(name)s:%(message)s". Support %(asctime)s, %(levelname)s, %(name)s, %(message)s, %(lineno)d.
- **Rationale**: 

## Q5: General
- **Question**: What is the scope and implementation strategy for the threading module?
- **Answer**: threading.Thread, .start()/.join()/.is_alive(), Lock, Event, current_thread(), active_count(). Rust std::thread for threads, std::sync::Mutex for Lock, std::sync::Condvar for Event. Skip multiprocessing (R6-R8) for P3 — too complex with NaN-boxed runtime. Focus on threading only. Constraint: each thread gets its own runtime scope copy (thread-local MbValue). Global Interpreter Lock (GIL) implemented to serialize access to shared state.
- **Rationale**: 

## Q6: General
- **Question**: What is the scope and implementation strategy for the socket/http modules?
- **Answer**: socket.socket(), .connect()/.bind()/.listen()/.accept()/.send()/.recv()/.close()/.settimeout(). urllib.request.urlopen(). TCP only (skip UDP for P3). Skip HTTPServer. Rust std::net::TcpStream/TcpListener for socket. Rust ureq or std::net for HTTP. urlopen() returns response with .read(), .status, .headers. Skip HTTPS for P3 (or use ureq which handles it). Dependency: #405 (bytes) is closed/done.
- **Rationale**: 

## Q7: General
- **Question**: What is the scope and implementation strategy for eval/exec builtins?
- **Answer**: eval(), exec(), compile(), globals(), locals(). Implement eval() for simple expressions only (arithmetic, string ops, variable lookups). exec() supports simple statements (assignment, print, if/for). Both re-enter the parser+interpreter pipeline at runtime. Expensive but correct. compile() returns an opaque code object (MbValue wrapping compiled MIR). globals()/locals() return dict snapshots of current scope. Risk: circular dependency with parser crate. May need to extract a lightweight eval path.
- **Rationale**: 

## Q8: General
- **Question**: What is the scope and implementation strategy for the sqlite3 module?
- **Answer**: sqlite3.connect(), cursor with execute()/fetchone()/fetchall(), commit()/rollback()/close(), context manager, parameterized queries. Rust rusqlite crate wrapping SQLite C library. Connection stored as ObjData::NativeHandle. Skip Row factory (R10) for P3. cursor.description returns column names. Parameter binding uses ? placeholders. Add rusqlite dependency to cclab-mamba Cargo.toml.
- **Rationale**: 

## Q9: General
- **Question**: What is the scope and implementation strategy for complex number support?
- **Answer**: complex(real, imag) constructor, arithmetic (+,-,*,/,**), .real/.imag properties, abs(), .conjugate(), mixed int/float arithmetic, cmath module. Store complex as ObjData::Complex(f64, f64). Arithmetic in Rust. cmath functions use num::Complex or manual formulas. Full scope including cmath. ComplexLit already parsed in AST. Add ObjData::Complex variant to rc.rs and wire through all match sites. Risk: updating ~7 match exhaustiveness sites across the codebase.
- **Rationale**: 

## Q10: General
- **Question**: What is the git workflow for this change?
- **Answer**: in_place workflow. Changes will be committed directly to the feature branch.
- **Rationale**: 

