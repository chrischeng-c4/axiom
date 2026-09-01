//! The durable record carries the group's configuration and its entry kinds,
//! and a record written by an earlier format still loads (#3568).
//!
//! The state file is hand-rolled little-endian with explicit per-field reads,
//! so a new field is a versioned format change rather than something a
//! self-describing codec absorbs. These rows craft records at the older
//! versions byte by byte, because after this change `RaftStore::save` can only
//! write the current one.

use tempfile::TempDir;

use raft_core::{ConfState, EntryKind, Membership, PersistedState, RaftEntry};
use raft_runtime::{FsyncPolicy, RaftStore};

const MAGIC_V1: &[u8; 8] = b"RAFTST01";
const MAGIC_V2: &[u8; 8] = b"RAFTST02";
const MAGIC_V4: &[u8; 8] = b"RAFTST04";
const LOG_MAGIC_V1: &[u8; 8] = b"RAFTLG01";

/// One log entry as every format before this change wrote it: term, index,
/// length-prefixed command, and nothing else.
fn push_unkinded_entry(buf: &mut Vec<u8>, term: u64, index: u64, command: &[u8]) {
    buf.extend_from_slice(&term.to_le_bytes());
    buf.extend_from_slice(&index.to_le_bytes());
    buf.extend_from_slice(&(command.len() as u64).to_le_bytes());
    buf.extend_from_slice(command);
}

/// A `RAFTST02` record with an empty snapshot. The reader consults the snapshot
/// digest only when `snapshot_len` is non-zero, so the 32 digest bytes are
/// unread here.
fn v2_record(term: u64, voted_for: Option<u64>, commands: &[&[u8]]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(MAGIC_V2);
    buf.extend_from_slice(&term.to_le_bytes());
    match voted_for {
        Some(id) => {
            buf.push(1);
            buf.extend_from_slice(&id.to_le_bytes());
        }
        None => buf.push(0),
    }
    buf.extend_from_slice(&(commands.len() as u64).to_le_bytes()); // commit_index
    buf.extend_from_slice(&0u64.to_le_bytes()); // snapshot_index
    buf.extend_from_slice(&0u64.to_le_bytes()); // snapshot_term
    buf.extend_from_slice(&0u64.to_le_bytes()); // snapshot_len
    buf.extend_from_slice(&[0u8; 32]); // snapshot digest
    buf.extend_from_slice(&(commands.len() as u64).to_le_bytes());
    for (i, command) in commands.iter().enumerate() {
        push_unkinded_entry(&mut buf, term, i as u64 + 1, command);
    }
    buf
}

/// A `RAFTST01` record: the snapshot bytes are inline and there is no digest.
fn v1_record(term: u64, voted_for: Option<u64>, commands: &[&[u8]]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(MAGIC_V1);
    buf.extend_from_slice(&term.to_le_bytes());
    match voted_for {
        Some(id) => {
            buf.push(1);
            buf.extend_from_slice(&id.to_le_bytes());
        }
        None => buf.push(0),
    }
    buf.extend_from_slice(&(commands.len() as u64).to_le_bytes()); // commit_index
    buf.extend_from_slice(&0u64.to_le_bytes()); // snapshot_index
    buf.extend_from_slice(&0u64.to_le_bytes()); // snapshot_term
    buf.extend_from_slice(&0u64.to_le_bytes()); // snapshot_len
    buf.extend_from_slice(&(commands.len() as u64).to_le_bytes());
    for (i, command) in commands.iter().enumerate() {
        push_unkinded_entry(&mut buf, term, i as u64 + 1, command);
    }
    buf
}

fn open(dir: &TempDir) -> RaftStore {
    RaftStore::open(dir.path().to_str().unwrap(), 7, FsyncPolicy::Os).unwrap()
}

#[test]
fn a_record_written_by_the_current_format_loads_with_no_configuration() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    std::fs::write(store.path(), v2_record(3, Some(2), &[b"one", b"two"])).unwrap();

    let state = store.load().unwrap().expect("a RAFTST02 record must load");

    assert_eq!(
        state.conf, None,
        "a record written before membership was durable must load with no configuration rather than be rejected"
    );
    assert_eq!(state.term, 3);
    assert_eq!(state.voted_for, Some(2));
    assert_eq!(state.commit_index, 2);
    assert_eq!(
        state.log,
        vec![
            RaftEntry {
                term: 3,
                index: 1,
                command: b"one".to_vec(),
                kind: EntryKind::Command,
            },
            RaftEntry {
                term: 3,
                index: 2,
                command: b"two".to_vec(),
                kind: EntryKind::Command,
            },
        ],
        "every entry in a record written before this change is a client command"
    );
}

