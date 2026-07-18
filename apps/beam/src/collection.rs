//! In-memory vector collection — the storage substrate every index reads.
//!
//! A [`Collection`] holds its vectors row-major in one contiguous `Vec<f32>`
//! (`n * dim`) plus a parallel `Vec<String>` of external ids. This layout is
//! exactly what both the CPU oracle scan and the GPU storage-buffer upload
//! want: row `i` is the `dim`-long slice `data[i*dim .. (i+1)*dim]`, paired
//! with `external_ids[i]`.
//!
//! Metric handling is normalized at the storage boundary: for [`Metric::Cosine`]
//! the inserted vector is L2-normalized on `add`, so cosine similarity reduces
//! to a plain dot product at search time (the query is normalized by the index).
//! L2 and Dot store the raw vector unchanged.
//!
//! Alongside the vectors, each row carries an optional [`Payload`] of scalar
//! attributes, stored row-aligned in a parallel `Vec<Payload>`. Plain [`add`]
//! attaches the empty payload (so existing callers are unchanged);
//! [`add_with_payload`] attaches attributes. This is the metadata store that
//! filtered k-NN reads (see [`crate::payload`]).
//!
//! ## Mutability: external ids, tombstones, and append-only storage
//!
//! A real vector database mutates, so the collection is addressable by external
//! id and supports delete / update / upsert on top of the append-only vector
//! store:
//!
//! * An `id_map` (`external_id -> live row`) makes every row addressable by its
//!   external id; an id resolves to at most one live row.
//! * A per-row `live` bit tombstones rows. [`delete`] flips the bit and drops the
//!   id from the map; the physical row stays put (storage never shifts). Every
//!   index folds this live bit into the SAME keep-bitmask + sentinel path that
//!   filtered search uses, so a tombstoned row is excluded from all search (CPU
//!   oracle and every GPU path alike) exactly as a filtered-out row is.
//! * [`update`] is LSM-style — tombstone the old row, append a new live row under
//!   the same id — so index internals stay append-only and uniform across the
//!   flat and IVF backends (no in-place vector overwrite to special-case).
//! * [`len`] is the LIVE count; [`capacity`] is the physical row count (live +
//!   tombstoned). [`compact`] physically drops tombstones and renumbers rows.
//!
//! [`add`]: Collection::add
//! [`add_with_payload`]: Collection::add_with_payload
//! [`delete`]: Collection::delete
//! [`update`]: Collection::update
//! [`len`]: Collection::len
//! [`capacity`]: Collection::capacity
//! [`compact`]: Collection::compact

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::payload::Payload;
use crate::persist::{load_framed, save_framed, COLLECTION_MAGIC};

/// Distance / similarity metric a collection is scored under.
///
/// The numeric [`Metric::code`] is the convention shared verbatim by the CPU
/// oracle and the GPU WGSL kernel, so their per-row scores — and therefore their
/// top-k — agree bit-for-bit in intent (float rounding aside).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Metric {
    /// Squared Euclidean distance. Smaller score = closer = better.
    L2,
    /// Dot product. Larger score = better.
    Dot,
    /// Cosine similarity. Vectors are L2-normalized on insert so this is a dot
    /// product over unit vectors; larger score = better.
    Cosine,
}

impl Metric {
    /// Parse the CLI spelling (`l2` / `dot` / `cosine`, case-insensitive).
    pub fn parse(s: &str) -> Option<Metric> {
        match s.trim().to_ascii_lowercase().as_str() {
            "l2" | "euclidean" => Some(Metric::L2),
            "dot" | "ip" | "inner" => Some(Metric::Dot),
            "cosine" | "cos" => Some(Metric::Cosine),
            _ => None,
        }
    }

    /// The kernel/oracle metric code: L2=0, Dot=1, Cosine=2. This is the value
    /// handed to the GPU `Params` uniform and switched on in the CPU scorer.
    pub fn code(self) -> u32 {
        match self {
            Metric::L2 => 0,
            Metric::Dot => 1,
            Metric::Cosine => 2,
        }
    }

