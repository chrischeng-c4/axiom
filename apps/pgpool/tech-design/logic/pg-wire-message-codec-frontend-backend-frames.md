---
id: apps-pgpool-wire-codec
summary: PostgreSQL wire protocol 3.0 message codec for pgpool - frontend/backend message types, encode/decode over bytes::BytesMut, an incremental bounded-size async FrameReader with typed errors (no panics on malformed input), and ReadyForQuery transaction-status tracking (idle/in-transaction/failed) for the next slice's TcpHandler seam.
capability_refs:
  - id: postgres-pooler-core
    role: primary
    gap: pg-wire-frontend-protocol
    claim: pg-wire-frontend-protocol
    coverage: full
    rationale: "Defines and closes the pg-wire-frontend-protocol work root: frontend+backend message codec, bounded incremental frame reader, and transaction-status tracking, verified offline by cargo test -p pgpool --test wire_codec."
fill_sections: [logic, state-machine, schema, config, unit-test]
---

# pgpool wire codec — frontend/backend PostgreSQL protocol 3.0 frames

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-wire-codec-logic-flow
entry: poll_stream
nodes:
  poll_stream:
    kind: start
    label: "FrameReader.poll_frame reads available bytes from the async stream into BytesMut"
  has_header:
    kind: decision
    label: "Buffer holds enough bytes for a frame header (1 tag byte + 4-byte length for tagged frames; 4-byte length only for the untagged StartupMessage/SSLRequest)?"
  need_more_header:
    kind: process
    label: "Not enough header bytes yet: keep the partial bytes in BytesMut and return Pending (split/partial-read boundary)"
  read_length:
    kind: process
    label: "Parse the declared i32 frame length from the header"
  bounds_check:
    kind: decision
    label: "declared length <= configured max_frame_bytes (bounded frame size)?"
  reject_oversized:
    kind: terminal
    label: "Return FrameError::Oversized{declared,max} — typed error, connection is closed by the caller, no panic"
  buffer_complete:
    kind: decision
    label: "Buffer already holds the full declared frame length?"
  need_more_body:
    kind: process
    label: "Frame body still incomplete: keep bytes in BytesMut and return Pending (handles split/partial reads across buffer boundaries)"
  decode_message:
    kind: process
    label: "Decode the frame payload into a typed FrontendMessage or BackendMessage variant by tag byte (or, untagged, by protocol version code for StartupMessage/SSLRequest)"
  decode_ok:
    kind: decision
    label: "Decode succeeded: known tag, well-formed fixed/variable fields for that message type?"
  reject_malformed:
    kind: terminal
    label: "Return FrameError::Malformed{tag,reason} — typed error, no panic, buffer cursor still advances past the bad frame"
  update_tx_status:
    kind: process
    label: "If the decoded message is ReadyForQuery, update TransactionStatus from the status byte (I/T/E -> Idle/InTransaction/Failed)"
  emit_frame:
    kind: terminal
    label: "Advance the buffer cursor past the consumed frame and emit the typed Frame to the caller (tcp-server TcpHandler seam)"
  encode_call:
    kind: start
    label: "Caller encodes a typed FrontendMessage or BackendMessage for the write path"
  encode_write:
    kind: process
    label: "Serialize the message's tag byte (if any) + i32 length + field payload into a caller-supplied BytesMut per protocol 3.0 layout"
  encode_done:
    kind: terminal
    label: "Return the encoded byte range ready for the transport write path"