#[test]
fn the_oldest_record_format_still_loads() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    std::fs::write(store.path(), v1_record(1, None, &[b"only"])).unwrap();

    let state = store.load().unwrap().expect("a RAFTST01 record must load");

    assert_eq!(state.conf, None);
    assert_eq!(state.term, 1);
    assert_eq!(state.voted_for, None);
    assert_eq!(state.log[0].kind, EntryKind::Command);
    assert_eq!(state.log[0].command, b"only".to_vec());
}

#[test]
fn the_configuration_and_entry_kinds_survive_the_durable_round_trip() {
    let dir = TempDir::new().unwrap();
    let conf = ConfState {
        membership: Membership {
            voters: vec![0, 1, 2],
            learners: vec![3],
        },
        outgoing: None,
        generation: 9,
    };
    let state = PersistedState {
        term: 4,
        voted_for: Some(0),
        log: vec![
            RaftEntry {
                term: 4,
                index: 1,
                command: b"cmd".to_vec(),
                kind: EntryKind::Command,
            },
            RaftEntry {
                term: 4,
                index: 2,
                command: conf.encode(),
                kind: EntryKind::Config,
            },
        ],
        commit_index: 2,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: Some(conf.clone()),
    };

    {
        let store = open(&dir);
        store.save(&state).unwrap();
    }
    let reloaded = open(&dir).load().unwrap().expect("saved state must load");

    assert_eq!(
        reloaded, state,
        "the durable record must carry the configuration and both entry kinds unchanged"
    );
    assert_eq!(reloaded.conf, Some(conf));
    assert_eq!(reloaded.log[1].kind, EntryKind::Config);
}

#[test]
fn the_record_carries_the_configuration_as_the_canonical_encoder_writes_it() {
    let dir = TempDir::new().unwrap();
    let conf = ConfState {
        membership: Membership {
            voters: vec![4, 5, 6],
            learners: vec![7, 8],
        },
        outgoing: None,
        generation: 11,
    };
    let state = PersistedState {
        term: 2,
        voted_for: None,
        log: vec![RaftEntry {
            term: 2,
            index: 1,
            command: b"payload".to_vec(),
            kind: EntryKind::Command,
        }],
        commit_index: 1,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: Some(conf.clone()),
    };

    let store = open(&dir);
    store.save(&state).unwrap();
    let bytes = std::fs::read(store.path()).unwrap();

    // The expectation is the canonical encoder's output, never a byte string
    // spelled out here: a literal is satisfied by whichever implementation
    // wrote the file and would keep passing while the two drift apart.
    let encoded = conf.encode();
    let matches: Vec<usize> = bytes
        .windows(encoded.len())
        .enumerate()
        .filter(|(_, window)| *window == encoded.as_slice())
        .map(|(at, _)| at)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "the record must carry the configuration exactly as ConfState::encode writes it, but those {} bytes occur {} times in the {} the store wrote",
        encoded.len(),
        matches.len(),
        bytes.len()
    );

    // The V4 log-artifact tag begins where the encoder stopped. A writer
    // emitting a different number of bytes than the canonical encoder accounts
    // for leaves everything after the configuration at the wrong offset.
    let after = matches[0] + encoded.len();
    assert_eq!(
        bytes[after], 1,
        "the log-artifact tag must begin immediately after the bytes the canonical encoder wrote"
    );

    let log_artifacts = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("raft-7-log-") && name.ends_with(".artifact"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        log_artifacts.len(),
        1,
        "the current record must reference exactly one immutable log artifact"
    );
    assert!(
        std::fs::read(&log_artifacts[0])
            .unwrap()
            .starts_with(LOG_MAGIC_V1),
        "the split log must use the checksummed log-artifact format"
    );

    assert_eq!(
        open(&dir).load().unwrap().unwrap(),
        state,
        "the record the canonical encoder's bytes were written into must read back unchanged"
    );
}

#[test]
fn saving_after_loading_an_older_record_upgrades_the_file_in_place() {
    let dir = TempDir::new().unwrap();
    let store = open(&dir);
    std::fs::write(store.path(), v2_record(3, Some(2), &[b"one"])).unwrap();

    let loaded = store.load().unwrap().unwrap();
    let bytes_before = std::fs::read(store.path()).unwrap();
    assert!(
        bytes_before.starts_with(MAGIC_V2),
        "loading must not rewrite the file on its own"
    );

    store.save(&loaded).unwrap();

    let bytes_after = std::fs::read(store.path()).unwrap();
    assert!(
        bytes_after.starts_with(MAGIC_V4),
        "the first save after loading an older record must write the current format, but the file still starts with {:?}",
        &bytes_after[..8]
    );
    assert_eq!(
        open(&dir).load().unwrap().unwrap(),
        loaded,
        "the upgraded file must read back as the same state"
    );
}

