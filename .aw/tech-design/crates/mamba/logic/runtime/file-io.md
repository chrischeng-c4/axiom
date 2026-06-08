---
id: file-io
title: File I/O — open, read, write, close
crate: mamba
files:
  - crates/mamba/src/runtime/file_io.rs
status: source-of-truth
last_synced_at: 2026-04-27
last_synced_commit: 146f6c211
---

# File I/O

Mamba file objects are *not* heap `MbObject` — they are thread-local
handle table entries (`MbFile { reader, writer, mode, closed }`)
returned to JIT code as `i64` IDs. This sidesteps `ObjData` having to
hold non-`Send` `BufReader<File>` / `File` types.

Three load-bearing invariants:

1. **File handles are thread-local** — opening a file on thread A
   produces a handle that thread B cannot use. The thread-local
   `HashMap<u64, MbFile>` registry is intentional; cross-thread file
   sharing is not supported and is unlikely to be needed for
   conformance work.
2. **Mode parsing strictly mirrors Python** — `'r'`, `'w'`, `'a'`,
   `'rb'`, `'wb'`, `'ab'`, `'r+'`, `'w+'` etc. each open a fresh
   `BufReader` / `File` slot; binary modes do not currently differ
   from text modes (no decode step), which is an open gap for
   `bytes`-returning reads.
3. **`is_file_handle(id)` gates dispatch** — `mb_iter` (see
   `iter.md`) and other places that may receive a generic `i64`
   handle must check `is_file_handle` to decide whether the value
   is a file iterator vs. a generator vs. a closure. Disjoint ID
   ranges across registries (this one starts at 1, generator at 1,
   iter at `0x1_0000_0000`) keep collisions resolvable but are NOT
   sufficient on their own — the explicit registry-membership check
   is required.

Open gap: `bytes`-returning reads when mode contains `b`. Today the
binary mode is parsed but the read path still constructs a Str. CPython
returns `bytes`. Out of scope for this spec; tracked separately.

## Type model
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: file-io-types
types:
  MbFile:        { kind: struct, label: "reader + writer + mode + closed" }
  Files:         { kind: struct, label: "thread_local HashMap<u64, MbFile>" }
  NextFileId:    { kind: struct, label: "thread_local Cell<u64> starting at 1" }
  BufReader:     { kind: struct, label: "std::io::BufReader<fs::File>" }
  File:          { kind: struct, label: "std::fs::File" }
  IterModule:    { kind: struct, label: "from runtime::iter (file as iterator yields lines)" }
  ExceptionMod:  { kind: struct, label: "exception.rs (TypeError / OSError)" }
edges:
  - { from: Files,      to: MbFile,    kind: owns }
  - { from: MbFile,     to: BufReader, kind: owns }
  - { from: MbFile,     to: File,      kind: owns,       label: "writer" }
  - { from: NextFileId, to: Files,     kind: references, label: "alloc_file_id" }
  - { from: IterModule, to: Files,     kind: references, label: "is_file_handle gate" }
  - { from: MbFile,     to: ExceptionMod, kind: references, label: "TypeError / OSError" }
---
classDiagram
    class MbFile
    class Files
    class NextFileId
    class BufReader
    class File
    class IterModule
    class ExceptionMod
    Files --> MbFile : owns
    MbFile --> BufReader : reader
    MbFile --> File : writer
    NextFileId --> Files : alloc id
    IterModule --> Files : is_file_handle
    MbFile --> ExceptionMod : raise
```

## File state shape
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "file-io-types"
$defs:
  MbFile:
    type: object
    x-rust-type: MbFile
    properties:
      reader:
        oneOf:
          - { type: "null" }
          - { x-rust-type: "BufReader<fs::File>" }
        description: "set when mode contains 'r' or '+'"
      writer:
        oneOf:
          - { type: "null" }
          - { x-rust-type: "fs::File" }
        description: "set when mode contains 'w' / 'a' / '+'"
      mode:   { type: string, description: "raw mode string from open() call" }
      closed: { type: boolean }
    required: [reader, writer, mode, closed]
  FileMode:
    type: string
    enum:
      - "r"
      - "w"
      - "a"
      - "rb"
      - "wb"
      - "ab"
      - "r+"
      - "w+"
      - "a+"
    description: "text vs binary modes parsed; binary read-as-bytes is open gap"
```

## Lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: file-lifecycle
initial: Open
nodes:
  Open:    { kind: initial, label: "mb_open allocated id; reader/writer set per mode" }
  Reading: { kind: normal,  label: "in-progress read (read / readline / readlines / iter)" }
  Writing: { kind: normal,  label: "in-progress write (write / writelines)" }
  Closed:  { kind: terminal, label: "mb_file_close set closed=true; reader/writer dropped" }
edges:
  - { from: Open,    to: Reading, event: "mb_file_read | readline | readlines | iter" }
  - { from: Open,    to: Writing, event: "mb_file_write | writelines" }
  - { from: Reading, to: Open,    event: "read returns" }
  - { from: Writing, to: Open,    event: "write flushes (no auto-close)" }
  - { from: Open,    to: Closed,  event: "mb_file_close" }
  - { from: Reading, to: Closed,  event: "mb_file_close (mid-stream)" }
  - { from: Writing, to: Closed,  event: "mb_file_close" }
---
stateDiagram-v2
    [*] --> Open
    Open --> Reading: read / readline / readlines / iter
    Open --> Writing: write / writelines
    Reading --> Open: return
    Writing --> Open: flush
    Open --> Closed: close
    Reading --> Closed: close mid
    Writing --> Closed: close
    Closed --> [*]
```

## Open / read / write logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: file-io-dispatch
entry: enter
nodes:
  enter:        { kind: start,    label: "mb_open | read | write | close (handle_or_path, ...)" }
  is_open:      { kind: decision, label: "mb_open?" }
  parse_mode:   { kind: process,  label: "parse mode 'r'/'w'/'a'/binary/+; pick reader/writer config" }
  fs_open:      { kind: process,  label: "fs::File::open or create per mode" }
  alloc_slot:   { kind: process,  label: "alloc_file_id; FILES.insert(id, MbFile)" }
  return_id:    { kind: terminal, label: "MbValue::from_int(id)" }
  is_io:        { kind: decision, label: "read / readline / readlines / write / writelines / close?" }
  lookup:       { kind: decision, label: "FILES.get(id)?" }
  read_all:     { kind: process,  label: "BufReader.read_to_string; MbObject::new_str" }
  read_line:    { kind: process,  label: "BufReader.read_line; new_str" }
  read_lines:   { kind: process,  label: "lines() collected → list of strs" }
  write_data:   { kind: process,  label: "writer.write_all" }
  write_lines:  { kind: process,  label: "for each: writer.write_all" }
  close_:       { kind: process,  label: "mark closed; drop reader/writer (Rust drop releases fd)" }
  bad_handle:   { kind: terminal, label: "TypeError / OSError" }
  done:         { kind: terminal, label: "return MbValue" }
edges:
  - { from: enter,      to: is_open }
  - { from: is_open,    to: parse_mode,  label: "yes" }
  - { from: is_open,    to: is_io,       label: "no" }
  - { from: parse_mode, to: fs_open }
  - { from: fs_open,    to: alloc_slot }
  - { from: alloc_slot, to: return_id }
  - { from: is_io,      to: lookup,      label: "yes" }
  - { from: is_io,      to: bad_handle,  label: "no" }
  - { from: lookup,     to: read_all,    label: "read" }
  - { from: lookup,     to: read_line,   label: "readline" }
  - { from: lookup,     to: read_lines,  label: "readlines" }
  - { from: lookup,     to: write_data,  label: "write" }
  - { from: lookup,     to: write_lines, label: "writelines" }
  - { from: lookup,     to: close_,      label: "close" }
  - { from: lookup,     to: bad_handle,  label: "miss" }
  - { from: read_all,   to: done }
  - { from: read_line,  to: done }
  - { from: read_lines, to: done }
  - { from: write_data, to: done }
  - { from: write_lines, to: done }
  - { from: close_,     to: done }
---
flowchart TD
    enter([file io]) --> is_open{open?}
    is_open -->|yes| parse_mode[parse mode]
    is_open -->|no| is_io{io op}
    parse_mode --> fs_open[fs::File open/create]
    fs_open --> alloc_slot[alloc id; insert]
    alloc_slot --> return_id([handle int])
    is_io --> lookup{handle in FILES?}
    lookup -->|read| read_all[BufReader read_to_string]
    lookup -->|readline| read_line[BufReader read_line]
    lookup -->|readlines| read_lines[lines collect]
    lookup -->|write| write_data[writer write_all]
    lookup -->|writelines| write_lines[per-line write]
    lookup -->|close| close_[mark closed; drop]
    lookup -->|miss| bad_handle([Type/OSError])
    read_all --> done([result])
    read_line --> done
    read_lines --> done
    write_data --> done
    write_lines --> done
    close_ --> done
```

