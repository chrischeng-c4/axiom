// HANDWRITE-BEGIN gap="missing-generator:hand-written:389a58e4" tracker="standardize-gap-projects-mamba-src-pkgmgr-resolver-pubgrub-glue-rs" reason="Adapter implementing the pubgrub::DependencyProvider trait against IndexClient. Async-to-sync bridge via tokio::runtime::Handle::block_on at this boundary only."
//! Adapter between mamba's async [`IndexClient`] and PubGrub's sync
//! `DependencyProvider` trait.
//!
//! Logic source: `.aw/tech-design/projects/mamba/pkgmgr/resolver.md#logic`
//! (`fetch_meta` + `filter_yanked` + `pick_pkg` nodes).
//!
//! ## Async-to-sync bridge
//!
//! PubGrub's solver loop is sync; the index client is async. We bridge once,
//! at this single boundary, by holding a `tokio::runtime::Handle` and calling
//! `Handle::block_on` per metadata fetch. The resolver's public entry point
//! (`Resolver::resolve`) runs PubGrub on a `spawn_blocking` worker so the
//! caller's reactor is not stalled.
//!
//! NOTE: the actual `impl pubgrub::DependencyProvider for IndexClientProvider`
//! lands together with the `pubgrub = "0.2"` dependency in the Cargo.toml
//! modify (final fire of this lifecycle). The adapter struct + helper methods
//! below are stable and exercised by `mod.rs::Resolver::resolve` regardless of
//! whether the PubGrub trait is wired yet.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::pkgmanage::pkgmgr::types::{IndexClient, IndexError, PackageMetadata};

use super::requirement::Requirement;
use super::specifier;

/// Cache key: PEP 503-normalised distribution name.
/// IndexError doesn't implement Clone, so the cache stores only successful
/// results. Errors are returned to the caller without memoisation; subsequent
/// calls re-fetch.
type Cache = Arc<Mutex<HashMap<String, PackageMetadata>>>;

/// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#logic (fetch_meta)
///
/// Wraps an [`IndexClient`] for sync consumption inside the PubGrub solver
/// loop. All I/O goes through `block_on`; results are memoised so each
/// distribution is fetched at most once per resolve.
pub struct IndexClientProvider {
    client: IndexClient,
    runtime: tokio::runtime::Handle,
    cache: Cache,
    /// Memoization for per-version `requires_dist` to avoid re-hitting the
    /// `/pypi/{name}/{version}/json` endpoint when the same node is examined
    /// from multiple paths in the resolution graph.
    requires_cache: std::sync::Mutex<std::collections::HashMap<(String, String), Vec<String>>>,
}

impl IndexClientProvider {
    pub fn new(client: IndexClient, runtime: tokio::runtime::Handle) -> Self {
        Self {
            client,
            runtime,
            cache: Cache::default(),
            requires_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Sync per-version requires_dist fetch with memoization.
    ///
    /// Hits `{index}/pypi/{name}/{version}/json` and returns the raw PEP
    /// 508 requirement strings. Caches indefinitely within one provider
    /// lifetime (one resolve). On error returns the raw IndexError —
    /// callers may choose to swallow and treat the node as leaf.
    pub fn fetch_version_requires_blocking(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Vec<String>, IndexError> {
        let key = (name.to_string(), version.to_string());
        if let Some(hit) = self.requires_cache.lock().unwrap().get(&key) {
            return Ok(hit.clone());
        }
        let fetched = self
            .runtime
            .block_on(self.client.fetch_version_requires(name, version))?;
        self.requires_cache
            .lock()
            .unwrap()
            .insert(key, fetched.clone());
        Ok(fetched)
    }

    /// Sync metadata fetch with cross-call memoisation.
    ///
    /// Returns the cached result on subsequent calls for the same name —
    /// including the cached `Err` when the index returned 404 / parse error.
    pub fn fetch_metadata_blocking(&self, name: &str) -> Result<PackageMetadata, IndexError> {
        if let Some(hit) = self.cache.lock().unwrap().get(name) {
            return Ok(hit.clone());
        }
        let fetched = self.runtime.block_on(self.client.fetch_metadata(name))?;
        self.cache
            .lock()
            .unwrap()
            .insert(name.to_string(), fetched.clone());
        Ok(fetched)
    }

    /// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#logic (filter_yanked)
    ///
    /// Return the candidate versions for `name` in newest-first order, with
    /// yanked + marker-excluded versions removed. The marker filter is a
    /// closure so the caller (resolver) can supply the active environment
    /// without leaking marker-eval policy into the adapter.
    pub fn candidate_versions<F>(
        &self,
        name: &str,
        marker_excludes: F,
    ) -> Result<Vec<String>, IndexError>
    where
        F: Fn(&str) -> bool,
    {
        let meta = self.fetch_metadata_blocking(name)?;
        let mut out: Vec<String> = meta
            .releases
            .iter()
            .filter(|(_, files)| files.iter().any(|f| !f.yanked))
            .map(|(v, _)| v.clone())
            .filter(|v| !marker_excludes(v))
            .collect();
        crate::pkgmanage::pkgmgr::pep440::sort_versions_newest_first(&mut out);
        Ok(out)
    }

    /// @spec .aw/tech-design/projects/mamba/pkgmgr/resolver.md#logic (intersect)
    ///
    /// Filter `versions` (already PEP 440 valid) to those satisfying every
    /// specifier in `req.specifiers`. Empty specifier set means "any version".
    pub fn intersect_specifiers(req: &Requirement, versions: &[String]) -> Vec<String> {
        if req.specifiers.is_empty() {
            return versions.to_vec();
        }
        versions
            .iter()
            .filter(|v| specifier::all_match(&req.specifiers, v))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::specifier::parse_set;
    use super::*;

    fn req(name: &str, specs: &str) -> Requirement {
        Requirement {
            name: name.to_string(),
            specifiers: parse_set(specs).unwrap(),
            extras: vec![],
            marker: None,
        }
    }

    #[test]
    fn intersect_keeps_only_matching() {
        let r = req("requests", ">=2.0, <3.0");
        let versions = vec![
            "1.5".to_string(),
            "2.0".to_string(),
            "2.31.0".to_string(),
            "3.0".to_string(),
        ];
        let kept = IndexClientProvider::intersect_specifiers(&r, &versions);
        assert_eq!(kept, vec!["2.0", "2.31.0"]);
    }

    #[test]
    fn intersect_empty_spec_keeps_all() {
        let r = req("anything", "");
        let versions = vec!["0.1".to_string(), "1.0".to_string()];
        let kept = IndexClientProvider::intersect_specifiers(&r, &versions);
        assert_eq!(kept, versions);
    }
}
// HANDWRITE-END
