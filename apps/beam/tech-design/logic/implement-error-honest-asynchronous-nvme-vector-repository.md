---
id: '2151'
summary: >
  Implement an error-honest asynchronous NVMe vector repository that correctly uses typed errors
  for I/O failures rather than silently replacing short reads with zeroed vectors.
capability_refs:
  - id: "batch-ingest-and-rebuild"
    role: primary
    claim: "beam-async-cold-vector-repository"
    coverage: full
    rationale: >
      Provides an honest NVMe storage abstraction for durable vectors that fails fast
      on bad offsets or missing files instead of masking errors.
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: implement-error-honest-asynchronous-nvme-vector-repository
entry: start
nodes:
  start: { kind: start, label: "Fetch raw vector data" }
  validate_file: { kind: decision, label: "Does file exist?" }
  return_missing: { kind: terminal, label: "Return StorageError::MissingFile" }
  validate_offset: { kind: decision, label: "Is offset valid?" }
  return_out_of_range: { kind: terminal, label: "Return StorageError::OutOfRange" }
  perform_read: { kind: process, label: "Perform read (FileExt or fallback)" }
  check_read_length: { kind: decision, label: "Was full length read?" }
  return_short_read: { kind: terminal, label: "Return StorageError::ShortRead" }
  done: { kind: terminal, label: "Return Vec<u8>" }
edges:
  - { from: start, to: validate_file }
  - { from: validate_file, to: return_missing, label: "No" }
  - { from: validate_file, to: validate_offset, label: "Yes" }
  - { from: validate_offset, to: return_out_of_range, label: "No" }
  - { from: validate_offset, to: perform_read, label: "Yes" }
  - { from: perform_read, to: check_read_length }
  - { from: check_read_length, to: return_short_read, label: "No" }
  - { from: check_read_length, to: done, label: "Yes" }
---
flowchart TD
    start([Fetch raw vector data]) --> validate_file{Does file exist?}
    validate_file -->|No| return_missing([Return StorageError::MissingFile])
    validate_file -->|Yes| validate_offset{Is offset valid?}
    validate_offset -->|No| return_out_of_range([Return StorageError::OutOfRange])
    validate_offset -->|Yes| perform_read[Perform platform-specific read]
    perform_read --> check_read_length{Read full length?}
    check_read_length -->|No| return_short_read([Return StorageError::ShortRead])
    check_read_length -->|Yes| done([Return valid bytes])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/beam/src/domain/ports.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "VectorRepository"
  - path: apps/beam/src/infrastructure/io_uring_repo.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "IoUringVectorRepository"
  - path: apps/beam/tests/io_uring_repository.rs
    action: create
    section: unit-test
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: 2151-verification
requirements:
  io_uring_or_honest_adapter:
    id: R1
    text: "Implement a Linux io_uring/O_DIRECT repository or rename and scope the adapter truthfully if the platform cannot support it."
    kind: functional
    risk: low
    verify: cargo test -p beam --test io_uring_repository
  portable_fallback:
    id: R3
    text: "Define a portable fallback without unconditional std::os::unix compilation failures."
    kind: functional
    risk: medium
    verify: cargo test -p beam --test io_uring_repository
  typed_errors:
    id: R2
    text: "Treat missing files, invalid alignment, short reads, and out-of-range offsets as typed errors."
    kind: functional
    risk: high
    verify: cargo test -p beam --test io_uring_repository
---
flowchart TD
    r1[R1 io uring or honest adapter] --> cargo_test_p_beam_test_io_uring_repository[cargo test -p beam --test io_uring_repository]
    r2[R2 typed errors] --> cargo_test_p_beam_test_io_uring_repository
    r3[R3 portable fallback] --> cargo_test_p_beam_test_io_uring_repository
```
