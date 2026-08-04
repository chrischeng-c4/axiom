# Storage Durable Capabilities

## Brief

`storage-durable` owns the local-disk mechanics that every axiom service with
durable state would otherwise reimplement: replacing a file without ever
publishing a half-written one, appending to a log that survives being cut off
mid-write, and keeping a directory of sequence-named snapshots in order.

It does not own domain codecs. A caller decides what its records and snapshots
contain; this crate decides when those bytes are safe to read back. Its defining
constraint is that it is the layer that runs while the machine is losing power:
every promise here is about what a reader finds after an interrupted write, not
about what a writer intended.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** define what `storage-durable` fundamentally does: publish a
  new file version atomically, and append to a log whose damaged tail is
  recoverable.
- **Non-Core Features** keep the stored set navigable — the snapshot directory
  stays ordered by sequence, and retention removes exactly what it says.
  Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Crash-Safe Replacement | - | implemented | verified | smoke | ready | core; a reader sees either the whole previous version or the whole new one, never a prefix of either |
| Torn-Tail Recovery | - | implemented | verified | smoke | ready | core; a frame is admitted only when its header, its payload, and its checksum all agree, and the first frame that fails ends the log |
| Sequence-Ordered Snapshots | - | implemented | verified | smoke | ready | non-core; order comes from the parsed sequence number, never from filename collation or modification time |

### Core Features

#### Crash-Safe Replacement

ID: crash-safe-replacement
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
Replacing a durable file publishes the new contents in one step. A reader
opening the path at any moment gets either the complete previous version or the
complete new one — never a truncated file, never an empty one, never a mixture.
The caller chooses through a flush policy how much is forced to stable storage
before the replacement is called done; the ordering of the steps does not change
with that choice.
Surfaces:
- Rust API: `storage_durable::atomic_write` - write bytes, flush per policy, then rename into place.
- Rust API: `storage_durable::sync_parent_dir` - force the directory entry so the rename itself survives.
- Rust API: `storage_durable::FsyncPolicy` - the four flush policies and which of them syncs at the durability boundary.
Rust internal: the staging path derived from the target path, and the classes of directory-open failure that are tolerated rather than propagated.
EC Dimensions:
- behavior: `cargo test -p storage-durable --lib` - the step order is write, flush, rename, sync-parent; the published path is exactly the requested path; a repeated write replaces rather than appends.
- security: `cargo test -p storage-durable --lib` - a failure before the rename leaves the previous version intact and publishes nothing, and a stale staging file from an earlier crash cannot be published as if it were the new version.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Atomic publication | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; the bytes reach a staging path first and become visible at the target path only through a rename, so no observation of the target path ever returns a partial write |
| Policy-driven flushing | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; every policy except the OS-cache policy forces both the file and its parent directory, and the policy changes which steps force stable storage without changing their order |
| Stale staging removal | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; a staging file left behind by an earlier interrupted write is discarded before the new write begins, so its contents can never be renamed into place |

#### Torn-Tail Recovery

ID: torn-tail-recovery
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
An append log that was cut off mid-frame reads back as the frames that
completed, and nothing else. A frame is admitted only when its header is whole,
its declared payload is entirely present in the file, and its checksum matches
the bytes actually stored. The first frame that fails any of those three tests
ends the log for every reader, and reopening the log for writing truncates the
file to that point so the damaged tail cannot be interleaved with new appends.
Surfaces:
- Rust API: `storage_durable::FramedLogWriter` - open, append, flush, sync, and compaction through a rewritten temp file.
- Rust API: `storage_durable::FramedLogReader` - replay from a sequence, read validated frames, and find the last good offset.
- Rust API: `storage_durable::LogFrame` - the sequence and payload a reader gets back.
Rust internal: the fixed-width frame header carrying sequence, payload length, and checksum, and the scan that decides the good end.
EC Dimensions:
- behavior: `cargo test -p storage-durable --lib` - a whole frame round-trips its sequence and payload, replay yields frames after a requested sequence in file order, and compaction keeps exactly the frames past the retention point.
- security: `cargo test -p storage-durable --lib` - a truncated header, a payload length larger than the file holds, and a payload whose checksum does not match are all refused rather than returned, and recovery only ever shortens the file.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Three-part frame admission | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; a frame is returned only when the header fits, the declared payload length fits within the remaining file, and the checksum recomputed over the stored payload equals the stored checksum — failing any one is refusal, not repair |
| Prefix-stable replay | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; the first failing frame ends the log for every reader, so appending garbage to a good log changes nothing about which frames replay, and every reader agrees on the same prefix |
| Truncating reopen | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; opening a log for append first shortens it to the last good end, so a subsequent append can never be preceded by a partial frame, and the reopen never lengthens the file |

### Non-Core Features

#### Sequence-Ordered Snapshots

ID: sequence-ordered-snapshots
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A snapshot directory is a set of files named from a caller-chosen prefix, a
sequence number, and a caller-chosen extension. Every ordering question — which
snapshot is latest, which ones retention drops — is answered by the parsed
sequence number, never by how the names sort as text and never by modification
time. A file in the directory that does not parse as a snapshot of this store is
not a snapshot of this store.
Surfaces:
- Rust API: `storage_durable::SnapshotFileStore` - save, load the latest, list, and prune to a retention count.
- Rust API: `storage_durable::SnapshotFile` - the sequence and path of one listed snapshot.
Rust internal: the name format that joins prefix, sequence, and extension, and the parse that recovers the sequence from a path.
EC Dimensions:
- behavior: `cargo test -p storage-durable --lib` - the latest snapshot is the highest sequence regardless of write order, listing is sorted ascending by sequence, and pruning keeps the highest sequences and reports how many it removed.
- security: `cargo test -p storage-durable --lib` - a file whose name does not parse under this store's prefix and extension is ignored rather than loaded, and pruning never removes more than the listed count minus the retention count.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Sequence is the order | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; snapshots written out of order still list ascending and still resolve the highest sequence as latest, so a store that would sort `snap-10` before `snap-9` as text does not do so here |
| Foreign name rejection | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; a file with the right extension but another prefix, or the right prefix but an unparsable sequence, is absent from the listing and can never be returned as the latest snapshot |
| Bounded retention | change | - | implemented | verified | smoke | `cargo test -p storage-durable --lib`; pruning to a retention count keeps exactly the highest-sequence snapshots, removes exactly the listed count minus the retention count, is a no-op when the store already holds no more than that, and reports the number it actually removed |