    /// Whether a larger score means a better (closer) match. L2 orders
    /// ascending (smaller better); Dot/Cosine order descending (larger better).
    pub fn larger_is_better(self) -> bool {
        !matches!(self, Metric::L2)
    }

    /// The "worst possible" score under this metric — the filter sentinel a
    /// non-matching row is assigned so it can never enter top-k: `+∞` for L2
    /// (smaller is better) and `-∞` for Dot/Cosine (larger is better).
    pub fn worst_score(self) -> f32 {
        if self.larger_is_better() {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        }
    }
}

/// L2-normalize `v` to unit length. A zero vector is returned unchanged (its
/// norm is 0, so there is nothing meaningful to scale — its dot with anything
/// is 0 regardless).
pub fn l2_normalize(v: &[f32]) -> Vec<f32> {
    let norm_sq: f32 = v.iter().map(|x| x * x).sum();
    if norm_sq <= 0.0 {
        return v.to_vec();
    }
    let inv = 1.0 / norm_sq.sqrt();
    v.iter().map(|x| x * inv).collect()
}

/// A named vector collection with a fixed dimension and metric.
///
/// In memory it is the storage substrate every index reads; on disk (via
/// [`Collection::save`] / [`Collection::load`]) it is a durable **segment** — the
/// row-major vectors, payloads, external ids, and `live` tombstone bits. The
/// `id_map` / `n_live` acceleration fields are NOT serialized (`#[serde(skip)]`):
/// they are rebuilt from `external_ids` + `live` on load, so the format stays the
/// minimal source-of-truth and can never disagree with itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Human/external identifier for the collection itself.
    pub id: String,
    dim: usize,
    metric: Metric,
    /// Row-major vectors: `capacity * dim` f32s, row `i` at `[i*dim .. (i+1)*dim]`.
    /// Append-only — tombstoned rows keep their slot (see `live`).
    data: Vec<f32>,
    /// External id per physical row, `external_ids[i]` pairs with row `i`. A
    /// tombstoned row keeps its (now stale) id here; only `id_map` is authoritative
    /// for id -> live row resolution.
    external_ids: Vec<String>,
    /// Attribute payload per physical row, `payloads[i]` pairs with row `i` (empty
    /// for rows added without one). The row-aligned metadata filtered search reads.
    payloads: Vec<Payload>,
    /// Liveness per physical row: `live[i] == false` marks row `i` tombstoned
    /// (deleted or superseded by an update). Folded into every index's keep-bitmask
    /// so tombstoned rows are excluded from search via the filter sentinel path.
    live: Vec<bool>,
    /// `external_id -> live physical row`. Only live ids are present, so an id
    /// resolves to at most one row; a delete removes the key, an update re-points it.
    /// Rebuilt from `external_ids` + `live` on [`Collection::load`], so it is not
    /// part of the on-disk format.
    #[serde(skip)]
    id_map: HashMap<String, u32>,
    /// Cached live-row count (`== id_map.len()`), so [`Collection::len`] is O(1).
    /// Recomputed alongside `id_map` on load, so it is not persisted.
    #[serde(skip)]
    n_live: usize,
}

impl Collection {
    /// Construct an empty collection with the given dimension and metric.
    pub fn new(id: impl Into<String>, dim: usize, metric: Metric) -> Self {
        Self {
            id: id.into(),
            dim,
            metric,
            data: Vec::new(),
            external_ids: Vec::new(),
            payloads: Vec::new(),
            live: Vec::new(),
            id_map: HashMap::new(),
            n_live: 0,
        }
    }

    /// Append a vector under `external_id` with an **empty** payload. The vector
    /// length must equal [`Collection::dim`]. For [`Metric::Cosine`] the vector
    /// is L2-normalized before storage so search can use a plain dot product.
    /// Use [`Collection::add_with_payload`] to attach attributes.
    pub fn add(&mut self, external_id: impl Into<String>, vector: &[f32]) -> anyhow::Result<()> {
        self.add_with_payload(external_id, vector, Payload::new())
    }

