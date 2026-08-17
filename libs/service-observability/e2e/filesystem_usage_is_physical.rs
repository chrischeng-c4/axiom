// HANDWRITE-BEGIN gap="missing-generator:e2e:filesystem-usage-physical" tracker="#2947" reason="End-to-end evidence that filesystem usage reads physical volume storage and tracks writes."
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use service_observability::filesystem_usage;

struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    fn new(prefix: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let pid = std::process::id();
        let dir_name = format!("service_obs_fs_{prefix}_{pid}_{nanos}_{count}");
        let path = std::env::temp_dir().join(dir_name);
        fs::create_dir_all(&path).expect("create scratch directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn total_bytes_agrees_with_an_independent_df_reading() {
    let scratch = ScratchDir::new("df_agreement");
    let sample =
        filesystem_usage(scratch.path()).expect("filesystem_usage should succeed on scratch dir");

    let output = Command::new("df")
        .args(["-k", "-P"])
        .arg(scratch.path())
        .output()
        .expect("df -k -P should execute");
    assert!(
        output.status.success(),
        "df exited with failure: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("df output is UTF-8");
    let lines = stdout.lines().collect::<Vec<_>>();
    assert!(
        lines.len() >= 2,
        "df output should have at least header and data line: {stdout}"
    );
    let fields = lines[1].split_whitespace().collect::<Vec<_>>();
    assert!(
        fields.len() >= 2,
        "df data line should have at least 2 fields: {}",
        lines[1]
    );
    let df_total_kib: u64 = fields[1].parse().expect("df 1024-blocks field is numeric");
    let expected_total_bytes = df_total_kib * 1024;

    assert_eq!(
        sample.total_bytes, expected_total_bytes,
        "total_bytes ({}) did not match df reading ({})",
        sample.total_bytes, expected_total_bytes
    );
}

#[test]
fn available_bytes_tracks_a_physical_write() {
    let scratch = ScratchDir::new("physical_write");
    let before = filesystem_usage(scratch.path()).expect("initial sample");

    let file_path = scratch.path().join("16mib_payload.bin");
    {
        let mut file = fs::File::create(&file_path).expect("create test payload file");
        let chunk = vec![0xAB_u8; 1024 * 1024]; // 1 MiB chunk
        for _ in 0..16 {
            file.write_all(&chunk).expect("write chunk");
        }
        file.sync_all().expect("sync file to disk");
    }

    let after = filesystem_usage(scratch.path()).expect("post-write sample");

    let drop = before.available_bytes.saturating_sub(after.available_bytes);
    const EIGHT_MIB: u64 = 8 * 1024 * 1024;
    const SIXTY_FOUR_MIB: u64 = 64 * 1024 * 1024;

    assert!(
        drop >= EIGHT_MIB,
        "available_bytes drop {drop} was less than 8 MiB (before: {}, after: {})",
        before.available_bytes,
        after.available_bytes
    );
    assert!(
        drop <= SIXTY_FOUR_MIB,
        "available_bytes drop {drop} was greater than 64 MiB (before: {}, after: {})",
        before.available_bytes,
        after.available_bytes
    );
}

#[test]
fn missing_path_is_an_error_not_a_zeroed_sample() {
    let scratch = ScratchDir::new("missing_path");
    let missing = scratch.path().join("never_created_path");
    let result = filesystem_usage(&missing);
    assert!(
        result.is_err(),
        "expected Err for non-existent path, got Ok: {:?}",
        result
    );
}
// HANDWRITE-END
