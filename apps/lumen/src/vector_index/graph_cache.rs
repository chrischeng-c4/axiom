//! Optional HNSW graph acceleration. Checkpoint vectors remain authoritative.
use super::*;
use anyhow::{ensure, Context};
use hnsw_rs::anndists::dist::{DistDot, DistL2};
use hnsw_rs::{api::AnnT, hnsw::Hnsw, hnswio::HnswIo};
use self_cell::{self_cell, MutBorrow};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

const BASENAME: &str = "graph";
const MANIFEST: &str = "manifest.json";
const FORMAT: &str = "lumen-hnsw-cache-v1/hnsw_rs-0.3.4-dump-v3";

/// Bounded per-field timings for an optional graph-cache restore attempt.
///
/// These are diagnostic-only. They deliberately retain every cache-integrity
/// check so a large cold start can identify its active phase without changing
/// recovery authority.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct LoadTiming {
    pub(super) prepare: Duration,
    pub(super) fingerprint: Duration,
    pub(super) manifest: Duration,
    pub(super) payload_hash: Duration,
    pub(super) materialize: Duration,
    pub(super) deserialize: Duration,
    pub(super) validate: Duration,
}

#[derive(Debug)]
pub(super) struct LoadFailure {
    pub(super) error: anyhow::Error,
    pub(super) timing: LoadTiming,
}