/// The record is bytes on disk, so both of the configuration's length fields
/// are reachable by corruption and nothing upstream of the decoder bounds them
/// (#3580). A claimed length the record cannot honour must be refused the way
/// every other malformed field is.
#[test]
fn a_configuration_claiming_a_length_the_record_cannot_hold_is_refused() {
    let conf = ConfState {
        membership: Membership {
            voters: vec![4, 5, 6],
            learners: vec![7, 8],
        },
        outgoing: None,
        generation: 11,
    };

    // Offsets inside the canonical encoder's output: generation, the voters
    // length, the voters, the learners length, the learners.
    const VOTERS_LEN_AT: usize = 8;
    let learners_len_at = VOTERS_LEN_AT + 8 + conf.membership.voters.len() * 8;

    // `claimed * 8` is the byte cost the decoder computes before it has decided
    // the claim is honourable. `1 << 61` makes that product wrap, which is the
    // arithmetic this row exists for. `1_000` does not wrap and is already
    // refused, and is here so that a fix which caps or truncates an over-long
    // claim rather than refusing it stays red.
    let cases: [(&str, usize, u64); 3] = [
        (
            "a voters length whose byte cost wraps",
            VOTERS_LEN_AT,
            1u64 << 61,
        ),
        (
            "a learners length whose byte cost wraps",
            learners_len_at,
            1u64 << 61,
        ),
        (
            "a voters length larger than the record",
            VOTERS_LEN_AT,
            1_000,
        ),
    ];

    for (what, field_at, claimed) in cases {
        let dir = TempDir::new().unwrap();
        let state = PersistedState {
            term: 2,
            voted_for: None,
            log: vec![RaftEntry {
                term: 2,
                index: 1,
                command: b"payload".to_vec(),
                kind: EntryKind::Command,
            }],
            commit_index: 1,
            snapshot_index: 0,
            snapshot_term: 0,
            snapshot: vec![],
            conf: Some(conf.clone()),
        };
        let store = open(&dir);
        store.save(&state).unwrap();

        // Locate the configuration by the canonical encoder's bytes rather than
        // by a fixed offset into the record, so this row keeps pointing at the
        // field it names if anything ahead of the configuration changes width.
        let mut bytes = std::fs::read(store.path()).unwrap();
        let encoded = conf.encode();
        let at = bytes
            .windows(encoded.len())
            .position(|window| window == encoded.as_slice())
            .unwrap_or_else(|| panic!("the record must carry the configuration, for {what}"));
        bytes[at + field_at..at + field_at + 8].copy_from_slice(&claimed.to_le_bytes());
        std::fs::write(store.path(), bytes).unwrap();

        match open(&dir).load() {
            Err(e) => assert_eq!(
                e.kind(),
                std::io::ErrorKind::InvalidData,
                "{what}: a corrupt configuration length must be refused as invalid data, not as {:?}",
                e.kind()
            ),
            Ok(loaded) => panic!(
                "{what}: the record claims {claimed} members it does not carry, and load() returned {loaded:?} — a claim that is capped or truncated instead of refused hands the caller a configuration the record never held"
            ),
        }
    }
}

/// The voters bound in `ConfState::decode_with_len` reserves eight bytes for
/// the trailing `learners_len` field (#3589). A claim of exactly one slot past
/// that bound must be refused by `RaftStore::load` as `InvalidData` rather than
/// proceeding to read past the end of the buffer and panicking.
#[test]
fn a_voters_length_claiming_one_slot_past_the_bound_is_refused() {
    let conf = ConfState {
        membership: Membership {
            voters: vec![4, 5, 6],
            learners: vec![7, 8],
        },
        outgoing: None,
        generation: 11,
    };

    const VOTERS_LEN_AT: usize = 8;

    let dir = TempDir::new().unwrap();
    let state = PersistedState {
        term: 2,
        voted_for: None,
        log: vec![RaftEntry {
            term: 2,
            index: 1,
            command: b"payload".to_vec(),
            kind: EntryKind::Command,
        }],
        commit_index: 1,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: Some(conf.clone()),
    };
    let store = open(&dir);
    store.save(&state).unwrap();

    let mut bytes = std::fs::read(store.path()).unwrap();
    let encoded = conf.encode();
    let at = bytes
        .windows(encoded.len())
        .position(|window| window == encoded.as_slice())
        .expect("the record must carry the configuration");

    let body_len = bytes.len() - at;
    // Compute the claim from the record that was just written: exactly one slot
    // past what the bound permits ((body_len - 16 - 8) / 8).
    let bound = (body_len - 16 - 8) / 8;
    let claimed = (bound + 1) as u64;

    bytes[at + VOTERS_LEN_AT..at + VOTERS_LEN_AT + 8].copy_from_slice(&claimed.to_le_bytes());
    std::fs::write(store.path(), bytes).unwrap();

    match open(&dir).load() {
        Err(e) => assert_eq!(
            e.kind(),
            std::io::ErrorKind::InvalidData,
            "a voters length one slot past the bound must be refused as invalid data, not as {:?}",
            e.kind()
        ),
        Ok(loaded) => panic!(
            "the record claims {claimed} voters (bound is {bound}), and load() returned {loaded:?} — an over-bound claim must be refused"
        ),
    }
}

