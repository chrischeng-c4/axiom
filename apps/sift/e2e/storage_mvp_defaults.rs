use sift::storage::StorageConfig;

#[test]
fn each_signal_starts_with_one_shard_and_large_immutable_segments() {
    let config = StorageConfig::default();
    assert_eq!(config.initial_logical_shards, 1);
    assert_eq!(config.max_segment_events, 100_000);
    assert_eq!(config.max_segment_bytes, 256 * 1024 * 1024);
}