    /// Append a vector under `external_id` carrying `payload`. Same storage
    /// rules as [`Collection::add`]; the payload is stored row-aligned and read
    /// by filtered k-NN.
    pub fn add_with_payload(
        &mut self,
        external_id: impl Into<String>,
        vector: &[f32],
        payload: Payload,
    ) -> anyhow::Result<()> {
        let external_id = external_id.into();
        if vector.len() != self.dim {
            anyhow::bail!(
                "vector dim mismatch: collection dim is {}, got {}",
                self.dim,
                vector.len()
            );
        }
        // Adding under an id that already maps to a live row is an upsert: the old
        // row is tombstoned and a fresh live row appended (see [`Collection::update`]),
        // so an external id always resolves to at most one live row.
        if self.id_map.contains_key(&external_id) {
            self.update(&external_id, vector, payload);
        } else {
            self.append_row(external_id, vector, payload);
        }
        Ok(())
    }

    /// Push `vector` onto the row store, L2-normalizing first for [`Metric::Cosine`]
    /// (so search is a plain dot product) and storing it verbatim otherwise.
    fn store_vector(&mut self, vector: &[f32]) {
        match self.metric {
            Metric::Cosine => self.data.extend_from_slice(&l2_normalize(vector)),
            _ => self.data.extend_from_slice(vector),
        }
    }

    /// Append a brand-new live row for `external_id` (caller guarantees the id is
    /// not already live and the vector length matches `dim`).
    fn append_row(&mut self, external_id: String, vector: &[f32], payload: Payload) {
        let row = self.external_ids.len() as u32;
        self.store_vector(vector);
        self.external_ids.push(external_id.clone());
        self.payloads.push(payload);
        self.live.push(true);
        self.id_map.insert(external_id, row);
        self.n_live += 1;
    }

    /// Replace the payload of an existing row (used to assign deterministic
    /// attributes to a pre-built corpus in tests and the bench). Panics if `i`
    /// is out of range.
    pub fn set_payload(&mut self, i: usize, payload: Payload) {
        self.payloads[i] = payload;
    }

    /// Number of **live** vectors (excludes tombstoned rows). Use
    /// [`Collection::capacity`] for the physical row count.
    pub fn len(&self) -> usize {
        self.n_live
    }

    /// Whether the collection holds zero **live** vectors (it may still hold
    /// tombstoned rows; see [`Collection::capacity`]).
    pub fn is_empty(&self) -> bool {
        self.n_live == 0
    }

    /// Physical row count: live + tombstoned. This is the number of rows in the
    /// backing `data` / `external_ids` / `payloads` / `live` arrays, and the row
    /// range (`0..capacity`) every index iterates and masks over.
    pub fn capacity(&self) -> usize {
        self.external_ids.len()
    }

    /// Number of tombstoned (deleted or superseded) rows still occupying storage —
    /// what [`Collection::compact`] reclaims.
    pub fn tombstoned(&self) -> usize {
        self.capacity() - self.n_live
    }

    /// Fraction of physical rows that are tombstoned, in `[0, 1]` (`0` when empty).
    /// The signal a caller can threshold on to decide when to [`Collection::compact`].
    pub fn tombstone_ratio(&self) -> f64 {
        let cap = self.capacity();
        if cap == 0 {
            0.0
        } else {
            self.tombstoned() as f64 / cap as f64
        }
    }

    /// Resolve `external_id` to its live physical row, or `None` if it is unknown
    /// or has been deleted.
    pub fn row_of(&self, external_id: &str) -> Option<u32> {
        self.id_map.get(external_id).copied()
    }

    /// Whether `external_id` currently maps to a live row.
    pub fn contains(&self, external_id: &str) -> bool {
        self.id_map.contains_key(external_id)
    }

