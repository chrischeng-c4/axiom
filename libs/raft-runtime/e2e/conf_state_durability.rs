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
const MAGIC_V3: &[u8; 8] = b"RAFTST03";

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

    // The log length begins where the encoder stopped. A writer emitting a
    // different number of bytes than the canonical encoder accounts for leaves
    // everything after the configuration at the wrong offset.
    let after = matches[0] + encoded.len();
    assert_eq!(
        &bytes[after..after + 8],
        &(state.log.len() as u64).to_le_bytes(),
        "the log length must begin immediately after the bytes the canonical encoder wrote"
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
        bytes_after.starts_with(MAGIC_V3),
        "the first save after loading an older record must write the current format, but the file still starts with {:?}",
        &bytes_after[..8]
    );
    assert_eq!(
        open(&dir).load().unwrap().unwrap(),
        loaded,
        "the upgraded file must read back as the same state"
    );
}