edges:
  - from: poll_stream
    to: has_header
    label: "bytes available"
  - from: has_header
    to: need_more_header
    label: "header incomplete"
  - from: need_more_header
    to: poll_stream
    label: "await more bytes"
  - from: has_header
    to: read_length
    label: "header complete"
  - from: read_length
    to: bounds_check
    label: "length parsed"
  - from: bounds_check
    to: reject_oversized
    label: "exceeds max_frame_bytes"
  - from: bounds_check
    to: buffer_complete
    label: "within bound"
  - from: buffer_complete
    to: need_more_body
    label: "body incomplete"
  - from: need_more_body
    to: poll_stream
    label: "await more bytes"
  - from: buffer_complete
    to: decode_message
    label: "full frame buffered"
  - from: decode_message
    to: decode_ok
    label: "fields parsed"
  - from: decode_ok
    to: reject_malformed
    label: "unknown tag or bad field"
  - from: decode_ok
    to: update_tx_status
    label: "well-formed"
  - from: update_tx_status
    to: emit_frame
    label: "status tracked (or message is not ReadyForQuery)"
  - from: encode_call
    to: encode_write
    label: "serialize fields"
  - from: encode_write
    to: encode_done
    label: "bytes written"
---
flowchart TD
    poll_stream([FrameReader polls stream into BytesMut]) --> has_header{Header bytes buffered?}
    has_header -->|no| need_more_header[Keep partial bytes, return Pending]
    need_more_header --> poll_stream
    has_header -->|yes| read_length[Parse declared i32 frame length]
    read_length --> bounds_check{length <= max_frame_bytes?}
    bounds_check -->|no| reject_oversized([FrameError::Oversized, no panic])
    bounds_check -->|yes| buffer_complete{Full frame buffered?}
    buffer_complete -->|no| need_more_body[Keep bytes, return Pending]
    need_more_body --> poll_stream
    buffer_complete -->|yes| decode_message[Decode payload by tag/version code]
    decode_message --> decode_ok{Fields well-formed?}
    decode_ok -->|no| reject_malformed([FrameError::Malformed, no panic])
    decode_ok -->|yes| update_tx_status[Update TransactionStatus if ReadyForQuery]
    update_tx_status --> emit_frame([Advance cursor, emit typed Frame])
    encode_call([Caller encodes a message]) --> encode_write[Serialize tag+length+fields into BytesMut]
    encode_write --> encode_done([Return encoded bytes for the write path])
```
## Transaction Status Tracking
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: pgpool-wire-codec-tx-status-fsm
initial: idle
nodes:
  idle:
    kind: initial
    label: "Idle: ReadyForQuery status byte 'I' — no transaction block open"
  in_transaction:
    kind: normal
    label: "InTransaction: ReadyForQuery status byte 'T' — an explicit or implicit transaction block is open"
  failed:
    kind: normal
    label: "Failed: ReadyForQuery status byte 'E' — a statement inside the open transaction block errored; only ROLLBACK/COMMIT-abort is accepted until the next ReadyForQuery"
edges:
  - from: idle
    to: in_transaction
    event: "ReadyForQuery('T') observed (BEGIN, or first statement of an implicit multi-statement block)"
  - from: in_transaction
    to: idle
    event: "ReadyForQuery('I') observed (COMMIT or ROLLBACK completed cleanly)"
  - from: in_transaction
    to: failed
    event: "ReadyForQuery('E') observed (a statement inside the transaction block errored)"
  - from: failed
    to: idle
    event: "ReadyForQuery('I') observed (ROLLBACK completed after failure)"
  - from: failed
    to: failed
    event: "ReadyForQuery('E') observed again (a further statement is rejected while still failed)"
---
stateDiagram-v2
    [*] --> idle
    idle --> in_transaction : ReadyForQuery('T')
    in_transaction --> idle : ReadyForQuery('I')
    in_transaction --> failed : ReadyForQuery('E')
    failed --> idle : ReadyForQuery('I')
    failed --> failed : ReadyForQuery('E')
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
(fill)
```

## Config
<!-- type: config lang: yaml -->

```yaml
(fill)
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-wire-codec-unit-test-pending
entry: pending
nodes:
  pending:
    kind: start
    label: "unit-test plan pending — to be authored in its own applicability section"
edges: []
---
flowchart TD
    pending([unit-test plan pending])
```