    /// Per physical row liveness bits (`live[i] == false` ⇒ row `i` is tombstoned).
    /// Indexes AND this into their keep-bitmask so tombstoned rows are excluded from
    /// search through the same sentinel path filtered k-NN uses.
    pub fn live(&self) -> &[bool] {
        &self.live
    }

    /// Whether physical row `i` is live (`false` for a tombstoned or out-of-range row).
    pub fn is_live(&self, row: u32) -> bool {
        self.live.get(row as usize).copied().unwrap_or(false)
    }

    /// Tombstone the live row addressed by `external_id`, returning `true` if a live
    /// row was removed (`false` if the id was unknown or already deleted). The
    /// physical row is retained (storage never shifts) but marked not-live, so every
    /// index excludes it from search on its next query. O(1).
    pub fn delete(&mut self, external_id: &str) -> bool {
        if let Some(row) = self.id_map.remove(external_id) {
            self.live[row as usize] = false;
            self.n_live -= 1;
            true
        } else {
            false
        }
    }

    /// Replace the vector + payload stored under `external_id` (LSM-style):
    /// tombstone the current row and append a new live row carrying the same id,
    /// re-pointing the id map. Returns `false` if the id is unknown or `new_vector`
    /// has the wrong dimension; the live count is unchanged on success.
    ///
    /// Append + tombstone (rather than an in-place overwrite) keeps the vector store
    /// append-only, so it is uniform across the flat and IVF backends — the IVF
    /// inverted lists only ever grow, and the superseded row is masked out by its
    /// live bit exactly like a delete.
    pub fn update(&mut self, external_id: &str, new_vector: &[f32], new_payload: Payload) -> bool {
        if new_vector.len() != self.dim {
            return false;
        }
        let Some(&old_row) = self.id_map.get(external_id) else {
            return false;
        };
        // Tombstone the superseded row (append-only: it keeps its slot, just loses
        // its live bit and its claim on the id).
        self.live[old_row as usize] = false;
        // Append the replacement as a new live row under the same id. Live count is
        // unchanged (one row retired, one added), so it is not touched here.
        let new_row = self.external_ids.len() as u32;
        self.store_vector(new_vector);
        self.external_ids.push(external_id.to_string());
        self.payloads.push(new_payload);
        self.live.push(true);
        self.id_map.insert(external_id.to_string(), new_row);
        true
    }

    /// Update `external_id` if it exists, else add it. Returns `Ok(true)` when an
    /// existing row was replaced and `Ok(false)` when a new row was added; errors
    /// only on a dimension mismatch.
    pub fn upsert(
        &mut self,
        external_id: impl Into<String>,
        vector: &[f32],
        payload: Payload,
    ) -> anyhow::Result<bool> {
        let external_id = external_id.into();
        if vector.len() != self.dim {
            anyhow::bail!(
                "vector dim mismatch: collection dim is {}, got {}",
                self.dim,
                vector.len()
            );
        }
        if self.id_map.contains_key(&external_id) {
            self.update(&external_id, vector, payload);
            Ok(true)
        } else {
            self.append_row(external_id, vector, payload);
            Ok(false)
        }
    }