enum LoadedGraph<'a> {
    L2(Hnsw<'a, f32, DistL2>),
    Cosine(Hnsw<'a, f32, DistDot>),
    Dot(Hnsw<'a, f32, DistDot>),
}

pub(super) fn load(
    spec: VectorSpec,
    vectors: &[(String, Vec<f32>)],
    codebook: Option<ScalarCodebook>,
    directory: &Path,
) -> Result<Option<HnswCpuIndex>> {
    load_timed(spec, vectors, codebook, directory)
        .map_err(|failure| failure.error)
        .map(|(index, _)| index)
}

// The loader owns every borrowed mapping for exactly as long as the graph.
// No leaked loader, forged 'static lifetime or new unsafe implementation.
self_cell!(
    pub(super) struct OwnedGraph {
        owner: MutBorrow<HnswIo>,
        #[not_covariant]
        dependent: LoadedGraph,
    }
);

impl OwnedGraph {
    pub(super) fn insert(&self, vector: &[f32], id: usize) {
        self.with_dependent(|_, graph| match graph {
            LoadedGraph::L2(h) => h.insert((vector, id)),
            LoadedGraph::Dot(h) => h.insert((vector, id)),
            LoadedGraph::Cosine(h) => h.insert((&normalize_unit_safe(vector), id)),
        });
    }

    pub(super) fn search(&self, vector: &[f32], k: usize, ef: usize) -> Vec<(usize, f32)> {
        self.with_dependent(|_, graph| {
            let hits = match graph {
                LoadedGraph::L2(h) => h.search(vector, k, ef),
                LoadedGraph::Dot(h) => h.search(vector, k, ef),
                LoadedGraph::Cosine(h) => h.search(&normalize_unit_safe(vector), k, ef),
            };
            hits.into_iter()
                .map(|hit| (hit.d_id, hit.distance))
                .collect()
        })
    }

    fn dump(&self, path: &Path) -> Result<String> {
        self.with_dependent(|_, graph| match graph {
            LoadedGraph::L2(h) => h.file_dump(path, BASENAME),
            LoadedGraph::Dot(h) | LoadedGraph::Cosine(h) => h.file_dump(path, BASENAME),
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    platform: String,
    fingerprint: String,
    next_id: usize,
    points: usize,
    ids: Vec<(String, usize)>,
    graph_bytes: u64,
    graph_sha256: String,
    data_bytes: u64,
    data_sha256: String,
}

fn platform() -> String {
    format!(
        "{}-{}-{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        usize::BITS,
        cfg!(target_endian = "little")
    )
}

fn fingerprint(
    spec: VectorSpec,
    vectors: &[(String, Vec<f32>)],
    codebook: Option<ScalarCodebook>,
) -> Result<String> {
    let mut hash = Sha256::new();
    hash.update(FORMAT.as_bytes());
    for setting in [HNSW_MAX_NB_CONNECTION, HNSW_EF_CONSTRUCTION, HNSW_MAX_LAYER] {
        hash.update((setting as u64).to_le_bytes());
    }
    hash.update(serde_json::to_vec(&spec)?);
    if let Some(book) = codebook {
        hash.update([1]);
        hash.update(book.min.to_bits().to_le_bytes());
        hash.update(book.max.to_bits().to_le_bytes());
        hash.update((book.dim as u64).to_le_bytes());
    } else {
        hash.update([0]);
    }
    let mut ordered: Vec<_> = vectors.iter().collect();
    ordered.sort_by(|left, right| left.0.cmp(&right.0));
    hash.update((ordered.len() as u64).to_le_bytes());
    for (eid, vector) in ordered {
        hash.update((eid.len() as u64).to_le_bytes());
        hash.update(eid.as_bytes());
        hash.update((vector.len() as u64).to_le_bytes());
        for value in vector {
            hash.update(value.to_bits().to_le_bytes());
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn real_directory(path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "cache directory is not a real directory"
    );
    Ok(())
}

fn file_hash(path: &Path, sync: bool) -> Result<(u64, String)> {
    let metadata = std::fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "cache payload is not a regular file"
    );
    let mut file = OpenOptions::new().read(true).write(sync).open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let size = file.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hash.update(&buffer[..size]);
    }
    if sync {
        file.sync_all()?;
    }
    Ok((metadata.len(), format!("{:x}", hash.finalize())))
}

fn graph_points(backend: &HnswBackend) -> usize {
    match backend {
        HnswBackend::L2(h) => h.get_nb_point(),
        HnswBackend::Cosine(h) | HnswBackend::Dot(h) => h.get_nb_point(),
        HnswBackend::Cached(h) => h.with_dependent(|_, graph| match graph {
            LoadedGraph::L2(h) => h.get_nb_point(),
            LoadedGraph::Cosine(h) | LoadedGraph::Dot(h) => h.get_nb_point(),
        }),
    }
}

pub(super) fn save(inner: &HnswCpuInner, directory: &Path) -> Result<bool> {
    if inner.store.len() == 0 {
        return Ok(false);
    }
    real_directory(directory)?;
    let vectors: Vec<_> = inner.store.iter_decoded().collect();
    let points = graph_points(&inner.hnsw);
    // Large orphan-only graphs are an optional cache miss, not a recovery
    // requirement. The ordinary authoritative rebuild compacts them.
    if points > vectors.len().saturating_mul(2) {
        return Ok(false);
    }
    // Only this private staging namespace belongs to the cache writer. Never
    // follow links or remove unrelated entries while recovering a partial dump.
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let owned = name
            .to_str()
            .and_then(|name| name.strip_prefix(".graph-stage-"))
            .is_some_and(|suffix| {
                suffix.len() == 6 && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
            });
        if owned && entry.file_type()?.is_dir() {
            std::fs::remove_dir_all(entry.path())?;
        }
    }
    let staging = tempfile::Builder::new()
        .prefix(".graph-stage-")
        .tempdir_in(directory)?;
    let name = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match &inner.hnsw {
        HnswBackend::L2(h) => h.file_dump(staging.path(), BASENAME),
        HnswBackend::Dot(h) | HnswBackend::Cosine(h) => h.file_dump(staging.path(), BASENAME),
        HnswBackend::Cached(h) => h.dump(staging.path()),
    }))
    .map_err(|_| anyhow!("optional HNSW dump panicked"))??;
    ensure!(name == BASENAME, "unexpected graph dump basename");
    let (graph_bytes, graph_sha256) = file_hash(&staging.path().join("graph.hnsw.graph"), true)?;
    let (data_bytes, data_sha256) = file_hash(&staging.path().join("graph.hnsw.data"), true)?;
    let mut ids: Vec<_> = inner
        .eid_to_id
        .iter()
        .map(|(eid, id)| (eid.clone(), *id))
        .collect();
    ids.sort_by(|left, right| left.0.cmp(&right.0));
    let manifest = Manifest {
        format: FORMAT.into(),
        platform: platform(),
        fingerprint: fingerprint(inner.store.spec, &vectors, inner.store.codebook)?,
        next_id: inner.next_id,
        points,
        ids,
        graph_bytes,
        graph_sha256,
        data_bytes,
        data_sha256,
    };
    // Rename replaces directory entries, never writes through an old symlink.
    // A crash between payloads leaves mismatched old checksums and a cache miss.
    for name in ["graph.hnsw.graph", "graph.hnsw.data"] {
        std::fs::rename(staging.path().join(name), directory.join(name))?;
    }
    storage_durable::atomic_write_strict(
        directory.join(MANIFEST),
        &serde_json::to_vec(&manifest)?,
    )?;
    Ok(true)
}

pub(super) fn load_timed(
    spec: VectorSpec,
    vectors: &[(String, Vec<f32>)],
    codebook: Option<ScalarCodebook>,
    directory: &Path,
) -> std::result::Result<(Option<HnswCpuIndex>, LoadTiming), LoadFailure> {
    let mut timing = LoadTiming::default();

    macro_rules! stage {
        ($field:ident, $body:expr) => {{
            let started = Instant::now();
            let result = (|| -> Result<_> { $body })();
            timing.$field = started.elapsed();
            result
        }};
    }

    let result = (|| -> Result<Option<HnswCpuIndex>> {
        let available = stage!(prepare, {
            if vectors.is_empty() || !directory.exists() {
                Ok(false)
            } else {
                real_directory(directory).map(|()| true)
            }
        });
        if !available? {
            return Ok(None);
        }

        let path = directory.join(MANIFEST);
        let manifest: Manifest = stage!(manifest, {
            let metadata = std::fs::symlink_metadata(&path)?;
            // Bound parsing by the authoritative rows, including JSON string escaping.
            let max_manifest = vectors.iter().fold(1024usize * 1024, |sum, (eid, _)| {
                sum.saturating_add(eid.len().saturating_mul(6).saturating_add(128))
            });
            ensure!(
                metadata.is_file()
                    && !metadata.file_type().is_symlink()
                    && metadata.len() <= max_manifest as u64,
                "invalid graph cache manifest file"
            );
            let manifest: Manifest = serde_json::from_slice(&std::fs::read(&path)?)?;
            ensure!(
                manifest.format == FORMAT && manifest.platform == platform(),
                "incompatible graph cache"
            );
            Ok::<_, anyhow::Error>(manifest)
        })?;
        let expected_fingerprint = stage!(fingerprint, fingerprint(spec, vectors, codebook))?;
        ensure!(
            manifest.fingerprint == expected_fingerprint,
            "graph cache differs from durable vectors"
        );
        ensure!(
            manifest.ids.len() == vectors.len()
                && manifest.points >= vectors.len()
                && manifest.points <= vectors.len().saturating_mul(2),
            "invalid cached graph population"
        );
        ensure!(
            manifest.next_id >= manifest.points && manifest.next_id < usize::MAX,
            "invalid cached allocator"
        );

        stage!(payload_hash, {
            for (name, expected_size, expected_hash) in [
                (
                    "graph.hnsw.graph",
                    manifest.graph_bytes,
                    &manifest.graph_sha256,
                ),
                (
                    "graph.hnsw.data",
                    manifest.data_bytes,
                    &manifest.data_sha256,
                ),
            ] {
                let (size, hash) = file_hash(&directory.join(name), false)?;
                ensure!(
                    size == expected_size && &hash == expected_hash,
                    "graph cache payload checksum mismatch"
                );
            }
            Ok::<_, anyhow::Error>(())
        })?;

        let (store, eid_to_id, id_to_eid, expected_points) = stage!(materialize, {
            let mut store = VectorStore::new(spec);
            store.codebook = codebook;
            let mut ordered: Vec<_> = vectors.iter().collect();
            ordered.sort_by(|left, right| left.0.cmp(&right.0));
            let mut eid_to_id = HashMap::with_capacity(vectors.len());
            let mut id_to_eid = HashMap::with_capacity(vectors.len());
            let mut expected_points = HashMap::with_capacity(vectors.len());
            for ((eid, vector), (cached_eid, id)) in ordered.into_iter().zip(&manifest.ids) {
                ensure!(
                    eid == cached_eid && *id < manifest.next_id,
                    "graph cache identity mismatch"
                );
                ensure!(
                    id_to_eid.insert(*id, eid.clone()).is_none(),
                    "duplicate cached graph identity"
                );
                ensure!(
                    eid_to_id.insert(eid.clone(), *id).is_none(),
                    "duplicate durable vector identity"
                );
                store.put(eid, vector)?;
                let decoded = store
                    .get_decoded(eid)
                    .context("cached store vector missing")?;
                expected_points.insert(
                    *id,
                    if spec.metric == VectorMetric::Cosine {
                        normalize_unit_safe(&decoded)
                    } else {
                        decoded
                    },
                );
            }
            Ok::<_, anyhow::Error>((store, eid_to_id, id_to_eid, expected_points))
        })?;
        let owned = stage!(deserialize, {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                OwnedGraph::try_new(MutBorrow::new(HnswIo::new(directory, BASENAME)), |owner| {
                    let loader = owner.borrow_mut();
                    Ok::<_, anyhow::Error>(match spec.metric {
                        VectorMetric::L2 => LoadedGraph::L2(loader.load_hnsw::<f32, DistL2>()?),
                        VectorMetric::Cosine => {
                            LoadedGraph::Cosine(loader.load_hnsw::<f32, DistDot>()?)
                        }
                        VectorMetric::Dot => LoadedGraph::Dot(loader.load_hnsw::<f32, DistDot>()?),
                    })
                })
            }))
            .map_err(|_| anyhow!("optional HNSW loader rejected malformed graph"))?
        })?;
        stage!(validate, {
            owned.with_dependent(|_, graph| match graph {
                LoadedGraph::L2(h) => {
                    validate_points(h, manifest.points, manifest.next_id, &expected_points)
                }
                LoadedGraph::Cosine(h) | LoadedGraph::Dot(h) => {
                    validate_points(h, manifest.points, manifest.next_id, &expected_points)
                }
            })
        })?;
        Ok(Some(HnswCpuIndex {
            inner: RwLock::new(HnswCpuInner {
                store,
                eid_to_id,
                id_to_eid,
                next_id: manifest.next_id,
                hnsw: HnswBackend::Cached(Box::new(owned)),
                ef_search: hnsw_search_ef(),
            }),
            exact_scan_fallbacks: AtomicU64::new(0),
        }))
    })();

    result
        .map(|index| (index, timing))
        .map_err(|error| LoadFailure { error, timing })
}