/// A `Config` entry whose command does not decode as a `ConfState` must be
/// refused by `load()`, the way the record's own configuration field already is
/// (#3582). Until it is, the entry survives the load and `take_committed`
/// silently drops it — on every restart, because `last_applied` is not
/// persisted — so the node runs on a membership its own log does not agree
/// with, and nothing is reported to the host.
#[test]
fn a_config_entry_whose_command_does_not_decode_is_refused() {
    let dir = TempDir::new().unwrap();
    let conf = ConfState {
        membership: Membership {
            voters: vec![0, 1, 2],
            learners: vec![3],
        },
        outgoing: None,
        generation: 9,
    };

    // Shorter than the decoder's own floor, so it cannot decode. Asserted here
    // rather than assumed: if that floor ever moves, this row says so instead
    // of quietly becoming a test of a well-formed entry.
    let undecodable = b"not a conf".to_vec();
    assert!(
        ConfState::decode(&undecodable).is_none(),
        "this row needs a command that does not decode; {undecodable:?} does"
    );

    let state = PersistedState {
        term: 4,
        voted_for: Some(0),
        log: vec![
            RaftEntry {
                term: 4,
                index: 1,
                command: b"cmd".to_vec(),
                kind: EntryKind::Command,
            },
            RaftEntry {
                term: 4,
                index: 2,
                command: undecodable,
                kind: EntryKind::Config,
            },
        ],
        commit_index: 2,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: Some(conf),
    };

    open(&dir).save(&state).unwrap();

    match open(&dir).load() {
        Err(e) => assert_eq!(
            e.kind(),
            std::io::ErrorKind::InvalidData,
            "a Config entry whose command does not decode must be refused as invalid data, not as {:?}",
            e.kind()
        ),
        Ok(loaded) => panic!(
            "load() accepted a record whose Config entry command cannot decode, returning {loaded:?} — the entry is dropped on every restart and the log disagrees with the membership in force"
        ),
    }
}

/// The companion to the row above, and the reason it is not enough on its own:
/// there, the command is ten bytes, so a store that refused every short command
/// without ever consulting the decoder would pass it. Here the command is past
/// the decoder's length floor and still does not decode, so length cannot be the
/// reason it is refused. Together the two rows say the criterion is the
/// decoder's verdict.
#[test]
fn a_config_entry_longer_than_the_decoders_floor_is_still_judged_by_the_decoder() {
    let dir = TempDir::new().unwrap();
    let conf = ConfState {
        membership: Membership {
            voters: vec![0, 1, 2],
            learners: vec![3],
        },
        outgoing: None,
        generation: 9,
    };

    // The decoder's own layout: generation, then a voters length claiming far
    // more slots than the remaining bytes can hold. Forty bytes, so the 24-byte
    // floor is cleared and the refusal has to come from the bound check.
    let mut undecodable = Vec::new();
    undecodable.extend_from_slice(&0u64.to_le_bytes());
    undecodable.extend_from_slice(&1000u64.to_le_bytes());
    undecodable.extend_from_slice(&[0u8; 24]);
    assert!(
        undecodable.len() >= 24,
        "this row is worthless unless the command clears the decoder's floor; it is {} bytes",
        undecodable.len()
    );
    assert!(
        ConfState::decode(&undecodable).is_none(),
        "this row needs a long command that does not decode; {undecodable:?} does"
    );

    let state = PersistedState {
        term: 4,
        voted_for: Some(0),
        log: vec![RaftEntry {
            term: 4,
            index: 1,
            command: undecodable,
            kind: EntryKind::Config,
        }],
        commit_index: 1,
        snapshot_index: 0,
        snapshot_term: 0,
        snapshot: vec![],
        conf: Some(conf),
    };

    open(&dir).save(&state).unwrap();

    match open(&dir).load() {
        Err(e) => assert_eq!(
            e.kind(),
            std::io::ErrorKind::InvalidData,
            "a Config entry whose command does not decode must be refused as invalid data, not as {:?}",
            e.kind()
        ),
        Ok(loaded) => panic!(
            "load() accepted a record whose Config entry command is long but undecodable, returning {loaded:?} — the refusal is keyed on the command's length, not on whether it decodes"
        ),
    }
}