    /// Physically drop every tombstoned row, compacting the backing arrays to the
    /// live rows and renumbering them `0..len`. After this, `capacity == len` and
    /// `tombstoned == 0`. Rebuild any index over the collection afterward — the row
    /// indices have changed.
    pub fn compact(&mut self) {
        if self.tombstoned() == 0 {
            return;
        }
        let dim = self.dim;
        let mut data = Vec::with_capacity(self.n_live * dim);
        let mut external_ids = Vec::with_capacity(self.n_live);
        let mut payloads = Vec::with_capacity(self.n_live);
        for row in 0..self.external_ids.len() {
            if self.live[row] {
                data.extend_from_slice(&self.data[row * dim..(row + 1) * dim]);
                external_ids.push(self.external_ids[row].clone());
                payloads.push(self.payloads[row].clone());
            }
        }
        self.id_map = external_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i as u32))
            .collect();
        self.live = vec![true; external_ids.len()];
        self.n_live = external_ids.len();
        self.data = data;
        self.external_ids = external_ids;
        self.payloads = payloads;
    }

    /// The vector dimension every row has.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// The collection's metric.
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// The full row-major `capacity * dim` backing buffer (live + tombstoned rows)
    /// — the exact slice uploaded to the GPU storage buffer and scanned by the CPU
    /// oracle. Tombstoned rows are present here but excluded at scoring time via the
    /// live-mask, so an index materializes this whole buffer once and masks.
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// The external ids, indexed by row.
    pub fn external_ids(&self) -> &[String] {
        &self.external_ids
    }

    /// The attribute payloads, indexed by row (row-aligned with the vectors).
    pub fn payloads(&self) -> &[Payload] {
        &self.payloads
    }

    /// Row `i`'s attribute payload.
    pub fn payload(&self, i: usize) -> &Payload {
        &self.payloads[i]
    }

    /// Row `i`'s `dim`-long vector slice (as stored — already normalized for
    /// Cosine).
    pub fn row(&self, i: usize) -> &[f32] {
        &self.data[i * self.dim..(i + 1) * self.dim]
    }

    /// Persist this collection segment to `path`: the row-major vectors, payloads,
    /// external ids, and `live` tombstone bits, behind beam's magic + version
    /// header (see [`crate::persist`]). The `id_map` / `n_live` accelerators are
    /// omitted — they are rebuilt on [`Collection::load`], so the file is the
    /// minimal source-of-truth. A cold [`Collection::load`] reproduces this exact
    /// collection (same rows, tombstones, and search results) with no rebuild cost.
    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        save_framed(path.as_ref(), COLLECTION_MAGIC, self)
    }

    /// Load a collection segment previously written by [`Collection::save`],
    /// rebuilding the `id_map` (external id -> live row) and the cached live count
    /// from the persisted `external_ids` + `live` bits. Errors on a missing file, a
    /// wrong magic (not a collection file), or an unknown format version.
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut collection: Collection = load_framed(path.as_ref(), COLLECTION_MAGIC)?;
        collection.rebuild_id_map();
        Ok(collection)
    }

    /// Rebuild the `id_map` and `n_live` cache from `external_ids` + `live` — the
    /// post-load fixup for the `#[serde(skip)]` acceleration fields. Every live row
    /// claims its external id (an id resolves to at most one live row by
    /// construction, since `update`/`delete` retire the prior row's live bit), so
    /// this reproduces the exact map the in-memory collection held at save time.
    pub fn rebuild_id_map(&mut self) {
        let mut id_map = HashMap::with_capacity(self.external_ids.len());
        let mut n_live = 0usize;
        for row in 0..self.external_ids.len() {
            if self.live[row] {
                id_map.insert(self.external_ids[row].clone(), row as u32);
                n_live += 1;
            }
        }
        self.id_map = id_map;
        self.n_live = n_live;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_parse_and_code() {
        assert_eq!(Metric::parse("L2"), Some(Metric::L2));
        assert_eq!(Metric::parse("Dot"), Some(Metric::Dot));
        assert_eq!(Metric::parse("cosine"), Some(Metric::Cosine));
        assert_eq!(Metric::parse("nope"), None);
        assert_eq!(Metric::L2.code(), 0);
        assert_eq!(Metric::Dot.code(), 1);
        assert_eq!(Metric::Cosine.code(), 2);
    }

    #[test]
    fn add_validates_length() {
        let mut c = Collection::new("t", 3, Metric::L2);
        assert!(c.add("a", &[1.0, 2.0, 3.0]).is_ok());
        assert!(c.add("b", &[1.0, 2.0]).is_err());
        assert_eq!(c.len(), 1);
        assert_eq!(c.row(0), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn cosine_normalizes_on_insert() {
        let mut c = Collection::new("t", 2, Metric::Cosine);
        c.add("a", &[3.0, 4.0]).unwrap();
        let row = c.row(0);
        let norm: f32 = row.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "row norm should be 1, got {norm}");
        assert!((row[0] - 0.6).abs() < 1e-6);
        assert!((row[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn delete_tombstones_and_drops_from_id_map() {
        let mut c = Collection::new("t", 2, Metric::L2);
        c.add("a", &[0.0, 0.0]).unwrap();
        c.add("b", &[1.0, 0.0]).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c.capacity(), 2);

        assert!(c.delete("a"));
        assert_eq!(c.len(), 1, "live count drops");
        assert_eq!(c.capacity(), 2, "physical storage retained (append-only)");
        assert_eq!(c.tombstoned(), 1);
        assert!(!c.is_live(0));
        assert!(c.is_live(1));
        assert!(c.row_of("a").is_none());
        assert_eq!(c.row_of("b"), Some(1));

        // Deleting an unknown / already-deleted id is a no-op returning false.
        assert!(!c.delete("a"));
        assert!(!c.delete("missing"));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn update_tombstones_old_and_appends_new_same_id() {
        let mut c = Collection::new("t", 2, Metric::L2);
        c.add("a", &[0.0, 0.0]).unwrap();
        c.add("b", &[1.0, 0.0]).unwrap();

        assert!(c.update("a", &[9.0, 9.0], Payload::new()));
        assert_eq!(c.len(), 2, "update keeps live count");
        assert_eq!(c.capacity(), 3, "old row retained, new row appended");
        // The id now resolves to the NEW row, carrying the new vector.
        let row = c.row_of("a").unwrap();
        assert_eq!(row, 2);
        assert_eq!(c.row(row as usize), &[9.0, 9.0]);
        assert!(!c.is_live(0), "old row tombstoned");

        // Update of an unknown id, or a wrong-dimension vector, fails without change.
        assert!(!c.update("missing", &[1.0, 1.0], Payload::new()));
        assert!(!c.update("b", &[1.0, 1.0, 1.0], Payload::new()));
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn add_of_existing_id_is_an_upsert() {
        let mut c = Collection::new("t", 2, Metric::L2);
        c.add("a", &[0.0, 0.0]).unwrap();
        // Re-adding the same id replaces (upserts) rather than duplicating it.
        c.add("a", &[5.0, 5.0]).unwrap();
        assert_eq!(c.len(), 1, "id is not duplicated");
        assert_eq!(c.row(c.row_of("a").unwrap() as usize), &[5.0, 5.0]);
    }

    #[test]
    fn upsert_adds_then_replaces() {
        let mut c = Collection::new("t", 2, Metric::L2);
        assert!(!c.upsert("a", &[0.0, 0.0], Payload::new()).unwrap(), "new id → added");
        assert_eq!(c.len(), 1);
        assert!(c.upsert("a", &[2.0, 2.0], Payload::new()).unwrap(), "existing id → replaced");
        assert_eq!(c.len(), 1, "replace keeps live count");
        assert_eq!(c.row(c.row_of("a").unwrap() as usize), &[2.0, 2.0]);
    }

    #[test]
    fn compact_drops_tombstones_and_renumbers() {
        let mut c = Collection::new("t", 2, Metric::L2);
        for i in 0..5 {
            c.add(format!("v{i}"), &[i as f32, 0.0]).unwrap();
        }
        c.delete("v1");
        c.update("v3", &[30.0, 0.0], Payload::new());
        let live_before = c.len();
        assert!(c.tombstoned() > 0);

        c.compact();
        assert_eq!(c.len(), live_before, "live rows survive compaction");
        assert_eq!(c.capacity(), live_before, "tombstones reclaimed");
        assert_eq!(c.tombstoned(), 0);
        // Surviving ids still resolve to their (now renumbered) live rows/vectors.
        assert!(c.row_of("v1").is_none());
        assert_eq!(c.row(c.row_of("v3").unwrap() as usize), &[30.0, 0.0]);
        assert_eq!(c.row(c.row_of("v0").unwrap() as usize), &[0.0, 0.0]);
    }
}
