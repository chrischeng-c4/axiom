//! Shard routing.
//!
//! Two layers, split between client and server:
//!   Layer 1 (client) — shard math: `crc32(collection_id) % shard_count`.
//!   Layer 2 (server) — Raft leader forwarding inside the shard.
//!
//! Clients only need Layer 1; the server handles re-election transparently.

pub fn shard_index(collection_id: &str, shard_count: u32) -> u32 {
    debug_assert!(shard_count > 0, "shard_count must be > 0");
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(collection_id.as_bytes());
    hasher.finalize() % shard_count
}

/// DNS for a given shard's stable client entry (any replica will do —
/// the server forwards writes internally).
pub fn shard_host(prefix: &str, shard: u32, headless_service: &str) -> String {
    format!("{prefix}-{shard}.{headless_service}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_index_is_deterministic() {
        let a = shard_index("data-table:42", 3);
        let b = shard_index("data-table:42", 3);
        assert_eq!(a, b);
        assert!(a < 3);
    }

    #[test]
    fn shard_index_spreads() {
        let mut seen = std::collections::HashSet::new();
        for i in 0..256 {
            seen.insert(shard_index(&format!("c:{i}"), 3));
        }
        assert!(seen.len() > 1, "shard hash collapsed to a single bucket");
    }

    #[test]
    fn shard_index_single_shard_always_zero() {
        for s in ["a", "very-long-string", "中文"] {
            assert_eq!(shard_index(s, 1), 0);
        }
    }

    #[test]
    fn shard_host_formats_dns() {
        let h = shard_host("lumen", 2, "lumen-peer");
        assert_eq!(h, "lumen-2.lumen-peer");
    }
}