fn validate_points<D: hnsw_rs::anndists::dist::Distance<f32> + Send + Sync>(
    graph: &Hnsw<'_, f32, D>,
    points: usize,
    next_id: usize,
    expected: &HashMap<usize, Vec<f32>>,
) -> Result<()> {
    ensure!(
        graph.get_nb_point() == points,
        "graph cache point count mismatch"
    );
    ensure!(
        graph.get_max_nb_connection() as usize == HNSW_MAX_NB_CONNECTION
            && graph.get_ef_construction() == HNSW_EF_CONSTRUCTION,
        "cached graph construction quality differs"
    );
    let mut seen = std::collections::HashSet::with_capacity(expected.len());
    for point in graph.get_point_indexation() {
        ensure!(
            point.get_origin_id() < next_id,
            "cached allocator overlaps an existing graph node"
        );
        if let Some(vector) = expected.get(&point.get_origin_id()) {
            ensure!(
                seen.insert(point.get_origin_id()),
                "duplicate graph point identity"
            );
            ensure!(
                point.get_v().len() == vector.len()
                    && point
                        .get_v()
                        .iter()
                        .zip(vector)
                        .all(|(a, b)| a.to_bits() == b.to_bits()),
                "cached graph vector differs from durable store"
            );
        }
    }
    ensure!(
        seen.len() == expected.len(),
        "cached graph lost a live identity"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_cache_allocator_must_exceed_orphan_ids_too() {
        let (spec, index) = fixture(VectorMetric::L2);
        {
            let mut inner = index.inner.write().unwrap();
            inner.hnsw.insert(&[0.4, 0.2], 100);
            inner.next_id = 101;
        }
        let directory = tempfile::tempdir().unwrap();
        save(&index.inner.read().unwrap(), directory.path()).unwrap();
        let path = directory.path().join(MANIFEST);
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        manifest["next_id"] = serde_json::json!(33);
        std::fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let (vectors, codebook) = index.dump_for_snapshot().unwrap();
        assert!(
            load(spec, &vectors, codebook, directory.path()).is_err(),
            "a future live ID must never collide with an orphan node"
        );
    }

    #[test]
    fn graph_cache_preserves_a_stable_scalar_codebook() {
        let spec = VectorSpec {
            dim: 2,
            metric: VectorMetric::Cosine,
            backend: crate::types::VectorBackend::HnswCpu,
            quantize: Some(VectorQuantize::Sq),
        };
        let index = HnswCpuIndex::new(spec);
        index.add("bounds", &[0.0, 1.0]).unwrap();
        for i in 1..32 {
            index
                .add(&format!("sq-{i}"), &[i as f32 / 32.0, 0.5])
                .unwrap();
        }
        let directory = tempfile::tempdir().unwrap();
        save(&index.inner.read().unwrap(), directory.path()).unwrap();
        let (vectors, codebook) = index.dump_for_snapshot().unwrap();
        let loaded = load(spec, &vectors, codebook, directory.path())
            .unwrap()
            .unwrap();
        for (eid, value) in &vectors {
            assert_eq!(loaded.checkpoint_vector(eid).unwrap().as_ref(), Some(value));
        }
        assert_eq!(
            loaded.search_knn(&[0.251, 0.5], 5).unwrap(),
            index.search_knn(&[0.251, 0.5], 5).unwrap()
        );
    }

    #[test]
    fn graph_cache_removes_only_its_abandoned_staging_directory() {
        let (_, index) = fixture(VectorMetric::L2);
        let directory = tempfile::tempdir().unwrap();
        let partial = tempfile::Builder::new()
            .prefix(".graph-stage-")
            .tempdir_in(directory.path())
            .unwrap()
            .keep();
        std::fs::write(partial.join("partial"), b"partial cache").unwrap();
        let unrelated = directory.path().join("keep-this-directory");
        std::fs::create_dir(&unrelated).unwrap();
        std::fs::write(unrelated.join("sentinel"), b"keep").unwrap();
        save(&index.inner.read().unwrap(), directory.path()).unwrap();
        assert!(!partial.exists());
        assert_eq!(std::fs::read(unrelated.join("sentinel")).unwrap(), b"keep");
    }

    fn fixture(metric: VectorMetric) -> (VectorSpec, HnswCpuIndex) {
        let spec = VectorSpec {
            dim: 2,
            metric,
            backend: crate::types::VectorBackend::HnswCpu,
            quantize: None,
        };
        let index = HnswCpuIndex::new(spec);
        for i in 0..32 {
            index
                .add(&format!("id-{i}"), &[(i as f32 + 1.0) / 64.0, 0.2])
                .unwrap();
        }
        (spec, index)
    }

    #[test]
    fn graph_cache_roundtrips_all_metrics_and_can_be_saved_again() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OwnedGraph>();
        for metric in [VectorMetric::Cosine, VectorMetric::Dot, VectorMetric::L2] {
            let (spec, index) = fixture(metric);
            let expected = index.search_knn(&[0.173, 0.2], 10).unwrap();
            let directory = tempfile::tempdir().unwrap();
            let (vectors, codebook) = index.dump_for_snapshot().unwrap();
            save(&index.inner.read().unwrap(), directory.path()).unwrap();
            let loaded = load(spec, &vectors, codebook, directory.path())
                .unwrap()
                .unwrap();
            assert_eq!(loaded.search_knn(&[0.173, 0.2], 10).unwrap(), expected);
            save(&loaded.inner.read().unwrap(), directory.path()).unwrap();
            let loaded_again = load(spec, &vectors, codebook, directory.path())
                .unwrap()
                .unwrap();
            drop(directory);
            assert_eq!(
                loaded_again.search_knn(&[0.173, 0.2], 10).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn graph_cache_rejects_stale_content_incompatible_metadata_and_bad_payloads() {
        let (spec, index) = fixture(VectorMetric::L2);
        let directory = tempfile::tempdir().unwrap();
        let (vectors, codebook) = index.dump_for_snapshot().unwrap();
        save(&index.inner.read().unwrap(), directory.path()).unwrap();
        let mut stale = vectors.clone();
        stale[0].1[0] += 1.0;
        assert!(load(spec, &stale, codebook, directory.path()).is_err());
        assert!(load(
            VectorSpec {
                metric: VectorMetric::Dot,
                ..spec
            },
            &vectors,
            codebook,
            directory.path()
        )
        .is_err());
        for mutation in 0..6 {
            save(&index.inner.read().unwrap(), directory.path()).unwrap();
            let path = directory.path().join(MANIFEST);
            let mut manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            match mutation {
                0 => manifest["platform"] = serde_json::json!("foreign-platform"),
                1 => manifest["format"] = serde_json::json!("unknown-version"),
                2 => manifest["ids"][1][1] = manifest["ids"][0][1].clone(),
                3 => manifest["next_id"] = serde_json::json!(0),
                4 => manifest["points"] = serde_json::json!(0),
                _ => manifest["ids"][0][0] = serde_json::json!("foreign-id"),
            }
            std::fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
            assert!(
                load(spec, &vectors, codebook, directory.path()).is_err(),
                "mutation {mutation}"
            );
        }
        save(&index.inner.read().unwrap(), directory.path()).unwrap();
        std::fs::write(directory.path().join("graph.hnsw.graph"), b"corrupt").unwrap();
        assert!(load(spec, &vectors, codebook, directory.path()).is_err());
        let recovered =
            HnswCpuIndex::restore_with_graph_cache(spec, vectors, codebook, Some(directory.path()))
                .unwrap();
        assert_eq!(
            recovered.search_knn(&[0.173, 0.2], 10).unwrap(),
            index.search_knn(&[0.173, 0.2], 10).unwrap()
        );
    }

    #[test]
    fn graph_cache_restore_timing_distinguishes_hit_and_rejected_fallback() {
        let (spec, index) = fixture(VectorMetric::L2);
        let directory = tempfile::tempdir().unwrap();
        let (vectors, codebook) = index.dump_for_snapshot().unwrap();
        let expected = index.search_knn(&[0.173, 0.2], 10).unwrap();
        save(&index.inner.read().unwrap(), directory.path()).unwrap();

        let (loaded, hit_timing) = HnswCpuIndex::restore_with_graph_cache_timed(
            spec,
            vectors.clone(),
            codebook,
            Some(directory.path()),
        )
        .unwrap();
        assert_eq!(hit_timing.cache_result, Some(HnswGraphCacheResult::Hit));
        assert_eq!(hit_timing.fallback_rebuild, Duration::ZERO);
        assert!(hit_timing.total >= hit_timing.cache_deserialize);
        assert_eq!(loaded.search_knn(&[0.173, 0.2], 10).unwrap(), expected);

        std::fs::write(directory.path().join("graph.hnsw.graph"), b"corrupt").unwrap();
        let (rebuilt, rejected_timing) = HnswCpuIndex::restore_with_graph_cache_timed(
            spec,
            vectors,
            codebook,
            Some(directory.path()),
        )
        .unwrap();
        assert_eq!(
            rejected_timing.cache_result,
            Some(HnswGraphCacheResult::Rejected)
        );
        assert!(
            rejected_timing.cache_payload_hash > Duration::ZERO,
            "a rejected payload must retain its completed timing"
        );
        assert!(rejected_timing.total >= rejected_timing.fallback_rebuild);
        assert_eq!(
            rebuilt.search_knn(&[0.173, 0.2], 10).unwrap(),
            expected,
            "a rejected optional cache must retain the authoritative recovery outcome"
        );
    }

    #[test]
    fn graph_cache_format_requires_the_locked_codec_version() {
        let lock: toml::Value = toml::from_str(include_str!("../../../../Cargo.lock")).unwrap();
        let package = lock["package"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["name"].as_str() == Some("hnsw_rs"))
            .unwrap();
        assert_eq!(
            package["version"].as_str(),
            Some("0.3.4"),
            "a codec upgrade needs an explicit cache format review"
        );
    }

    #[cfg(unix)]
    #[test]
    fn graph_cache_refuses_symlink_directories_without_touching_the_target() {
        let (spec, index) = fixture(VectorMetric::L2);
        let directory = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("sentinel"), b"keep").unwrap();
        let alias = directory.path().join("alias");
        std::os::unix::fs::symlink(outside.path(), &alias).unwrap();
        assert!(save(&index.inner.read().unwrap(), &alias).is_err());
        let (vectors, codebook) = index.dump_for_snapshot().unwrap();
        assert!(load(spec, &vectors, codebook, &alias).is_err());
        assert_eq!(
            std::fs::read(outside.path().join("sentinel")).unwrap(),
            b"keep"
        );
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 1);
    }

    #[test]
    fn graph_cache_roundtrip_keeps_replaced_and_deleted_ids_out_of_results() {
        let spec = VectorSpec {
            dim: 2,
            metric: VectorMetric::L2,
            backend: crate::types::VectorBackend::HnswCpu,
            quantize: None,
        };
        let index = HnswCpuIndex::new(spec);
        for i in 0..64 {
            index.add(&format!("row-{i}"), &[i as f32, 1.0]).unwrap();
        }
        index.add("row-3", &[100.0, 1.0]).unwrap();
        index.remove("row-4").unwrap();
        let expected = index.search_knn(&[100.0, 1.0], 10).unwrap();
        let directory = tempfile::tempdir().unwrap();
        assert!(
            save(&index.inner.read().unwrap(), directory.path()).unwrap(),
            "a populated HNSW graph must have a cache"
        );
        let (vectors, codebook) = index.dump_for_snapshot().unwrap();
        let loaded = load(spec, &vectors, codebook, directory.path())
            .unwrap()
            .expect("identical durable vectors must accept their graph cache");
        assert_eq!(loaded.len(), 63);
        assert_eq!(loaded.search_knn(&[100.0, 1.0], 10).unwrap(), expected);
        assert_eq!(loaded.checkpoint_vector("row-4").unwrap(), None);
        loaded.add("after-load", &[101.0, 1.0]).unwrap();
        assert_eq!(
            loaded.search_knn(&[101.0, 1.0], 1).unwrap()[0].0,
            "after-load"
        );
    }
}