## with-statement interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: with-open-flow
actors:
  - { id: User,    kind: actor }
  - { id: JIT,     kind: system }
  - { id: FileIO,  kind: system, label: "file_io.rs" }
  - { id: Class,   kind: system, label: "class.rs (__enter__/__exit__ on file handle proxy)" }
messages:
  - { from: User,   to: JIT,    name: "with open(path, 'r') as f: f.read()" }
  - { from: JIT,    to: FileIO, name: mb_open(path, mode) }
  - { from: FileIO, to: JIT,    name: handle_id, returns: MbValue }
  - { from: JIT,    to: Class,  name: "__enter__: returns handle as-is" }
  - { from: JIT,    to: FileIO, name: mb_file_read(handle) }
  - { from: FileIO, to: JIT,    name: contents, returns: MbValue }
  - { from: JIT,    to: Class,  name: "__exit__: regardless of exception" }
  - { from: Class,  to: FileIO, name: mb_file_close(handle) }
  - { from: FileIO, to: FileIO, name: "drop reader/writer; closed = true" }
---
sequenceDiagram
    actor User
    participant JIT
    participant FileIO
    participant Class
    User->>JIT: with open(path) as f
    JIT->>FileIO: mb_open
    FileIO-->>JIT: handle
    JIT->>Class: __enter__
    JIT->>FileIO: read
    FileIO-->>JIT: contents
    JIT->>Class: __exit__
    Class->>FileIO: mb_file_close
    FileIO->>FileIO: drop; closed
```

## Acceptance scenarios
<!-- type: scenarios lang: yaml -->
```yaml
scenarios:
  - id: file-read
    given: language/file_read.py opens a file in read mode
    when: with open(path, "r") as f reads the contents
    then: open, read, and close execute through the handle table and contents print
  - id: file-write
    given: language/file_write.py opens a file in write mode
    when: f.write writes text
    then: writer.write_all persists the data and close releases the descriptor
  - id: file-iter
    given: language/file_iter.py iterates over open(path)
    when: mb_iter receives the handle
    then: is_file_handle routes iteration to file lines
```

## Tests
<!-- type: test-plan lang: mermaid -->
```mermaid
---
id: runtime-file-io-test-plan
title: File I/O Test Plan
---
flowchart TD
    Runner["cargo test -p mamba --test conformance_tests --release -- {name} --test-threads=1"]
    Runner --> Read["language/file_read.py"]
    Runner --> Write["language/file_write.py"]
    Runner --> Iter["language/file_iter.py"]
    Runner --> Readlines["language/file_readlines.py"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: crates/mamba/src/runtime/file_io.rs
    action: modify
    impl_mode: hand-written
    description: "MbFile + thread-local FILES registry + alloc_file_id; mb_open / read / readline / readlines / write / writelines / close; is_file_handle gate. Hand-written; binary read-as-bytes is open gap."
```
