// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:c75293ac" tracker="pending-tracker" reason="Client-side sharding helper."
//! Client-side sharding: pick a shard for a routing key with no L4 load
//! balancer. The client computes `crc32(key) % shards` and resolves the
//! per-shard headless DNS name itself.

/// Return the shard index for `key` given `shards` total shards.
///
/// Stable for a given (key, shards) pair. `shards` of 0 is treated as 1.
///
/// @spec projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#logic
pub fn shard_for(key: &str, shards: u32) -> u32 {
    let shards = shards.max(1);
    crc32fast::hash(key.as_bytes()) % shards
}

#[cfg(test)]
mod tests {
    use super::shard_for;

    #[test]
    fn in_range_and_stable() {
        for key in ["a", "subject.orders", "user-42", ""] {
            let s = shard_for(key, 8);
            assert!(s < 8);
            assert_eq!(s, shard_for(key, 8), "stable for the same key");
        }
    }

    #[test]
    fn zero_shards_is_one() {
        assert_eq!(shard_for("x", 0), 0);
    }
}
// HANDWRITE-END
