use std::{fs::OpenOptions, io::Write};

use anyhow::bail;
use storage_durable::{
    FramedLogCursor, FramedLogReader, FramedLogWriter, FsyncPolicy, MAX_FRAME_PAYLOAD_BYTES,
};

#[test]
fn visitor_streams_only_the_requested_suffix_and_propagates_failure() {
    let data = tempfile::tempdir().unwrap();
    let path = data.path().join("receipts.framed");
    let mut writer = FramedLogWriter::open(&path, FsyncPolicy::Os).unwrap();
    for sequence in 1..=1_000_u64 {
        writer
            .append(
                sequence,
                &vec![u8::try_from(sequence % 251).unwrap(); 4_096],
            )
            .unwrap();
    }
    writer.flush().unwrap();

    let mut visited = 0_u64;
    let maximum = FramedLogReader::visit_frames(&path, 900, |frame| {
        visited += 1;
        assert_eq!(frame.seq, 900 + visited);
        assert_eq!(frame.payload.len(), 4_096);
        Ok(())
    })
    .unwrap();
    assert_eq!(visited, 100);
    assert_eq!(maximum, 1_000);

    let mut stopped_at = 0_u64;
    let error = FramedLogReader::visit_frames(&path, 0, |frame| {
        stopped_at = frame.seq;
        if frame.seq == 10 {
            bail!("injected visitor failure");
        }
        Ok(())
    })
    .unwrap_err();
    assert_eq!(stopped_at, 10);
    assert!(error.to_string().contains("injected visitor failure"));
}

#[test]
fn cursor_keeps_its_byte_offset_and_holds_only_one_frame() {
    let data = tempfile::tempdir().unwrap();
    let path = data.path().join("large-frames.framed");
    let mut writer = FramedLogWriter::open(&path, FsyncPolicy::Os).unwrap();
    for sequence in 1..=4_u64 {
        writer
            .append(sequence, &vec![u8::try_from(sequence).unwrap(); 1_048_576])
            .unwrap();
    }
    writer.flush().unwrap();

    let mut cursor = FramedLogCursor::open(&path).unwrap();
    let mut previous_offset = 0;
    for sequence in 1..=4_u64 {
        let frame = cursor.next_frame().unwrap().unwrap();
        assert_eq!(frame.seq, sequence);
        assert_eq!(frame.payload.len(), 1_048_576);
        assert!(cursor.byte_offset() > previous_offset);
        previous_offset = cursor.byte_offset();
        drop(frame);
    }
    assert!(cursor.next_frame().unwrap().is_none());
}

#[test]
fn oversized_sparse_frame_is_rejected_before_payload_allocation() {
    let data = tempfile::tempdir().unwrap();
    let path = data.path().join("oversized-frame.framed");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap();
    let declared_len = u32::MAX;
    file.write_all(&1_u64.to_le_bytes()).unwrap();
    file.write_all(&declared_len.to_le_bytes()).unwrap();
    file.write_all(&1_u32.to_le_bytes()).unwrap();
    file.set_len(16 + u64::from(declared_len)).unwrap();
    drop(file);

    assert!(FramedLogCursor::open(&path)
        .unwrap()
        .next_frame()
        .unwrap_err()
        .to_string()
        .contains("legacy log frame"));
    assert!(FramedLogReader::read_frames(&path, 0)
        .unwrap_err()
        .to_string()
        .contains("legacy log frame"));

    let original_len = std::fs::metadata(&path).unwrap().len();
    assert!(FramedLogWriter::open(&path, FsyncPolicy::Os).is_err());
    assert_eq!(std::fs::metadata(&path).unwrap().len(), original_len);
    assert!(MAX_FRAME_PAYLOAD_BYTES < declared_len as usize);
}

#[test]
fn valid_legacy_frame_above_the_new_limit_is_preserved_on_open() {
    let data = tempfile::tempdir().unwrap();
    let path = data.path().join("legacy-large-frame.framed");
    let payload = vec![0x5a; MAX_FRAME_PAYLOAD_BYTES + 1];
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap();
    file.write_all(&7_u64.to_le_bytes()).unwrap();
    file.write_all(&(payload.len() as u32).to_le_bytes())
        .unwrap();
    file.write_all(&crc32fast::hash(&payload).to_le_bytes())
        .unwrap();
    file.write_all(&payload).unwrap();
    file.sync_all().unwrap();
    drop(file);
    let original_len = std::fs::metadata(&path).unwrap().len();

    let _writer = FramedLogWriter::open(&path, FsyncPolicy::Os).unwrap();
    assert_eq!(std::fs::metadata(&path).unwrap().len(), original_len);

    let error = FramedLogReader::read_frames(&path, 0).unwrap_err();
    assert!(error.to_string().contains("validated legacy log frame"));
    assert_eq!(std::fs::metadata(&path).unwrap().len(), original_len);
}
