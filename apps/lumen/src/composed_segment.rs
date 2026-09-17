//! Immutable scalar and text views over one base and ordered sparse deltas.
//!
//! Coverage includes absent rows. It therefore hides every older value even
//! when the newest row is a deletion. Only the ID maps live in memory; payload
//! and postings stay in their original mmaps. Dictionary walks keep one head
//! per segment and never materialize the complete dictionary.

use crate::segment::{
    cached_text_posting_weight, posting_cache_bytes, CachedTextPosting, SegmentReader,
    SortedIdCursor,
};
use anyhow::{anyhow, bail, Result};
use roaring::RoaringBitmap;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

#[path = "composed_segment/checkpoint_publication.rs"]
mod checkpoint_publication;
pub(crate) use checkpoint_publication::{PreparedScalarPublication, ScalarCheckpointCut};

#[derive(Debug)]
struct DeltaLayer {
    reader: Arc<SegmentReader>,
    ids: Vec<u32>,
    local_by_global: BTreeMap<u32, u32>,
    coverage: RoaringBitmap,
    /// Private layers are post-capture runtime state. They never describe the
    /// durable catalog and generic compaction cannot consume them.
    private: bool,
}

/// Catalog deltas and post-checkpoint private deltas share one query-time
/// composition bound. The base reader is not an incremental layer.
pub(crate) const MAX_INCREMENTAL_LAYERS: usize = 16;

/// A Text token's posting resolved for a small candidate set by
/// [`ComposedSegmentReader::text_posting_at`] (#4246).
#[derive(Debug)]
pub(crate) enum TextPostingAt {
    /// The full composed posting was already resident; the caller uses it as
    /// it would a `text_postings_arc` result.
    Cached(CachedTextPosting),
    /// The posting was cold and was streamed, not materialized: the exact df
    /// of the composed posting minus hidden ids, and the `(id, tf)` of every
    /// candidate that holds the token, ascending by id.
    Sparse { df: usize, hits: Vec<(u32, u32)> },
}

// #4246 cost oracle: every Text posting `text_tokens_all` materializes as an
// owned pair of vectors. A distinct-term count must materialize none.
#[cfg(test)]
thread_local! {
    static TEXT_POSTING_CLONES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_text_posting_clones() {
    TEXT_POSTING_CLONES.with(|clones| clones.set(0));
}

#[cfg(test)]
pub(crate) fn text_posting_clones() -> u64 {
    TEXT_POSTING_CLONES.with(std::cell::Cell::get)
}

// #4246 cost oracle: dictionary terms visited and composed postings decoded on
// behalf of one read. A distinct-term count must be independent of both.
#[cfg(test)]
thread_local! {
    static TEXT_TERM_PROBES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_text_term_probes() {
    TEXT_TERM_PROBES.with(|probes| probes.set(0));
}

#[cfg(test)]
pub(crate) fn text_term_probes() -> u64 {
    TEXT_TERM_PROBES.with(std::cell::Cell::get)
}

#[inline]
fn note_text_term_probes(_probes: u64) {
    #[cfg(test)]
    TEXT_TERM_PROBES.with(|probes| probes.set(probes.get().saturating_add(_probes)));
}

#[inline]
fn note_text_posting_clones(_clones: u64) {
    #[cfg(test)]
    TEXT_POSTING_CLONES.with(|clones| clones.set(clones.get().saturating_add(_clones)));
}

#[derive(Debug, thiserror::Error)]
#[error(
    "scalar field has reached the {MAX_INCREMENTAL_LAYERS}-layer incremental composition limit"
)]
pub(crate) struct LayerCapacityFull;

#[derive(Clone, Debug)]
pub(crate) struct ComposedSegmentReader {
    base: Arc<SegmentReader>,
    base_map: Option<Arc<DeltaLayer>>,
    layers: Vec<Arc<DeltaLayer>>,
    n_docs: u32,
    /// False only for the synthetic empty base used before the first catalog
    /// checkpoint. It must never escape as a catalog identity or compaction input.
    has_catalog_base: bool,
    /// EXACT count of dictionary terms with a non-empty composed posting, when
    /// it is known without a walk (#4246).
    ///
    /// `Some(n)` is an invariant, not a hint: it is carried only while every
    /// composition step has been ADDITIVE — a sealed base (whose projection
    /// writes only terms that had a live document) plus delta layers that
    /// covered no runtime ID which already held a Text value. Under that
    /// invariant no older posting can be shadowed empty, so the composed term
    /// set is exactly the union of the layer dictionaries and each attach adds
    /// the delta's terms that no lower dictionary holds — work bounded by one
    /// checkpoint interval of writes, paid once per publication. Any step that
    /// can hide an older row (an update, a delete, a compaction, a base
    /// replacement) sets `None`, and the reader that needs the number walks for
    /// it instead of reporting an approximation.
    distinct_terms: Option<u64>,
    /// Query-time caches derived purely from this composition (#4246); see
    /// [`QueryCache`]. Shared by clones of the same composition, fresh for
    /// every constructor that changes the base or the layers.
    query_cache: Arc<QueryCache>,
}

/// Results a `ComposedSegmentReader` derives from nothing but its immutable
/// base + layers, so they stay valid for the composition's whole life
/// (#4246). Without them a composition that is not a bare dense base paid,
/// on EVERY probe, a full re-decode + re-merge of a token's posting through
/// every layer (`text_postings_arc`) and one `BTreeMap` probe per layer per
/// scored row for its doc length (`text_doc_len`) — 20 × 500k rows and 500k
/// winner walks per cold ngram AND query against the 500k-hot-doc fixture.
pub(crate) struct QueryCache {
    /// Composed `(docids, tfs)` per token, byte-budgeted with the same
    /// weigher and `LUMEN_SEG_POSTING_CACHE_MB` budget as the base reader's
    /// own `text_posting_cache`.
    text_postings: moka::sync::Cache<String, CachedTextPosting>,
    /// Global id → text doc length for every `id < n_docs`, materialized
    /// once by the first hot BM25 walk (see `text_doc_lens`).
    text_doc_lens: OnceLock<Vec<u32>>,
    /// Coverage unions shared by every token's `merge_text_postings`,
    /// materialized once per composition (see [`CoverageUnions`]).
    coverage_unions: OnceLock<CoverageUnions>,
}

/// What a composed text posting merge needs to know about its layers, once
/// per composition rather than once per token (#4246).
///
/// A base row survives only when NO layer covers its id, and a layer row only
/// when no NEWER layer covers it. With those two facts precomputed, one token's
/// merge is a filter of the base posting by `all` plus a filter of each layer's
/// (small) posting by its `newer_than` union — O(|coverage| · log df) galloping
/// over a sorted base posting instead of the O(df × layers) `retain` sweep that
/// cost ~58 ms per distinct ngram token against a 500k-row base in Docker.
struct CoverageUnions {
    /// Union of every layer's coverage.
    all: RoaringBitmap,
    /// `newer_than[l]` is the union of the coverage of every layer newer than
    /// layer `l` (empty for the newest layer).
    newer_than: Vec<RoaringBitmap>,
    /// Whether `base_map.ids` is strictly ascending, so the mapped base posting
    /// is already sorted by global id and needs no sort.
    base_map_ascending: bool,
}

/// Copies every `(id, tf)` of a posting sorted by ascending unique `id` whose
/// id is NOT in `covered`, preserving order. Sparse coverage gallops: each
/// covered id is located by binary search from the last hit and the runs
/// between hits are copied whole, O(|covered| · log df + df memcpy). Dense
/// coverage walks both sorted sequences in lockstep, O(df + |covered|).
fn retain_uncovered_sorted(
    ids: &[u32],
    tfs: &[u32],
    covered: &RoaringBitmap,
    out_ids: &mut Vec<u32>,
    out_tfs: &mut Vec<u32>,
) {
    debug_assert_eq!(ids.len(), tfs.len());
    out_ids.reserve(ids.len());
    out_tfs.reserve(ids.len());
    if covered.is_empty() || ids.is_empty() {
        out_ids.extend_from_slice(ids);
        out_tfs.extend_from_slice(tfs);
        return;
    }
    let last = *ids.last().expect("non-empty");
    if covered.len().saturating_mul(16) < ids.len() as u64 {
        let (mut start, mut pos) = (0usize, 0usize);
        for id in covered.iter() {
            if id > last || pos >= ids.len() {
                break;
            }
            match ids[pos..].binary_search(&id) {
                Ok(offset) => {
                    let hit = pos + offset;
                    out_ids.extend_from_slice(&ids[start..hit]);
                    out_tfs.extend_from_slice(&tfs[start..hit]);
                    start = hit + 1;
                    pos = hit + 1;
                }
                Err(offset) => pos += offset,
            }
        }
        out_ids.extend_from_slice(&ids[start..]);
        out_tfs.extend_from_slice(&tfs[start..]);
        return;
    }
    let mut hidden = covered.iter().peekable();
    for (&id, &tf) in ids.iter().zip(tfs) {
        while hidden.peek().is_some_and(|&h| h < id) {
            hidden.next();
        }
        if hidden.peek() == Some(&id) {
            continue;
        }
        out_ids.push(id);
        out_tfs.push(tf);
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self {
            text_postings: moka::sync::Cache::builder()
                .weigher(|k: &String, v: &CachedTextPosting| {
                    cached_text_posting_weight(v)
                        .saturating_add(k.len().min(u32::MAX as usize) as u32)
                })
                .max_capacity(posting_cache_bytes())
                .build(),
            text_doc_lens: OnceLock::new(),
            coverage_unions: OnceLock::new(),
        }
    }
}

impl std::fmt::Debug for QueryCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryCache")
            .field("text_postings", &self.text_postings.entry_count())
            .field("text_doc_lens", &self.text_doc_lens.get().map(Vec::len))
            .field(
                "coverage_unions",
                &self.coverage_unions.get().map(|u| u.all.len()),
            )
            .finish()
    }
}

impl ComposedSegmentReader {
    pub(crate) fn incremental_layer_count(&self) -> usize {
        self.layers.len()
    }

    pub(crate) fn has_private_layers(&self) -> bool {
        self.layers.iter().any(|layer| layer.private)
    }

    pub(crate) fn can_append_incremental(&self) -> Result<()> {
        if self.incremental_layer_count() >= MAX_INCREMENTAL_LAYERS {
            return Err(anyhow::Error::new(LayerCapacityFull));
        }
        Ok(())
    }
}

/// Row maps and coverage are built before publication. Installing this object
/// checks immutable identities and moves only layer Arcs, never row arrays.
pub(crate) struct PreparedScalarReplacement {
    base: Arc<SegmentReader>,
    inputs: Vec<Arc<SegmentReader>>,
    replacement: Arc<DeltaLayer>,
    includes_base: bool,
    n_docs: u32,
    has_catalog_base: bool,
    catalog_len: usize,
}

/// Build a compact private row space from immutable layers in age order.
/// Deleted rows remain in the row map, so partial merges cannot reveal old data.
pub(crate) fn compose_checkpoint_layers(
    layers: Vec<(Arc<SegmentReader>, Vec<String>)>,
) -> Result<(ComposedSegmentReader, Vec<String>)> {
    if layers.is_empty() {
        bail!("checkpoint compaction needs at least one layer");
    }
    let mut external = BTreeSet::new();
    for (reader, ids) in &layers {
        if reader.n_docs() as usize != ids.len() {
            bail!("checkpoint row map does not match segment row count");
        }
        let mut seen = BTreeSet::new();
        for eid in ids {
            if !seen.insert(eid) {
                bail!("checkpoint row map contains duplicate external ID");
            }
            external.insert(eid.clone());
        }
    }
    let output_ids: Vec<String> = external.into_iter().collect();
    u32::try_from(output_ids.len()).map_err(|_| anyhow!("compaction row space exceeds u32"))?;
    let dense: BTreeMap<&str, u32> = output_ids
        .iter()
        .enumerate()
        .map(|(id, eid)| (eid.as_str(), id as u32))
        .collect();
    let mut layers = layers.into_iter().map(|(reader, ids)| {
        let mapped = ids.iter().map(|eid| dense[eid.as_str()]).collect();
        (reader, mapped)
    });
    let (reader, ids) = layers.next().expect("checked nonempty");
    let mut view = ComposedSegmentReader::from_mapped_base(reader, ids)?;
    for (reader, ids) in layers {
        view = view.with_delta(reader, ids)?;
    }
    Ok((view, output_ids))
}

impl ComposedSegmentReader {
    /// Only the layer-Arc vector is copied when a private delta is appended.
    /// Its new row map is priced separately from the selected source rows.
    pub(crate) fn private_append_metadata_bound(&self) -> Option<usize> {
        self.layers
            .len()
            .checked_add(1)?
            .checked_mul(2 * std::mem::size_of::<Arc<DeltaLayer>>())?
            .checked_add(std::mem::size_of::<Self>())
    }

    pub(crate) fn prepare_replacement(
        &self,
        base: Option<&Arc<SegmentReader>>,
        inputs: &[Arc<SegmentReader>],
        reader: Arc<SegmentReader>,
        ids: Vec<u32>,
    ) -> Result<PreparedScalarReplacement> {
        if !self.has_catalog_base {
            bail!("compaction requires a published catalog base");
        }
        let prepared = if let Some(base) = base {
            self.replace_base_inputs(base, inputs, reader.clone(), ids)?
        } else {
            self.replace_delta_inputs(inputs, reader.clone(), ids)?
        };
        let replacement = if base.is_some() {
            prepared
                .base_map
                .as_ref()
                .expect("mapped replacement base")
                .clone()
        } else {
            prepared
                .layers
                .iter()
                .find(|layer| Arc::ptr_eq(&layer.reader, &reader))
                .ok_or_else(|| anyhow!("prepared replacement layer is absent"))?
                .clone()
        };
        Ok(PreparedScalarReplacement {
            base: self.base.clone(),
            inputs: inputs.to_vec(),
            replacement,
            includes_base: base.is_some(),
            n_docs: prepared.n_docs,
            has_catalog_base: prepared.has_catalog_base,
            catalog_len: self
                .layers
                .iter()
                .take_while(|layer| !layer.private)
                .count(),
        })
    }

    pub(crate) fn install_prepared_replacement(
        &self,
        prepared: &PreparedScalarReplacement,
    ) -> Result<Self> {
        if !self.has_catalog_base
            || !prepared.has_catalog_base
            || !Arc::ptr_eq(&self.base, &prepared.base)
        {
            bail!("compacted base input no longer matches live base");
        }
        let matches = |window: &[Arc<DeltaLayer>]| {
            window.len() == prepared.inputs.len()
                && window
                    .iter()
                    .zip(&prepared.inputs)
                    .all(|(layer, input)| Arc::ptr_eq(&layer.reader, input))
        };
        if prepared.inputs.len() > prepared.catalog_len {
            bail!("prepared compaction includes private layers");
        }
        let n_docs = self.n_docs.max(prepared.n_docs);
        let live_catalog_len = self
            .layers
            .iter()
            .take_while(|layer| !layer.private)
            .count();
        if prepared.includes_base {
            if prepared.inputs.len() > live_catalog_len
                || !matches(&self.layers[..prepared.inputs.len()])
            {
                bail!("compacted base inputs no longer match live layers");
            }
            let layers = self.layers[prepared.inputs.len()..].to_vec();
            let distinct_terms = Self::refold_distinct_terms(
                &prepared.replacement.reader,
                Some(prepared.replacement.clone()),
                &layers,
                n_docs,
            );
            return Ok(Self {
                base: prepared.replacement.reader.clone(),
                base_map: Some(prepared.replacement.clone()),
                layers,
                n_docs,
                has_catalog_base: true,
                distinct_terms,
                query_cache: Arc::default(),
            });
        }
        if prepared.inputs.is_empty() {
            bail!("empty compaction input identity");
        }
        if prepared.inputs.len() > live_catalog_len {
            bail!("compaction inputs no longer fit catalog prefix");
        }
        let start = self
            .layers
            .windows(prepared.inputs.len())
            .take(live_catalog_len - prepared.inputs.len() + 1)
            .position(matches)
            .ok_or_else(|| anyhow!("compaction inputs no longer match live layers"))?;
        let mut layers = self.layers.clone();
        layers.splice(
            start..start + prepared.inputs.len(),
            [prepared.replacement.clone()],
        );
        Ok(Self {
            base: self.base.clone(),
            base_map: self.base_map.clone(),
            layers,
            n_docs,
            has_catalog_base: true,
            // A window merge below the base holds the invariant: while it was
            // additive no term-bearing row was ever hidden, so merging those
            // layers drops no row and the live term set is unchanged (#4246).
            distinct_terms: self.distinct_terms,
            query_cache: Arc::default(),
        })
    }

    pub(crate) fn immutable_base_reader(&self) -> Arc<SegmentReader> {
        self.base.clone()
    }

    pub(crate) fn replace_base_inputs(
        &self,
        base: &Arc<SegmentReader>,
        deltas: &[Arc<SegmentReader>],
        reader: Arc<SegmentReader>,
        ids: Vec<u32>,
    ) -> Result<Self> {
        let catalog_len = self
            .layers
            .iter()
            .take_while(|layer| !layer.private)
            .count();
        if !self.has_catalog_base
            || !Arc::ptr_eq(base, &self.base)
            || deltas.len() > catalog_len
            || !self
                .layers
                .iter()
                .zip(deltas)
                .all(|(layer, input)| Arc::ptr_eq(&layer.reader, input))
        {
            bail!("compacted base inputs no longer match live layers");
        }
        let mut coverage = self
            .base_map
            .as_ref()
            .map(|map| map.coverage.clone())
            .unwrap_or_else(|| (0..self.base.n_docs()).collect());
        for layer in &self.layers[..deltas.len()] {
            coverage |= &layer.coverage;
        }
        let mut replacement = Self::from_mapped_base(reader, ids)?;
        if replacement.base_map.as_ref().unwrap().coverage != coverage {
            bail!("compacted base row coverage differs from captured inputs");
        }
        replacement.layers = self.layers[deltas.len()..].to_vec();
        replacement.n_docs = self.n_docs.max(replacement.n_docs);
        replacement.distinct_terms = Self::refold_distinct_terms(
            &replacement.base,
            replacement.base_map.clone(),
            &replacement.layers,
            replacement.n_docs,
        );
        Ok(replacement)
    }

    pub(crate) fn delta_readers(&self) -> Vec<Arc<SegmentReader>> {
        self.layers
            .iter()
            .filter(|layer| !layer.private)
            .map(|layer| layer.reader.clone())
            .collect()
    }

    pub(crate) fn replace_delta_inputs(
        &self,
        inputs: &[Arc<SegmentReader>],
        reader: Arc<SegmentReader>,
        ids: Vec<u32>,
    ) -> Result<Self> {
        if inputs.is_empty() {
            bail!("empty compaction input identity");
        }
        let catalog_len = self
            .layers
            .iter()
            .take_while(|layer| !layer.private)
            .count();
        if !self.has_catalog_base || inputs.len() > catalog_len {
            bail!("compaction inputs include private layers or lack a catalog base");
        }
        let start = self
            .layers
            .windows(inputs.len())
            .take(catalog_len - inputs.len() + 1)
            .position(|window| {
                window
                    .iter()
                    .zip(inputs)
                    .all(|(layer, input)| Arc::ptr_eq(&layer.reader, input))
            })
            .ok_or_else(|| anyhow!("compaction inputs no longer match live layers"))?;
        self.replace_delta_range(start, inputs.len(), reader, ids)
    }

    pub(crate) fn from_base(base: Arc<SegmentReader>) -> Self {
        // A sealed base is written from a live projection, so every dictionary
        // entry it holds has at least one document: its dictionary size IS the
        // composed distinct-term count, read from the column header (#4246).
        let distinct_terms = base
            .has_text_postings()
            .then(|| base.keyword_ordinal_count().map(u64::from))
            .flatten();
        Self {
            n_docs: base.n_docs(),
            base,
            base_map: None,
            layers: Vec::new(),
            has_catalog_base: true,
            distinct_terms,
            query_cache: Arc::default(),
        }
    }

    pub(crate) fn with_delta(&self, reader: Arc<SegmentReader>, ids: Vec<u32>) -> Result<Self> {
        if !self.has_catalog_base || self.layers.iter().any(|layer| layer.private) {
            bail!("cannot append catalog delta without its published prefix");
        }
        self.can_append_incremental()?;
        let layer = Arc::new(DeltaLayer::new(reader, ids)?);
        let n_docs = layer.ids.iter().try_fold(self.n_docs, |n_docs, &global| {
            let candidate = global
                .checked_add(1)
                .ok_or_else(|| anyhow!("delta runtime ID overflow"))?;
            Ok::<u32, anyhow::Error>(n_docs.max(candidate))
        })?;
        let distinct_terms = self.additive_distinct_terms(&layer);
        let mut layers = self.layers.clone();
        layers.push(layer);
        Ok(Self {
            base: self.base.clone(),
            base_map: self.base_map.clone(),
            layers,
            n_docs,
            has_catalog_base: self.has_catalog_base,
            distinct_terms,
            query_cache: Arc::default(),
        })
    }

    /// The composed distinct-term count after appending `layer`, or `None` when
    /// this attach is not additive and the invariant on `distinct_terms` would
    /// break.
    ///
    /// The attach is additive only when no covered runtime ID already holds a
    /// Text value here: an insert can never empty an older posting, while an
    /// update or a delete can. Both loops are bounded by the layer — the
    /// covered IDs and the delta dictionary — never by the base dictionary, so
    /// the cost is one checkpoint interval of writes paid once at publication.
    fn additive_distinct_terms(&self, layer: &DeltaLayer) -> Option<u64> {
        let known = self.distinct_terms?;
        if layer.ids.iter().any(|&id| self.text_is_present(id)) {
            return None;
        }
        let added = layer.reader.keyword_ordinal_count()?;
        let mut fresh = 0u64;
        for ordinal in 0..added {
            let term = layer.reader.keyword_term_at_ordinal_cow(ordinal)?;
            if !self.has_dictionary_term(&term) {
                fresh += 1;
            }
        }
        known.checked_add(fresh)
    }

    /// Whether any composed layer's dictionary holds `term`. One binary search
    /// per layer and no posting decode.
    pub(crate) fn has_dictionary_term(&self, term: &str) -> bool {
        self.base.text_has_term(term)
            || self
                .layers
                .iter()
                .any(|layer| layer.reader.text_has_term(term))
    }

    /// The exact number of dictionary terms with a non-empty composed posting,
    /// when it is known without walking the dictionary. See `distinct_terms`.
    pub(crate) fn known_distinct_terms(&self) -> Option<u64> {
        self.distinct_terms
    }

    /// Re-derive `distinct_terms` after a compaction folded the base in
    /// (#4246). The merged base is a projection of composed postings
    /// (`segment_rdb::read_delta_values`), so its dictionary size IS its live
    /// term count; each remaining layer is then re-folded by the same additive
    /// rule `with_delta` applies, which re-checks the invariant per layer and
    /// yields `None` the moment a layer hides a term-bearing row. Cost is
    /// bounded by the remaining layers, never by the base — and a full merge
    /// is how a reader voided by an update gets its O(1) count back.
    fn refold_distinct_terms(
        base: &Arc<SegmentReader>,
        base_map: Option<Arc<DeltaLayer>>,
        layers: &[Arc<DeltaLayer>],
        n_docs: u32,
    ) -> Option<u64> {
        let mut view = Self {
            base: base.clone(),
            base_map,
            layers: Vec::with_capacity(layers.len()),
            n_docs,
            has_catalog_base: true,
            distinct_terms: base
                .has_text_postings()
                .then(|| base.keyword_ordinal_count().map(u64::from))
                .flatten(),
            query_cache: Arc::default(),
        };
        for layer in layers {
            view.distinct_terms = view.additive_distinct_terms(layer);
            view.distinct_terms?;
            view.layers.push(layer.clone());
        }
        view.distinct_terms
    }

    /// Source `0` is the base; source `k` is `layers[k - 1]` — the order
    /// [`StringTermCursor`] reports dictionary ordinals in.
    fn source_reader(&self, source: usize) -> &SegmentReader {
        match source.checked_sub(1) {
            None => self.base.as_ref(),
            Some(layer) => self.layers[layer].reader.as_ref(),
        }
    }

    /// Whether `source`'s local row `local` is still the live composed row:
    /// mapped to its runtime ID, not covered by a NEWER layer, and not in
    /// `dead` (the index's pending tombstones). `None` for a row map that does
    /// not know `local` — a torn layer, reported rather than counted.
    fn source_row_live(&self, source: usize, local: u32, dead: &RoaringBitmap) -> Option<bool> {
        let (global, newer) = match source.checked_sub(1) {
            None => {
                let global = match &self.base_map {
                    Some(map) => *map.ids.get(local as usize)?,
                    None => local,
                };
                (global, &self.layers[..])
            }
            Some(layer) => (
                *self.layers[layer].ids.get(local as usize)?,
                &self.layers[layer + 1..],
            ),
        };
        Some(!dead.contains(global) && !newer.iter().any(|layer| layer.coverage.contains(global)))
    }

    /// Whether the posting at `source`'s dictionary ordinal `dict_id` has a
    /// live composed docid, decoded only that far. `None` on a torn block or
    /// row map.
    fn source_posting_live(
        &self,
        source: usize,
        dict_id: u32,
        dead: &RoaringBitmap,
    ) -> Option<bool> {
        let mut torn = false;
        let any = self
            .source_reader(source)
            .text_posting_any_at(dict_id, |local| {
                self.source_row_live(source, local, dead)
                    .unwrap_or_else(|| {
                        torn = true;
                        true
                    })
            })?;
        (!torn).then_some(any)
    }

    /// Whether `token` still has a live composed document under `dead`
    /// (#4246). The newest layer holding it is probed first — an insert lands
    /// there and nothing can cover it — and each posting is decoded only as
    /// far as its first live docid; no posting is materialized or cached.
    /// `Some(false)` for an absent token, `None` for a torn dictionary,
    /// posting block, or row map.
    pub(crate) fn text_term_has_live_doc(&self, token: &str, dead: &RoaringBitmap) -> Option<bool> {
        note_text_term_probes(1);
        for source in (0..=self.layers.len()).rev() {
            let Some(dict_id) = self.source_reader(source).text_dict_id(token) else {
                continue;
            };
            if self.source_posting_live(source, dict_id, dead)? {
                return Some(true);
            }
        }
        Some(false)
    }

    /// The exact number of dictionary terms with a live composed document
    /// under `dead` — what `distinct_terms` carries when it is known, computed
    /// here for a reader with pending tombstones or a hidden row (#4246).
    ///
    /// ONE merged dictionary walk. Per term the cursor already names the
    /// layers holding it and their ordinals, so there is no re-lookup; the
    /// newest of them is probed first and each posting is decoded only as far
    /// as its first live docid. An insert-mostly corpus therefore costs one
    /// varint decode per term with no per-term allocation beyond the cursor,
    /// where the materializing walk it replaces composed and copied every
    /// posting (43 µs per term, 9 s at 200k documents). `None` on a torn
    /// dictionary, posting block, or row map; a partial count is never
    /// reported.
    pub(crate) fn live_text_term_count(&self, dead: &RoaringBitmap) -> Option<u64> {
        let mut cursor = self.string_terms(false).ok()?;
        let mut count = 0u64;
        loop {
            let Some((_term, ordinals)) = cursor.next_entry_cow(true).ok()? else {
                break;
            };
            note_text_term_probes(1);
            for &(source, dict_id) in ordinals.iter().rev() {
                if self.source_posting_live(source, dict_id, dead)? {
                    count += 1;
                    break;
                }
            }
        }
        Some(count)
    }

    /// Replace an exact, adjacent live delta window after its immutable
    /// compacted reader has been durably published. The replacement must cover
    /// precisely the same external IDs, including deleted rows, so later
    /// layers keep their precedence unchanged.
    pub(crate) fn replace_delta_range(
        &self,
        start: usize,
        count: usize,
        reader: Arc<SegmentReader>,
        ids: Vec<u32>,
    ) -> Result<Self> {
        let end = start
            .checked_add(count)
            .ok_or_else(|| anyhow!("delta replacement range overflow"))?;
        let catalog_len = self
            .layers
            .iter()
            .take_while(|layer| !layer.private)
            .count();
        if !self.has_catalog_base || count == 0 || end > catalog_len {
            bail!("delta replacement range is outside live layers");
        }
        let replacement = Arc::new(DeltaLayer::new(reader, ids)?);
        let mut coverage = RoaringBitmap::new();
        for layer in &self.layers[start..end] {
            coverage |= &layer.coverage;
        }
        if coverage != replacement.coverage {
            bail!("delta replacement coverage differs from live range");
        }
        let mut layers = self.layers.clone();
        layers.splice(start..end, [replacement]);
        Ok(Self {
            base: self.base.clone(),
            base_map: self.base_map.clone(),
            layers,
            n_docs: self.n_docs,
            has_catalog_base: self.has_catalog_base,
            // A compacted window can drop only a row a newer layer in the window
            // hid — and while the count was known no term-bearing row was ever
            // hidden, so the merge changes no live term. `None` stays `None`.
            distinct_terms: self.distinct_terms,
            query_cache: Arc::default(),
        })
    }
}

impl DeltaLayer {
    fn new(reader: Arc<SegmentReader>, ids: Vec<u32>) -> Result<Self> {
        if ids.len() != reader.n_docs() as usize {
            bail!("delta local ID count does not match segment");
        }
        let mut local_by_global = BTreeMap::new();
        let mut coverage = RoaringBitmap::new();
        for (local, &global) in ids.iter().enumerate() {
            if local_by_global.insert(global, local as u32).is_some() {
                bail!("duplicate delta runtime ID");
            }
            coverage.insert(global);
        }
        Ok(Self {
            reader,
            ids,
            local_by_global,
            coverage,
            private: false,
        })
    }
}

impl ComposedSegmentReader {
    /// A compacted field may use its own dense local row space. The map
    /// keeps it independent of collection ID allocation and other fields.
    pub(crate) fn from_mapped_base(reader: Arc<SegmentReader>, ids: Vec<u32>) -> Result<Self> {
        let base_terms = reader
            .has_text_postings()
            .then(|| reader.keyword_ordinal_count().map(u64::from))
            .flatten();
        let mut view = Self::from_base(reader.clone()).with_delta(reader, ids)?;
        view.base_map = view.layers.pop();
        // The mapping layer IS the base reader: the dictionary is unchanged, so
        // the base's own term count still holds.
        view.distinct_terms = base_terms;
        Ok(view)
    }

    fn dense_base_only(&self) -> bool {
        self.base_map.is_none() && self.layers.is_empty()
    }

    pub(crate) fn base_reader(&self) -> Option<&Arc<SegmentReader>> {
        self.dense_base_only().then_some(&self.base)
    }
    pub(crate) fn n_docs(&self) -> u32 {
        self.n_docs
    }
    /// Sparse IDs covered by the newest appended delta, including deletions.
    pub(crate) fn covered_ids(&self) -> &[u32] {
        self.layers.last().map_or(&[], |layer| layer.ids.as_slice())
    }
    pub(crate) fn applied_seq(&self) -> u64 {
        self.layers.last().map_or_else(
            || self.base.applied_seq(),
            |layer| layer.reader.applied_seq(),
        )
    }
    fn winner(&self, id: u32) -> (&SegmentReader, u32) {
        for layer in self.layers.iter().rev() {
            if let Some(&local) = layer.local_by_global.get(&id) {
                return (&layer.reader, local);
            }
        }
        let local = self.base_map.as_ref().map_or(id, |map| {
            map.local_by_global.get(&id).copied().unwrap_or(u32::MAX)
        });
        (&self.base, local)
    }

    /// After checkpoint publication, private winners are precisely the layers
    /// applied after its cut. They already hide the older catalog row.
    pub(crate) fn has_private_winner(&self, id: u32) -> bool {
        self.layers
            .iter()
            .rev()
            .find(|layer| layer.coverage.contains(id))
            .is_some_and(|layer| layer.private)
    }
    pub(crate) fn vector_at(&self, id: u32, dim: usize) -> Option<&[f32]> {
        let (reader, row) = self.winner(id);
        reader.vector_at(row, dim)
    }
    pub(crate) fn keyword_at(&self, id: u32) -> Option<String> {
        let (r, i) = self.winner(id);
        r.keyword_at(i)
    }
    /// Internal scalar projection view. Raw dictionary values borrow their
    /// source mmap; legacy LZ4 values carry the selected owned fallback.
    pub(crate) fn keyword_at_cow(&self, id: u32) -> Option<Cow<'_, str>> {
        let (reader, local) = self.winner(id);
        reader.keyword_at_cow(local)
    }
    pub(crate) fn number_at(&self, id: u32) -> Option<f64> {
        let (r, i) = self.winner(id);
        r.number_at(i)
    }
    pub(crate) fn set_at(&self, id: u32) -> Option<Vec<String>> {
        let (r, i) = self.winner(id);
        r.set_at(i)
    }
    /// `(present, count)` without constructing a row of owned strings.
    pub(crate) fn set_row_member_count(&self, id: u32) -> Option<(bool, u32)> {
        let (reader, local) = self.winner(id);
        reader.set_row_member_count(local)
    }
    /// Internal scalar projection member view. The caller must consume this
    /// Cow before advancing to the next member.
    pub(crate) fn set_member_at_cow(&self, id: u32, member: u32) -> Option<Cow<'_, str>> {
        let (reader, local) = self.winner(id);
        reader.set_member_at_cow(local, member)
    }
    pub(crate) fn hash_at(&self, id: u32) -> Option<u64> {
        let (r, i) = self.winner(id);
        r.hash_at(i)
    }
    pub(crate) fn text_is_present(&self, id: u32) -> bool {
        let (r, i) = self.winner(id);
        r.text_is_present(i)
    }
    pub(crate) fn text_doc_len(&self, id: u32) -> u32 {
        let (r, i) = self.winner(id);
        r.text_doc_len(i)
    }
    /// Every row's text doc length by global id, for hot BM25 walks that
    /// would otherwise resolve `winner(id)` (one `BTreeMap` probe per layer)
    /// for every scored docid. A bare dense base borrows its column; any
    /// other composition materializes the winners once per composition into
    /// `query_cache` (O(n_docs + Σ layer rows), no per-row map probe) and
    /// returns exactly what `text_doc_len(id)` returns for every
    /// `id < n_docs()` (#4246).
    pub(crate) fn text_doc_lens(&self) -> Option<&[u32]> {
        if let Some(base) = self.base_reader() {
            return base.text_doc_lens();
        }
        Some(
            self.query_cache
                .text_doc_lens
                .get_or_init(|| self.materialize_text_doc_lens()),
        )
    }

    fn materialize_text_doc_lens(&self) -> Vec<u32> {
        let mut lens = vec![0u32; self.n_docs as usize];
        match &self.base_map {
            None => {
                if let Some(column) = self.base.text_doc_lens() {
                    let n = column.len().min(lens.len());
                    lens[..n].copy_from_slice(&column[..n]);
                }
            }
            Some(map) => {
                for (local, &global) in map.ids.iter().enumerate() {
                    if let Some(slot) = lens.get_mut(global as usize) {
                        *slot = self.base.text_doc_len(local as u32);
                    }
                }
            }
        }
        // Applied in order so the newest layer holding a row wins, exactly
        // as `winner` walks the layers newest-first.
        for layer in &self.layers {
            for (local, &global) in layer.ids.iter().enumerate() {
                if let Some(slot) = lens.get_mut(global as usize) {
                    *slot = layer.reader.text_doc_len(local as u32);
                }
            }
        }
        lens
    }

    fn postings(
        &self,
        read: impl Fn(&SegmentReader) -> Option<RoaringBitmap>,
    ) -> Option<RoaringBitmap> {
        let local = read(&self.base).unwrap_or_default();
        let mut out = if let Some(map) = &self.base_map {
            let mut global = RoaringBitmap::new();
            for id in local {
                global.insert(*map.ids.get(id as usize)?);
            }
            global
        } else {
            local
        };
        for layer in &self.layers {
            out -= &layer.coverage;
            if let Some(local) = read(&layer.reader) {
                for id in local {
                    out.insert(*layer.ids.get(id as usize)?);
                }
            }
        }
        Some(out)
    }
    pub(crate) fn keyword_postings(&self, value: &str) -> Option<RoaringBitmap> {
        if self.dense_base_only() {
            return self.base.keyword_postings(value);
        }
        self.postings(|reader| reader.keyword_postings(value))
            .filter(|p| !p.is_empty())
    }
    pub(crate) fn keyword_df(&self, value: &str) -> Option<u64> {
        if self.dense_base_only() {
            return self.base.keyword_df(value);
        }
        self.keyword_postings(value).map(|p| p.len())
    }
    pub(crate) fn set_postings(&self, value: &str) -> Option<RoaringBitmap> {
        if self.dense_base_only() {
            return self.base.set_postings(value);
        }
        self.postings(|reader| reader.set_postings(value))
            .filter(|p| !p.is_empty())
    }
    pub(crate) fn set_df(&self, value: &str) -> Option<u64> {
        if self.dense_base_only() {
            return self.base.set_df(value);
        }
        self.set_postings(value).map(|p| p.len())
    }
    pub(crate) fn number_value_postings(&self, bits: u64) -> Option<RoaringBitmap> {
        if self.dense_base_only() {
            return self.base.number_value_postings(bits);
        }
        self.postings(|reader| reader.number_value_postings(bits))
            .filter(|p| !p.is_empty())
    }
    pub(crate) fn number_value_df(&self, bits: u64) -> Option<u64> {
        if self.dense_base_only() {
            return self.base.number_value_df(bits);
        }
        self.number_value_postings(bits).map(|p| p.len())
    }
    pub(crate) fn number_range(
        &self,
        low: Option<(u64, bool)>,
        high: Option<(u64, bool)>,
    ) -> Option<RoaringBitmap> {
        if self.dense_base_only() {
            return self.base.number_range(low, high);
        }
        self.postings(|reader| reader.number_range(low, high))
    }
    pub(crate) fn number_range_df(
        &self,
        low: Option<(u64, bool)>,
        high: Option<(u64, bool)>,
    ) -> Option<u64> {
        if self.dense_base_only() {
            return self.base.number_range_df(low, high);
        }
        self.number_range(low, high).map(|p| p.len())
    }
    pub(crate) fn number_range_distinct_count(
        &self,
        low: Option<(u64, bool)>,
        high: Option<(u64, bool)>,
    ) -> Option<u64> {
        if self.dense_base_only() {
            return self.base.number_range_distinct_count(low, high);
        }
        let mut keys = self.number_keys(low, high, false).ok()?;
        let mut count = 0;
        while let Some(key) = keys.next().ok()? {
            if self
                .number_value_postings(key)
                .is_some_and(|p| !p.is_empty())
            {
                count += 1;
            }
        }
        Some(count)
    }
    pub(crate) fn text_postings_arc(&self, token: &str) -> Option<Arc<(Vec<u32>, Vec<u32>)>> {
        note_text_term_probes(1);
        if self.dense_base_only() {
            return self.base.text_postings_arc(token);
        }
        if let Some(hit) = self.query_cache.text_postings.get(token) {
            return Some(hit);
        }
        let merged = self.merge_text_postings(token)?;
        self.query_cache
            .text_postings
            .insert(token.to_owned(), merged.clone());
        Some(merged)
    }

    /// The layered `(docids, tfs)` merge behind [`Self::text_postings_arc`]
    /// for a composition that is not a bare dense base. Pure in the
    /// composition, so `text_postings_arc` caches its result per token in
    /// `query_cache`: a cold 20-token ngram AND against a 500k-row base with
    /// live delta layers otherwise re-decoded and re-merged 20 × 500k rows
    /// on every request (#4246).
    fn merge_text_postings(&self, token: &str) -> Option<CachedTextPosting> {
        // Byte-identical to `merge_text_postings_reference` (the
        // `#[cfg(test)]` original), which paid one O(df) `retain` sweep per
        // layer per token. Here the base posting is filtered ONCE by the
        // union of every layer's coverage, and each layer's posting by the
        // union of the coverage newer than it; a torn map still yields `None`
        // and every reader is probed exactly once per token.
        let unions = self.coverage_unions();
        let (mut base_ids, mut base_tfs) = (Vec::new(), Vec::new());
        if let Some(base) = self.base.text_postings_arc(token) {
            if let Some(map) = &self.base_map {
                let mut mapped = Vec::with_capacity(base.0.len());
                for (&local, &tf) in base.0.iter().zip(&base.1) {
                    let id = *map.ids.get(local as usize)?;
                    if !unions.all.contains(id) {
                        mapped.push((id, tf));
                    }
                }
                if !unions.base_map_ascending {
                    mapped.sort_unstable_by_key(|&(id, _)| id);
                }
                base_ids.reserve_exact(mapped.len());
                base_tfs.reserve_exact(mapped.len());
                for (id, tf) in mapped {
                    base_ids.push(id);
                    base_tfs.push(tf);
                }
            } else {
                retain_uncovered_sorted(
                    &base.0,
                    &base.1,
                    &unions.all,
                    &mut base_ids,
                    &mut base_tfs,
                );
            }
        }
        let mut newer = Vec::new();
        for (index, layer) in self.layers.iter().enumerate() {
            if let Some(posting) = layer.reader.text_postings_arc(token) {
                let hidden = &unions.newer_than[index];
                for (&local, &tf) in posting.0.iter().zip(&posting.1) {
                    let id = *layer.ids.get(local as usize)?;
                    if !hidden.contains(id) {
                        newer.push((id, tf));
                    }
                }
            }
        }
        if newer.is_empty() {
            if base_ids.is_empty() {
                return None;
            }
            return Some(Arc::new((base_ids, base_tfs)));
        }
        // Every surviving id is unique: a layer row survives only from the
        // newest layer covering it, and no surviving base row is covered.
        newer.sort_unstable_by_key(|&(id, _)| id);
        let total = base_ids.len() + newer.len();
        let (mut ids, mut tfs) = (Vec::with_capacity(total), Vec::with_capacity(total));
        let (mut b, mut n) = (0, 0);
        while b < base_ids.len() && n < newer.len() {
            if base_ids[b] < newer[n].0 {
                ids.push(base_ids[b]);
                tfs.push(base_tfs[b]);
                b += 1;
            } else {
                ids.push(newer[n].0);
                tfs.push(newer[n].1);
                n += 1;
            }
        }
        ids.extend_from_slice(&base_ids[b..]);
        tfs.extend_from_slice(&base_tfs[b..]);
        for &(id, tf) in &newer[n..] {
            ids.push(id);
            tfs.push(tf);
        }
        Some(Arc::new((ids, tfs)))
    }

    fn coverage_unions(&self) -> &CoverageUnions {
        self.query_cache.coverage_unions.get_or_init(|| {
            let mut newer_than = vec![RoaringBitmap::new(); self.layers.len()];
            let mut acc = RoaringBitmap::new();
            for (index, layer) in self.layers.iter().enumerate().rev() {
                newer_than[index] = acc.clone();
                acc |= &layer.coverage;
            }
            let base_map_ascending = self
                .base_map
                .as_ref()
                .is_none_or(|map| map.ids.windows(2).all(|w| w[0] < w[1]));
            CoverageUnions {
                all: acc,
                newer_than,
                base_map_ascending,
            }
        })
    }

    /// The original layered merge, kept as the oracle for the rewrite above.
    #[cfg(test)]
    fn merge_text_postings_reference(&self, token: &str) -> Option<CachedTextPosting> {
        // Decode one token at a time. Each pass holds two sorted postings
        // and removes covered older rows before merging the newer values.
        let mut out = Vec::new();
        if let Some(base) = self.base.text_postings_arc(token) {
            for (&local, &tf) in base.0.iter().zip(&base.1) {
                let id = if let Some(map) = &self.base_map {
                    *map.ids.get(local as usize)?
                } else {
                    local
                };
                out.push((id, tf));
            }
            if self.base_map.is_some() {
                out.sort_unstable_by_key(|&(id, _)| id);
            }
        }
        for layer in &self.layers {
            out.retain(|&(id, _)| !layer.coverage.contains(id));
            if let Some(posting) = layer.reader.text_postings_arc(token) {
                let mut incoming = Vec::with_capacity(posting.0.len());
                for (&local, &tf) in posting.0.iter().zip(&posting.1) {
                    incoming.push((*layer.ids.get(local as usize)?, tf));
                }
                incoming.sort_unstable_by_key(|&(id, _)| id);
                let mut merged = Vec::with_capacity(out.len() + incoming.len());
                let mut older = out.into_iter().peekable();
                let mut newer = incoming.into_iter().peekable();
                while let (Some(left), Some(right)) = (older.peek(), newer.peek()) {
                    if left.0 < right.0 {
                        merged.push(older.next()?);
                    } else {
                        merged.push(newer.next()?);
                    }
                }
                merged.extend(older);
                merged.extend(newer);
                out = merged;
            }
        }
        if out.is_empty() {
            return None;
        }
        let (ids, tfs) = out.into_iter().unzip();
        Some(Arc::new((ids, tfs)))
    }
    pub(crate) fn text_token_df(&self, token: &str) -> usize {
        if self.dense_base_only() {
            return self.base.text_token_df(token);
        }
        self.text_postings_arc(token).map_or(0, |p| p.0.len())
    }

    /// `token`'s posting for a small candidate set (#4246). A posting that is
    /// already resident — the composition's per-query cache, or the dense
    /// base's bounded posting cache — comes back whole as
    /// [`TextPostingAt::Cached`]. A cold posting is never materialized:
    /// the base is streamed through [`SegmentReader::text_posting_scan`] and
    /// each layer's (small) posting is walked, yielding the exact composed
    /// df and the tf of every id in `candidates` (ascending, distinct) as
    /// [`TextPostingAt::Sparse`]. Ids `hidden` returns true for count toward
    /// neither — the caller folds its own overlays (tombstones, live and
    /// staged rows) through it. Composition semantics are those of
    /// [`Self::text_postings_arc`]: the newest layer covering an id owns its
    /// row, an uncovered id falls back to the mapped base, and a torn map
    /// yields `None`. For a dense base `None` also means the dictionary lacks
    /// the token, so a `None` can always be re-resolved through
    /// `text_postings_arc` cheaply.
    pub(crate) fn text_posting_at(
        &self,
        token: &str,
        candidates: &[u32],
        mut hidden: impl FnMut(u32) -> bool,
    ) -> Option<TextPostingAt> {
        note_text_term_probes(1);
        debug_assert!(candidates.windows(2).all(|w| w[0] < w[1]));
        if self.dense_base_only() {
            if let Some(hit) = self.base.text_posting_cached(token) {
                return Some(TextPostingAt::Cached(hit));
            }
            let mut df = 0usize;
            let mut hits = Vec::new();
            let mut wanted = SortedIdCursor::new(candidates);
            self.base.text_posting_scan(token, |id, tf| {
                if hidden(id) {
                    return;
                }
                df += 1;
                if wanted.contains(id) {
                    hits.push((id, tf));
                }
            })?;
            return Some(TextPostingAt::Sparse { df, hits });
        }
        if let Some(hit) = self.query_cache.text_postings.get(token) {
            return Some(TextPostingAt::Cached(hit));
        }
        let unions = self.coverage_unions();
        let mut df = 0usize;
        let mut hits = Vec::new();
        let mut torn = false;
        {
            let map = self.base_map.as_deref();
            let mut wanted = SortedIdCursor::new(candidates);
            let scanned = self.base.text_posting_scan(token, |local, tf| {
                let id = match map {
                    Some(map) => match map.ids.get(local as usize) {
                        Some(&id) => id,
                        None => {
                            torn = true;
                            return;
                        }
                    },
                    None => local,
                };
                if unions.all.contains(id) || hidden(id) {
                    return;
                }
                df += 1;
                if wanted.contains(id) {
                    hits.push((id, tf));
                }
            });
            if scanned.is_none() {
                // A torn or absent base posting contributes nothing, exactly
                // as `merge_text_postings` treats a `None` base read.
                df = 0;
                hits.clear();
            }
        }
        if torn {
            return None;
        }
        for (index, layer) in self.layers.iter().enumerate() {
            let Some(posting) = layer.reader.text_postings_arc(token) else {
                continue;
            };
            let newer = &unions.newer_than[index];
            for (&local, &tf) in posting.0.iter().zip(&posting.1) {
                let id = *layer.ids.get(local as usize)?;
                if newer.contains(id) || hidden(id) {
                    continue;
                }
                df += 1;
                if candidates.binary_search(&id).is_ok() {
                    hits.push((id, tf));
                }
            }
        }
        // Every surviving id is unique (see `merge_text_postings`), so a
        // sort is all the merge needs.
        hits.sort_unstable_by_key(|&(id, _)| id);
        Some(TextPostingAt::Sparse { df, hits })
    }
    pub(crate) fn string_terms(&self, descending: bool) -> Result<StringTermCursor<'_>> {
        StringTermCursor::new(self.readers(), descending)
    }
    pub(crate) fn number_keys(
        &self,
        low: Option<(u64, bool)>,
        high: Option<(u64, bool)>,
        descending: bool,
    ) -> Result<NumberKeyCursor<'_>> {
        NumberKeyCursor::new(self.readers(), low, high, descending)
    }
    fn readers(&self) -> Vec<&SegmentReader> {
        std::iter::once(self.base.as_ref())
            .chain(self.layers.iter().map(|layer| layer.reader.as_ref()))
            .collect()
    }
    pub(crate) fn keyword_terms_all(&self) -> Option<Vec<(String, RoaringBitmap)>> {
        if self.dense_base_only() {
            return self.base.keyword_terms_all();
        }
        let mut cursor = self.string_terms(false).ok()?;
        let mut out = Vec::new();
        while let Some((term, ordinals)) = cursor.next_with_ordinals().ok()? {
            let mut matches = ordinals.into_iter().peekable();
            let local = if matches.peek().is_some_and(|(source, _)| *source == 0) {
                self.base.keyword_postings_at_ordinal(matches.next()?.1)?
            } else {
                RoaringBitmap::new()
            };
            let mut posting = if let Some(map) = &self.base_map {
                let mut global = RoaringBitmap::new();
                for id in local {
                    global.insert(*map.ids.get(id as usize)?);
                }
                global
            } else {
                local
            };
            for (index, layer) in self.layers.iter().enumerate() {
                // Even a layer without this term masks all rows it replaced.
                posting -= &layer.coverage;
                if matches
                    .peek()
                    .is_some_and(|(source, _)| *source == index + 1)
                {
                    let local = layer
                        .reader
                        .keyword_postings_at_ordinal(matches.next()?.1)?;
                    for id in local {
                        posting.insert(*layer.ids.get(id as usize)?);
                    }
                }
            }
            if !posting.is_empty() {
                out.push((term, posting));
            }
        }
        Some(out)
    }
    pub(crate) fn set_elements_all(&self) -> Option<Vec<(String, RoaringBitmap)>> {
        if self.dense_base_only() {
            return self.base.set_elements_all();
        }
        let mut cursor = self.string_terms(false).ok()?;
        let mut out = Vec::new();
        while let Some(term) = cursor.next().ok()? {
            if let Some(posting) = self.set_postings(&term) {
                out.push((term, posting));
            }
        }
        Some(out)
    }
    pub(crate) fn number_values_all(&self) -> Option<Vec<(u64, RoaringBitmap)>> {
        if self.dense_base_only() {
            return self.base.number_values_all();
        }
        let mut cursor = self.number_keys(None, None, false).ok()?;
        let mut out = Vec::new();
        while let Some(key) = cursor.next().ok()? {
            if let Some(posting) = self.number_value_postings(key) {
                out.push((key, posting));
            }
        }
        Some(out)
    }
    pub(crate) fn text_tokens_all(&self) -> Option<Vec<(String, Vec<u32>, Vec<u32>)>> {
        if self.dense_base_only() {
            let all = self.base.text_tokens_all()?;
            note_text_posting_clones(all.len() as u64);
            return Some(all);
        }
        let mut cursor = self.string_terms(false).ok()?;
        let mut out = Vec::new();
        while let Some(term) = cursor.next().ok()? {
            if let Some(posting) = self.text_postings_arc(&term) {
                note_text_posting_clones(1);
                out.push((term, posting.0.clone(), posting.1.clone()));
            }
        }
        Some(out)
    }
}

struct StringSource<'a> {
    reader: &'a SegmentReader,
    next_ordinal: Option<u32>,
    count: u32,
    head: Option<Cow<'a, str>>,
}
pub(crate) struct StringTermCursor<'a> {
    sources: Vec<StringSource<'a>>,
    descending: bool,
}
impl<'a> StringTermCursor<'a> {
    fn new(readers: Vec<&'a SegmentReader>, descending: bool) -> Result<Self> {
        let mut sources = Vec::new();
        for reader in readers {
            let count = reader
                .keyword_ordinal_count()
                .ok_or_else(|| anyhow!("missing string dictionary"))?;
            let ordinal = if descending {
                count.checked_sub(1)
            } else {
                (count > 0).then_some(0)
            };
            let head = ordinal
                .map(|i| {
                    reader
                        .keyword_term_at_ordinal_cow(i)
                        .ok_or_else(|| anyhow!("invalid dictionary ordinal"))
                })
                .transpose()?;
            sources.push(StringSource {
                reader,
                next_ordinal: ordinal,
                count,
                head,
            });
        }
        Ok(Self {
            sources,
            descending,
        })
    }
    pub(crate) fn next(&mut self) -> Result<Option<String>> {
        Ok(self.next_cow()?.map(Cow::into_owned))
    }
    /// Advance one dictionary term without copying raw mmap bytes.
    pub(crate) fn next_cow(&mut self) -> Result<Option<Cow<'a, str>>> {
        Ok(self.next_entry_cow(false)?.map(|(term, _)| term))
    }
    /// The selected dictionary ordinal is already known for each matching
    /// layer. Preserve it before advancing so callers need no point lookup.
    fn next_with_ordinals(&mut self) -> Result<Option<(String, Vec<(usize, u32)>)>> {
        Ok(self
            .next_entry_cow(true)?
            .map(|(term, ordinals)| (term.into_owned(), ordinals)))
    }
    fn next_entry_cow(
        &mut self,
        include_ordinals: bool,
    ) -> Result<Option<(Cow<'a, str>, Vec<(usize, u32)>)>> {
        let mut selected: Option<usize> = None;
        for (index, source) in self.sources.iter().enumerate() {
            let Some(head) = source.head.as_ref() else {
                continue;
            };
            let replace = match selected {
                None => true,
                Some(old) => {
                    let old_head = self.sources[old].head.as_ref().expect("selected head");
                    if self.descending {
                        head > old_head
                    } else {
                        head < old_head
                    }
                }
            };
            if replace {
                selected = Some(index);
            }
        }
        let Some(selected) = selected else {
            return Ok(None);
        };
        // Move the selected head. This transfers an LZ4 owned fallback and
        // keeps raw mmap bytes borrowed. No selected head is cloned.
        let term = self.sources[selected].head.take().expect("selected head");
        let mut ordinals = Vec::new();
        for (source_index, source) in self.sources.iter_mut().enumerate() {
            if source_index == selected
                || source
                    .head
                    .as_ref()
                    .is_some_and(|head| head.as_ref() == term.as_ref())
            {
                if include_ordinals {
                    ordinals.push((
                        source_index,
                        source.next_ordinal.expect("a head has an ordinal"),
                    ));
                }
                source.next_ordinal = source.next_ordinal.and_then(|i| {
                    if self.descending {
                        i.checked_sub(1)
                    } else {
                        i.checked_add(1).filter(|&j| j < source.count)
                    }
                });
                source.head = source
                    .next_ordinal
                    .map(|i| {
                        source
                            .reader
                            .keyword_term_at_ordinal_cow(i)
                            .ok_or_else(|| anyhow!("invalid dictionary ordinal"))
                    })
                    .transpose()?;
            }
        }
        Ok(Some((term, ordinals)))
    }
}

struct NumberSource<'a> {
    reader: &'a SegmentReader,
    next_ordinal: Option<u32>,
    low: u32,
    high: u32,
    head: Option<u64>,
}
pub(crate) struct NumberKeyCursor<'a> {
    sources: Vec<NumberSource<'a>>,
    descending: bool,
}
impl<'a> NumberKeyCursor<'a> {
    fn new(
        readers: Vec<&'a SegmentReader>,
        low: Option<(u64, bool)>,
        high: Option<(u64, bool)>,
        descending: bool,
    ) -> Result<Self> {
        let mut sources = Vec::new();
        for reader in readers {
            let (first, end) = reader
                .number_range_index_window(low, high)
                .ok_or_else(|| anyhow!("missing numeric dictionary"))?;
            let ordinal = if first >= end {
                None
            } else if descending {
                Some(end - 1)
            } else {
                Some(first)
            };
            let head = ordinal
                .map(|i| {
                    reader
                        .number_sorted_bits_at(i)
                        .ok_or_else(|| anyhow!("invalid numeric ordinal"))
                })
                .transpose()?;
            sources.push(NumberSource {
                reader,
                next_ordinal: ordinal,
                low: first,
                high: end,
                head,
            });
        }
        Ok(Self {
            sources,
            descending,
        })
    }
    pub(crate) fn next(&mut self) -> Result<Option<u64>> {
        let heads = self.sources.iter().filter_map(|source| source.head);
        let selected = if self.descending {
            heads.max()
        } else {
            heads.min()
        };
        let Some(key) = selected else {
            return Ok(None);
        };
        for source in &mut self.sources {
            if source.head == Some(key) {
                source.next_ordinal = source.next_ordinal.and_then(|i| {
                    if self.descending {
                        i.checked_sub(1).filter(|&j| j >= source.low)
                    } else {
                        i.checked_add(1).filter(|&j| j < source.high)
                    }
                });
                source.head = source
                    .next_ordinal
                    .map(|i| {
                        source
                            .reader
                            .number_sorted_bits_at(i)
                            .ok_or_else(|| anyhow!("invalid numeric ordinal"))
                    })
                    .transpose()?;
            }
        }
        Ok(Some(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::{
        write_hash_segment, write_keyword_segment, write_number_segment, write_set_segment,
        write_text_segment,
    };
    use crate::storage::Postings;
    use std::path::Path;

    fn keyword(path: &Path, values: &[Option<&str>]) -> Arc<SegmentReader> {
        let mut postings = BTreeMap::<String, RoaringBitmap>::new();
        for (id, value) in values.iter().enumerate() {
            if let Some(value) = value {
                postings
                    .entry((*value).to_owned())
                    .or_default()
                    .insert(id as u32);
            }
        }
        write_keyword_segment(path, 7, values, &postings).unwrap();
        Arc::new(SegmentReader::open(path).unwrap())
    }
    fn text(path: &Path, rows: &[Option<&[(&str, u32)]>]) -> Arc<SegmentReader> {
        let mut postings = BTreeMap::<String, Postings>::new();
        let mut lens = Vec::new();
        for (id, row) in rows.iter().enumerate() {
            let mut len = 0;
            for &(term, tf) in row.unwrap_or_default() {
                postings
                    .entry(term.to_owned())
                    .or_default()
                    .upsert(id as u32, tf);
                len += tf;
            }
            lens.push(len);
        }
        let present: Vec<_> = rows.iter().map(Option::is_some).collect();
        write_text_segment(
            path,
            7,
            &postings,
            &lens,
            &present,
            present.iter().filter(|&&v| v).count() as u64,
            lens.iter().map(|&v| v as u64).sum(),
        )
        .unwrap();
        Arc::new(SegmentReader::open(path).unwrap())
    }
    fn bits(value: f64) -> u64 {
        let bits = value.to_bits();
        if bits >> 63 == 1 {
            !bits
        } else {
            bits ^ (1 << 63)
        }
    }

    #[test]
    fn compaction_private_rows_keep_deletes_and_sorted_external_ids() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword(&dir.path().join("base"), &[Some("z-old"), Some("a-value")]);
        let delta = keyword(&dir.path().join("delta"), &[Some("m-value"), None]);
        let (view, ids) = compose_checkpoint_layers(vec![
            (base, vec!["z".into(), "a".into()]),
            (delta, vec!["m".into(), "z".into()]),
        ])
        .unwrap();
        assert_eq!(ids, ["a", "m", "z"]);
        assert_eq!(view.n_docs(), 3);
        assert_eq!(view.keyword_at(0).as_deref(), Some("a-value"));
        assert_eq!(view.keyword_at(1).as_deref(), Some("m-value"));
        assert_eq!(
            view.keyword_at(2),
            None,
            "partial compaction must preserve the deletion"
        );
        assert!(view.keyword_postings("z-old").is_none());
        let out = dir.path().join("merged");
        crate::segment::stream::write_keyword_stream(&out, 9, &view).unwrap();
        let reader = SegmentReader::open(&out).unwrap();
        assert_eq!(reader.n_docs(), 3);
        assert_eq!(reader.keyword_at(0).as_deref(), Some("a-value"));
        assert_eq!(reader.keyword_at(1).as_deref(), Some("m-value"));
        assert_eq!(reader.keyword_at(2), None);
    }

    #[test]
    fn compaction_private_rows_reject_invalid_input_maps() {
        let dir = tempfile::tempdir().unwrap();
        let reader = keyword(&dir.path().join("base"), &[Some("one"), Some("two")]);
        assert!(compose_checkpoint_layers(vec![]).is_err());
        assert!(compose_checkpoint_layers(vec![(reader.clone(), vec!["a".into()])]).is_err());
        let error =
            compose_checkpoint_layers(vec![(reader, vec!["a".into(), "a".into()])]).unwrap_err();
        assert!(error.to_string().contains("duplicate external ID"));
    }

    #[test]
    fn mapped_keyword_base_does_not_leak_local_row_ids() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword(&dir.path().join("base"), &[Some("a"), None, Some("b")]);
        let view = ComposedSegmentReader::from_mapped_base(base, vec![90, 10, 20]).unwrap();
        assert_eq!(view.n_docs(), 91);
        assert_eq!(view.keyword_at(0), None);
        assert_eq!(view.keyword_at(10), None);
        assert_eq!(view.keyword_at(90).as_deref(), Some("a"));
        assert_eq!(view.keyword_postings("b"), Some([20].into_iter().collect()));
        assert_eq!(view.keyword_df("a"), Some(1));
        assert!(view.base_reader().is_none());
        let newer = keyword(&dir.path().join("newer"), &[None, Some("c")]);
        let view = view.with_delta(newer, vec![90, 10]).unwrap();
        assert_eq!(view.keyword_postings("a"), None);
        assert_eq!(view.keyword_at(10).as_deref(), Some("c"));
        assert_eq!(view.keyword_postings("b"), Some([20].into_iter().collect()));
    }
    #[test]
    fn mapped_text_base_sorts_ids_and_keeps_tfs_attached() {
        let dir = tempfile::tempdir().unwrap();
        let base = text(
            &dir.path().join("base"),
            &[Some(&[("same", 7)]), Some(&[("same", 3)]), None],
        );
        let view = ComposedSegmentReader::from_mapped_base(base, vec![90, 10, 20]).unwrap();
        assert_eq!(
            *view.text_postings_arc("same").unwrap(),
            (vec![10, 90], vec![3, 7])
        );
        assert_eq!(view.text_doc_len(90), 7);
        assert!(
            Arc::ptr_eq(
                &view.text_postings_arc("same").unwrap(),
                &view.text_postings_arc("same").unwrap()
            ),
            "a repeated token lookup must be served from the per-composition cache"
        );
        assert!(!view.text_is_present(0));
        assert!(!view.text_is_present(20));
        let lens = view.text_doc_lens().unwrap();
        assert_eq!(lens.len(), view.n_docs() as usize);
        for id in 0..view.n_docs() {
            assert_eq!(lens[id as usize], view.text_doc_len(id), "id {id}");
        }
        let newer = text(&dir.path().join("newer"), &[Some(&[("same", 11)]), None]);
        let view = view.with_delta(newer, vec![20, 90]).unwrap();
        assert_eq!(
            *view.text_postings_arc("same").unwrap(),
            (vec![10, 20], vec![3, 11])
        );
        assert_eq!(view.text_token_df("same"), 2);
    }
    #[test]
    fn mapped_numeric_base_translates_ranges_and_materialized_values() {
        let dir = tempfile::tempdir().unwrap();
        let bp = dir.path().join("base");
        write_number_segment(&bp, 1, &[Some(3.0), Some(-2.0), None]).unwrap();
        let view = ComposedSegmentReader::from_mapped_base(
            Arc::new(SegmentReader::open(&bp).unwrap()),
            vec![90, 10, 20],
        )
        .unwrap();
        assert_eq!(view.number_at(0), None);
        assert_eq!(view.number_at(90), Some(3.0));
        assert_eq!(
            view.number_range(None, None),
            Some([10, 90].into_iter().collect())
        );
        assert_eq!(
            view.number_range(Some((bits(0.0), true)), None),
            Some([90].into_iter().collect())
        );
        assert_eq!(view.number_range_df(None, None), Some(2));
        assert_eq!(view.number_range_distinct_count(None, None), Some(2));
        assert_eq!(
            view.number_values_all().unwrap(),
            vec![
                (bits(-2.0), [10].into_iter().collect()),
                (bits(3.0), [90].into_iter().collect())
            ]
        );
    }
    #[test]
    fn sparse_keyword_deltas_hide_older_values_and_translate_postings() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword(&dir.path().join("base"), &[Some("old"), Some("same")]);
        let one = keyword(&dir.path().join("one"), &[Some("new"), Some("same"), None]);
        let two = keyword(&dir.path().join("two"), &[None, Some("last")]);
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(one, vec![1_000_000, 1, 0])
            .unwrap()
            .with_delta(two, vec![1, 1_000_000])
            .unwrap();
        assert_eq!(view.n_docs(), 1_000_001);
        assert_eq!(view.keyword_at(0), None);
        assert_eq!(view.keyword_at(1), None);
        assert_eq!(view.keyword_at(1_000_000), Some("last".to_owned()));
        for stale in ["old", "same", "new"] {
            assert_eq!(view.keyword_postings(stale), None);
        }
        assert_eq!(
            view.keyword_postings("last"),
            Some([1_000_000].into_iter().collect())
        );
        assert_eq!(
            view.layers
                .iter()
                .map(|layer| layer.ids.len())
                .sum::<usize>(),
            5
        );
        assert!(view.base_reader().is_none());
        crate::segment::DICTIONARY_SEARCHES.with(|count| count.set(0));
        assert_eq!(
            view.keyword_terms_all().unwrap(),
            vec![("last".to_owned(), [1_000_000].into_iter().collect())]
        );
        crate::segment::DICTIONARY_SEARCHES.with(|count| {
            assert_eq!(count.get(), 0,
            "enumeration must reuse known dictionary ordinals instead of repeating term searches")
        });
    }

    #[test]
    fn prepared_replacement_reuses_row_maps_and_preserves_later_layers() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword(
            &dir.path().join("prepared-base"),
            &[Some("base"), Some("keep")],
        );
        let first = keyword(&dir.path().join("prepared-first"), &[None]);
        let second = keyword(&dir.path().join("prepared-second"), &[Some("added")]);
        let later = keyword(&dir.path().join("prepared-later"), &[Some("latest")]);
        let captured = ComposedSegmentReader::from_base(base.clone())
            .with_delta(first.clone(), vec![0])
            .unwrap()
            .with_delta(second.clone(), vec![1_000_000])
            .unwrap();
        let live = captured.with_delta(later.clone(), vec![1_000_000]).unwrap();
        for includes_base in [false, true] {
            let (values, ids) = if includes_base {
                (
                    vec![Some("added"), Some("keep"), None],
                    vec![1_000_000, 1, 0],
                )
            } else {
                (vec![Some("added"), None], vec![1_000_000, 0])
            };
            let output = keyword(
                &dir.path().join(format!("prepared-{includes_base}")),
                &values,
            );
            let prepared = captured
                .prepare_replacement(
                    includes_base.then_some(&base),
                    &[first.clone(), second.clone()],
                    output,
                    ids,
                )
                .unwrap();
            let next = live.install_prepared_replacement(&prepared).unwrap();
            assert_eq!(next.keyword_at(0), None);
            assert_eq!(next.keyword_at(1).as_deref(), Some("keep"));
            assert_eq!(next.keyword_at(1_000_000).as_deref(), Some("latest"));
            assert!(Arc::ptr_eq(&next.layers.last().unwrap().reader, &later));
            let installed = if includes_base {
                next.base_map.as_ref().unwrap()
            } else {
                &next.layers[0]
            };
            assert!(
                Arc::ptr_eq(installed, &prepared.replacement),
                "binding must reuse the prepared row map, not rebuild it"
            );
            assert!(
                next.install_prepared_replacement(&prepared).is_err(),
                "already replaced input identities must be rejected"
            );
            let incomplete = ComposedSegmentReader::from_base(base.clone())
                .with_delta(first.clone(), vec![0])
                .unwrap();
            assert!(
                incomplete.install_prepared_replacement(&prepared).is_err(),
                "a missing input must be rejected without slicing past the layer list"
            );
        }
    }

    #[test]
    fn mapped_base_replacement_preserves_newer_layers_and_rejects_stale_inputs() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword(&dir.path().join("base"), &[Some("old"), Some("keep")]);
        let delta = keyword(&dir.path().join("delta"), &[None, Some("added")]);
        let later = keyword(&dir.path().join("later"), &[Some("later")]);
        let merged = keyword(
            &dir.path().join("merged"),
            &[Some("added"), Some("keep"), None],
        );
        let view = ComposedSegmentReader::from_base(base.clone())
            .with_delta(delta.clone(), vec![0, 1_000_000])
            .unwrap()
            .with_delta(later.clone(), vec![1_000_000])
            .unwrap();
        let replaced = view
            .replace_base_inputs(
                &base,
                &[delta.clone()],
                merged.clone(),
                vec![1_000_000, 1, 0],
            )
            .expect("replace captured base and prefix");
        assert_eq!(replaced.keyword_at(0), None);
        assert_eq!(replaced.keyword_at(1).as_deref(), Some("keep"));
        assert_eq!(replaced.keyword_at(1_000_000).as_deref(), Some("later"));
        assert_eq!(replaced.delta_readers().len(), 1);
        assert!(Arc::ptr_eq(&replaced.delta_readers()[0], &later));
        assert!(replaced
            .replace_base_inputs(&base, &[delta], merged.clone(), vec![1_000_000, 1, 0])
            .is_err());
        drop(view);
        assert_eq!(
            Arc::strong_count(&base),
            1,
            "replaced view must release old base"
        );
        assert_eq!(replaced.immutable_base_reader().as_ref().n_docs(), 3);
    }

    #[test]
    fn replacement_keeps_newer_layers_and_rejects_coverage_drift() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword(&dir.path().join("base"), &[Some("base"), Some("base")]);
        let first = keyword(&dir.path().join("first"), &[Some("one")]);
        let second = keyword(&dir.path().join("second"), &[Some("two")]);
        let newer = keyword(&dir.path().join("newer"), &[Some("newest")]);
        let merged = keyword(&dir.path().join("merged"), &[Some("one"), Some("two")]);
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(first, vec![0])
            .unwrap()
            .with_delta(second, vec![1])
            .unwrap()
            .with_delta(newer, vec![1])
            .unwrap();
        let replaced = view
            .replace_delta_range(0, 2, merged.clone(), vec![0, 1])
            .unwrap();
        assert_eq!(replaced.layers.len(), 2);
        assert_eq!(replaced.keyword_at(0).as_deref(), Some("one"));
        assert_eq!(replaced.keyword_at(1).as_deref(), Some("newest"));
        assert!(view.replace_delta_range(0, 2, merged, vec![0]).is_err());
    }
    /// #4246: the live-term walk is exact under layer coverage AND pending
    /// tombstones, agrees with the materialized composition, and copies
    /// nothing.
    #[test]
    fn live_text_term_count_is_exact_under_coverage_and_tombstones() {
        let dir = tempfile::tempdir().unwrap();
        let rows: &[Option<&[(&str, u32)]>] = &[
            Some(&[("a", 1), ("b", 1)]),
            Some(&[("a", 1), ("c", 1)]),
            Some(&[("e", 1)]),
        ];
        let base = text(&dir.path().join("base"), rows);
        let delta_rows: &[Option<&[(&str, u32)]>] = &[Some(&[("d", 1), ("a", 1)])];
        let delta = text(&dir.path().join("delta"), delta_rows);
        // Row 1 is rewritten by the delta: `c` is gone, `a` survives twice.
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(delta, vec![1])
            .unwrap();
        let brute = |dead: &RoaringBitmap| {
            view.text_tokens_all()
                .unwrap()
                .into_iter()
                .filter(|(_, ids, _)| ids.iter().any(|id| !dead.contains(*id)))
                .count() as u64
        };
        let none = RoaringBitmap::new();
        let dead: RoaringBitmap = [0u32].into_iter().collect();
        let all: RoaringBitmap = [0u32, 1, 2].into_iter().collect();
        assert_eq!(brute(&none), 4, "a b d e");
        assert_eq!(
            brute(&dead),
            3,
            "b dies with row 0; a survives in the delta row"
        );
        assert_eq!(brute(&all), 0);

        reset_text_posting_clones();
        assert_eq!(view.live_text_term_count(&none), Some(4));
        assert_eq!(view.live_text_term_count(&dead), Some(3));
        assert_eq!(view.live_text_term_count(&all), Some(0));
        assert_eq!(view.text_term_has_live_doc("a", &none), Some(true));
        assert_eq!(
            view.text_term_has_live_doc("c", &none),
            Some(false),
            "hidden by coverage"
        );
        assert_eq!(
            view.text_term_has_live_doc("b", &dead),
            Some(false),
            "hidden by a tombstone"
        );
        assert_eq!(view.text_term_has_live_doc("a", &dead), Some(true));
        assert_eq!(
            view.text_term_has_live_doc("zzz", &none),
            Some(false),
            "absent"
        );
        assert_eq!(
            text_posting_clones(),
            0,
            "liveness never materializes a posting"
        );
    }

    /// #4246: a window merge of additive layers changes no live term, so the
    /// O(1) count survives it; a merge that folds the base in is re-derived
    /// from the merged dictionary; a hidden row still voids it.
    #[test]
    fn compaction_keeps_or_refolds_the_distinct_term_count() {
        let dir = tempfile::tempdir().unwrap();
        let base = text(&dir.path().join("base"), &[Some(&[("a", 1)])]);
        let first = text(&dir.path().join("first"), &[Some(&[("b", 1)])]);
        let second = text(&dir.path().join("second"), &[Some(&[("c", 1), ("a", 1)])]);
        let third = text(&dir.path().join("third"), &[Some(&[("d", 1)])]);
        let view = ComposedSegmentReader::from_base(base.clone())
            .with_delta(first.clone(), vec![1])
            .unwrap()
            .with_delta(second.clone(), vec![2])
            .unwrap()
            .with_delta(third, vec![3])
            .unwrap();
        assert_eq!(view.known_distinct_terms(), Some(4), "a b c d");
        let none = RoaringBitmap::new();

        // Window merge of `first` + `second` (rows 1 and 2); the base stays.
        let merged_rows: &[Option<&[(&str, u32)]>] =
            &[Some(&[("b", 1)]), Some(&[("c", 1), ("a", 1)])];
        let merged = text(&dir.path().join("merged"), merged_rows);
        assert_eq!(
            view.replace_delta_range(0, 2, merged.clone(), vec![1, 2])
                .unwrap()
                .known_distinct_terms(),
            Some(4)
        );
        let prepared = view
            .prepare_replacement(
                None,
                &[first.clone(), second.clone()],
                merged.clone(),
                vec![1, 2],
            )
            .unwrap();
        let windowed = view.install_prepared_replacement(&prepared).unwrap();
        assert_eq!(windowed.known_distinct_terms(), Some(4));
        assert_eq!(windowed.live_text_term_count(&none), Some(4));

        // Base-including merge of base + `first` + `second` into a dense local
        // space; the remaining `third` layer is re-folded on top.
        let rebased_rows: &[Option<&[(&str, u32)]>] = &[
            Some(&[("a", 1)]),
            Some(&[("b", 1)]),
            Some(&[("c", 1), ("a", 1)]),
        ];
        let rebased = text(&dir.path().join("rebased"), rebased_rows);
        let prepared = view
            .prepare_replacement(
                Some(&base),
                &[first.clone(), second.clone()],
                rebased,
                vec![0, 1, 2],
            )
            .unwrap();
        let folded = view.install_prepared_replacement(&prepared).unwrap();
        assert_eq!(folded.known_distinct_terms(), Some(4));
        assert_eq!(folded.live_text_term_count(&none), Some(4));

        // An update hiding row 0 voids the count; a window merge below it
        // cannot restore what it never had, and the walk still answers.
        let update = text(&dir.path().join("update"), &[Some(&[("z", 1)])]);
        let hidden = view.with_delta(update, vec![0]).unwrap();
        assert_eq!(hidden.known_distinct_terms(), None);
        assert_eq!(hidden.live_text_term_count(&none), Some(5), "a b c d z");
        let prepared = hidden
            .prepare_replacement(None, &[first, second], merged, vec![1, 2])
            .unwrap();
        let still_hidden = hidden.install_prepared_replacement(&prepared).unwrap();
        assert_eq!(still_hidden.known_distinct_terms(), None);
        assert_eq!(still_hidden.live_text_term_count(&none), Some(5));
    }

    #[test]
    fn string_cursor_merges_once_in_both_orders() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword(&dir.path().join("base"), &[Some("a"), Some("c")]);
        let delta = keyword(
            &dir.path().join("delta"),
            &[Some("b"), Some("c"), Some("d")],
        );
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(delta, vec![9, 8, 7])
            .unwrap();
        for (descending, expected) in [
            (false, vec!["a", "b", "c", "d"]),
            (true, vec!["d", "c", "b", "a"]),
        ] {
            let mut cursor = view.string_terms(descending).unwrap();
            let mut result = Vec::new();
            while let Some(term) = cursor.next().unwrap() {
                result.push(term);
            }
            assert_eq!(result, expected);
            assert!(cursor.next().unwrap().is_none());
        }
    }
    #[test]
    fn numeric_range_and_cursor_use_each_layers_rows_and_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let bp = dir.path().join("base");
        let dp = dir.path().join("delta");
        write_number_segment(&bp, 1, &[Some(-2.0), Some(4.0), Some(9.0)]).unwrap();
        write_number_segment(&dp, 2, &[Some(3.0), None, Some(4.0)]).unwrap();
        let base = Arc::new(SegmentReader::open(&bp).unwrap());
        let delta = Arc::new(SegmentReader::open(&dp).unwrap());
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(delta, vec![0, 2, 100])
            .unwrap();
        assert_eq!(view.number_at(0), Some(3.0));
        assert_eq!(view.number_at(2), None);
        assert_eq!(
            view.number_range(Some((bits(3.0), true)), Some((bits(4.0), true))),
            Some([0, 1, 100].into_iter().collect())
        );
        assert_eq!(
            view.number_range(Some((bits(3.0), false)), Some((bits(4.0), false))),
            Some(RoaringBitmap::new())
        );
        let mut keys = view
            .number_keys(Some((bits(3.0), true)), Some((bits(4.0), true)), true)
            .unwrap();
        assert_eq!(keys.next().unwrap(), Some(bits(4.0)));
        assert_eq!(keys.next().unwrap(), Some(bits(3.0)));
        assert_eq!(keys.next().unwrap(), None);
        assert_eq!(view.number_range_distinct_count(None, None), Some(2));
    }
    #[test]
    fn text_tf_length_and_presence_choose_the_same_version() {
        let dir = tempfile::tempdir().unwrap();
        let base = text(
            &dir.path().join("base"),
            &[Some(&[("old", 3)]), Some(&[("shared", 2)])],
        );
        let delta = text(
            &dir.path().join("delta"),
            &[Some(&[("shared", 5), ("new", 1)]), None, Some(&[])],
        );
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(delta, vec![0, 1, 50])
            .unwrap();
        assert!(view.text_postings_arc("old").is_none());
        let p = view.text_postings_arc("shared").unwrap();
        assert_eq!(*p, (vec![0], vec![5]));
        assert_eq!(view.text_token_df("shared"), 1);
        assert_eq!(view.text_doc_len(0), 6);
        assert!(!view.text_is_present(1));
        assert!(view.text_is_present(50));
        assert_eq!(view.text_doc_len(50), 0);
        let lens = view.text_doc_lens().unwrap();
        assert_eq!(lens.len(), view.n_docs() as usize);
        for id in 0..view.n_docs() {
            assert_eq!(lens[id as usize], view.text_doc_len(id), "id {id}");
        }
        assert_eq!(view.text_tokens_all().unwrap().len(), 2);
    }
    #[test]
    fn set_and_hash_point_reads_keep_explicit_empty_and_absent_distinct() {
        let dir = tempfile::tempdir().unwrap();
        let bp = dir.path().join("base");
        let dp = dir.path().join("delta");
        let members = vec!["x".to_owned()];
        let postings = BTreeMap::from([("x".to_owned(), [0].into_iter().collect())]);
        write_set_segment(&bp, 1, &[Some(&members)], &postings).unwrap();
        write_set_segment(&dp, 2, &[None, Some(&[])], &BTreeMap::new()).unwrap();
        let view = ComposedSegmentReader::from_base(Arc::new(SegmentReader::open(&bp).unwrap()))
            .with_delta(Arc::new(SegmentReader::open(&dp).unwrap()), vec![0, 10])
            .unwrap();
        assert_eq!(view.set_at(0), None);
        assert_eq!(view.set_at(10), Some(vec![]));
        assert_eq!(view.set_postings("x"), None);
        let hp = dir.path().join("hashbase");
        let hd = dir.path().join("hashdelta");
        write_hash_segment(&hp, 1, &[Some(99)]).unwrap();
        write_hash_segment(&hd, 2, &[None, Some(0)]).unwrap();
        let hashes = ComposedSegmentReader::from_base(Arc::new(SegmentReader::open(&hp).unwrap()))
            .with_delta(Arc::new(SegmentReader::open(&hd).unwrap()), vec![0, 30])
            .unwrap();
        assert_eq!(hashes.hash_at(0), None);
        assert_eq!(hashes.hash_at(30), Some(0));
    }
    #[test]
    fn rejects_invalid_sparse_mapping_and_keeps_base_fast_path() {
        let dir = tempfile::tempdir().unwrap();
        let base = keyword(&dir.path().join("base"), &[Some("a"), None]);
        let view = ComposedSegmentReader::from_base(base.clone());
        assert!(view.with_delta(base.clone(), vec![0]).is_err());
        assert!(view.with_delta(base.clone(), vec![1, 1]).is_err());
        assert!(view.with_delta(base.clone(), vec![0, u32::MAX]).is_err());
        assert!(Arc::ptr_eq(view.base_reader().unwrap(), &base));
        assert_eq!(view.keyword_postings("a"), base.keyword_postings("a"));
        assert_eq!(view.keyword_at(1), base.keyword_at(1));
    }

    fn naive_uncovered(ids: &[u32], tfs: &[u32], covered: &RoaringBitmap) -> (Vec<u32>, Vec<u32>) {
        ids.iter()
            .zip(tfs)
            .filter(|(id, _)| !covered.contains(**id))
            .map(|(&id, &tf)| (id, tf))
            .unzip()
    }

    #[test]
    fn retain_uncovered_sorted_matches_naive_filter_on_both_paths() {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(0x4246);
        for round in 0..400 {
            let n = rng.gen_range(0..300usize);
            let mut ids: Vec<u32> = (0..n).map(|_| rng.gen_range(0..1200u32)).collect();
            ids.sort_unstable();
            ids.dedup();
            let tfs: Vec<u32> = ids.iter().map(|_| rng.gen_range(1..9u32)).collect();
            let mut covered = RoaringBitmap::new();
            // Sparse rounds exercise the galloping path, dense rounds the lockstep one.
            let count = if round % 2 == 0 {
                rng.gen_range(0..4u32)
            } else {
                rng.gen_range(0..400u32)
            };
            for _ in 0..count {
                let from_posting = !ids.is_empty() && rng.gen_bool(0.7);
                let id = if from_posting {
                    ids[rng.gen_range(0..ids.len())]
                } else {
                    rng.gen_range(0..1400u32)
                };
                covered.insert(id);
            }
            let (mut got_ids, mut got_tfs) = (Vec::new(), Vec::new());
            retain_uncovered_sorted(&ids, &tfs, &covered, &mut got_ids, &mut got_tfs);
            assert_eq!(
                (got_ids, got_tfs),
                naive_uncovered(&ids, &tfs, &covered),
                "round {round} n {n} covered {count}"
            );
        }
        let ids = vec![5, 9, 20, 21, 700];
        let tfs = vec![1, 2, 3, 4, 5];
        let (mut a, mut b) = (Vec::new(), Vec::new());
        retain_uncovered_sorted(
            &ids,
            &tfs,
            &RoaringBitmap::from_iter([5u32, 700, 9000]),
            &mut a,
            &mut b,
        );
        assert_eq!((a, b), (vec![9, 20, 21], vec![2, 3, 4]));
        let (mut a, mut b) = (Vec::new(), Vec::new());
        retain_uncovered_sorted(
            &ids,
            &tfs,
            &RoaringBitmap::from_iter(ids.iter().copied()),
            &mut a,
            &mut b,
        );
        assert!(a.is_empty() && b.is_empty());
    }

    const MERGE_TOKENS: [&str; 3] = ["shared", "rare", "absent-from-base"];

    fn random_text_rows(
        rng: &mut impl rand::Rng,
        n: usize,
    ) -> Vec<Option<Vec<(&'static str, u32)>>> {
        (0..n)
            .map(|_| {
                if rng.gen_bool(0.15) {
                    return None;
                }
                let mut row = Vec::new();
                for (index, token) in MERGE_TOKENS.iter().enumerate() {
                    let keep = match index {
                        0 => rng.gen_bool(0.8),
                        1 => rng.gen_bool(0.2),
                        _ => rng.gen_bool(0.3),
                    };
                    if keep {
                        row.push((*token, rng.gen_range(1..12u32)));
                    }
                }
                Some(row)
            })
            .collect()
    }

    fn text_rows(path: &Path, rows: &[Option<Vec<(&str, u32)>>]) -> Arc<SegmentReader> {
        let refs: Vec<Option<&[(&str, u32)]>> = rows.iter().map(|r| r.as_deref()).collect();
        text(path, &refs)
    }

    fn assert_merge_matches_reference(view: &ComposedSegmentReader, label: &str) {
        for token in MERGE_TOKENS.iter().chain(["never-written"].iter()) {
            let fast = view.merge_text_postings(token).map(|p| (*p).clone());
            let reference = view
                .merge_text_postings_reference(token)
                .map(|p| (*p).clone());
            assert_eq!(fast, reference, "{label} token {token}");
            let cached = view.text_postings_arc(token).map(|p| (*p).clone());
            assert_eq!(
                cached, reference,
                "{label} token {token} via text_postings_arc"
            );
            assert_eq!(
                view.text_token_df(token),
                reference.as_ref().map_or(0, |p| p.0.len()),
                "{label} token {token} df"
            );
            if let Some((ids, tfs)) = &reference {
                assert!(
                    ids.windows(2).all(|w| w[0] < w[1]),
                    "{label} token {token} ids sorted unique"
                );
                assert_eq!(ids.len(), tfs.len());
            }
        }
    }

    /// Fixture-level model of the composition: the newest layer covering an
    /// id owns its row; an id no layer covers falls back to the (mapped) base.
    fn model_posting(
        token: &str,
        base_ids: &[u32],
        base_rows: &[Option<Vec<(&str, u32)>>],
        layers: &[(Vec<u32>, Vec<Option<Vec<(&str, u32)>>>)],
    ) -> Option<(Vec<u32>, Vec<u32>)> {
        let tf_in = |row: &Option<Vec<(&str, u32)>>| {
            row.as_ref()
                .and_then(|r| r.iter().find(|(t, _)| *t == token).map(|&(_, tf)| tf))
        };
        let mut out = Vec::new();
        for id in 0..200u32 {
            let owner = layers.iter().rev().find_map(|(ids, rows)| {
                ids.iter().position(|&g| g == id).map(|local| &rows[local])
            });
            let tf = match owner {
                Some(row) => tf_in(row),
                None => base_ids
                    .iter()
                    .position(|&g| g == id)
                    .and_then(|local| tf_in(&base_rows[local])),
            };
            if let Some(tf) = tf {
                out.push((id, tf));
            }
        }
        if out.is_empty() {
            None
        } else {
            Some(out.into_iter().unzip())
        }
    }

    #[test]
    fn coverage_driven_text_merge_matches_the_layered_reference() {
        use rand::seq::SliceRandom;
        use rand::{Rng, SeedableRng};
        let dir = tempfile::tempdir().unwrap();
        let mut rng = rand::rngs::StdRng::seed_from_u64(0x0424_6);
        for round in 0..60 {
            let round_dir = dir.path().join(format!("r{round}"));
            std::fs::create_dir_all(&round_dir).unwrap();
            let n_base = rng.gen_range(1..48usize);
            let base_rows = random_text_rows(&mut rng, n_base);
            let base = text_rows(&round_dir.join("base"), &base_rows);
            let mapped = round % 3 != 0;
            let base_ids: Vec<u32> = if mapped {
                let mut pool: Vec<u32> = (0..140).collect();
                pool.shuffle(&mut rng);
                let mut ids: Vec<u32> = pool[..n_base].to_vec();
                if round % 6 == 1 {
                    ids.sort_unstable();
                }
                ids
            } else {
                (0..n_base as u32).collect()
            };
            let mut view = if mapped {
                ComposedSegmentReader::from_mapped_base(base, base_ids.clone()).unwrap()
            } else {
                ComposedSegmentReader::from_base(base)
            };
            let mut model_layers: Vec<(Vec<u32>, Vec<Option<Vec<(&str, u32)>>>)> = Vec::new();
            let check = |view: &ComposedSegmentReader,
                         model_layers: &[(Vec<u32>, Vec<Option<Vec<(&str, u32)>>>)],
                         label: &str| {
                assert_merge_matches_reference(view, label);
                for token in MERGE_TOKENS {
                    assert_eq!(
                        view.text_postings_arc(token).map(|p| (*p).clone()),
                        model_posting(token, &base_ids, &base_rows, model_layers),
                        "{label} token {token} vs fixture model"
                    );
                }
            };
            check(
                &view,
                &model_layers,
                &format!("round {round} layers 0 mapped {mapped}"),
            );
            let layers = rng.gen_range(0..=5usize);
            for layer in 0..layers {
                let m = rng.gen_range(1..=14usize);
                let mut pool: Vec<u32> = (0..140).collect();
                pool.shuffle(&mut rng);
                let mut ids: Vec<u32> = pool[..m].to_vec();
                if rng.gen_bool(0.5) {
                    ids.sort_unstable();
                }
                let rows = random_text_rows(&mut rng, m);
                let reader = text_rows(&round_dir.join(format!("l{layer}")), &rows);
                view = view.with_delta(reader, ids.clone()).unwrap();
                model_layers.push((ids, rows));
                check(
                    &view,
                    &model_layers,
                    &format!("round {round} layers {} mapped {mapped}", layer + 1),
                );
            }
        }
    }

    #[test]
    fn coverage_driven_text_merge_gallops_over_a_wide_base() {
        // Coverage far sparser than the base posting (|C| · 16 < df) takes the
        // galloping path; the hidden rows and the layer rows must still land
        // exactly where the reference puts them.
        let dir = tempfile::tempdir().unwrap();
        let base_rows: Vec<Option<Vec<(&str, u32)>>> = (0..700u32)
            .map(|i| Some(vec![("shared", 1 + i % 5)]))
            .collect();
        let base = text_rows(&dir.path().join("base"), &base_rows);
        let older = text_rows(
            &dir.path().join("older"),
            &[Some(vec![("shared", 40)]), None, Some(vec![])],
        );
        let newer = text_rows(
            &dir.path().join("newer"),
            &[Some(vec![("shared", 50)]), Some(vec![("shared", 51)])],
        );
        let view = ComposedSegmentReader::from_base(base)
            .with_delta(older, vec![10, 699, 300])
            .unwrap()
            .with_delta(newer, vec![10, 900])
            .unwrap();
        assert_merge_matches_reference(&view, "wide base");
        let p = view.text_postings_arc("shared").unwrap();
        assert_eq!(p.0.len(), 700 - 3 + 2);
        let tf_of = |id: u32| p.0.binary_search(&id).ok().map(|i| p.1[i]);
        assert_eq!(tf_of(10), Some(50));
        assert_eq!(tf_of(900), Some(51));
        assert_eq!(tf_of(300), None);
        assert_eq!(tf_of(699), None);
        assert_eq!(tf_of(11), Some(1 + 11 % 5));
    }

    /// `text_posting_at` (#4246) against the materialized composition: on a
    /// cold view every token resolves `Sparse` with the exact df of the
    /// un-hidden ids and the tf of every un-hidden candidate, for dense,
    /// mapped and layered compositions alike; once `text_postings_arc` has
    /// made the posting resident the same call reports `Cached` and never
    /// streams. A token no source carries is `None` on a dense base and an
    /// empty `Sparse` on a composed one — both mean "no posting".
    #[test]
    fn sparse_text_posting_at_matches_the_materialized_composition() {
        use rand::seq::SliceRandom;
        use rand::{Rng, SeedableRng};
        let dir = tempfile::tempdir().unwrap();
        let mut rng = rand::rngs::StdRng::seed_from_u64(0x4246_0003);
        let mut sparse_seen = 0usize;
        for round in 0..60 {
            let round_dir = dir.path().join(format!("r{round}"));
            std::fs::create_dir_all(&round_dir).unwrap();
            let n_base = rng.gen_range(1..48usize);
            let base_rows = random_text_rows(&mut rng, n_base);
            let base = text_rows(&round_dir.join("base"), &base_rows);
            let mapped = round % 3 != 0;
            let mut view = if mapped {
                let mut pool: Vec<u32> = (0..140).collect();
                pool.shuffle(&mut rng);
                let mut ids: Vec<u32> = pool[..n_base].to_vec();
                if round % 6 == 1 {
                    ids.sort_unstable();
                }
                ComposedSegmentReader::from_mapped_base(base, ids).unwrap()
            } else {
                ComposedSegmentReader::from_base(base)
            };
            let layers = if round % 3 == 0 && round % 2 == 0 {
                0
            } else {
                rng.gen_range(0..=5usize)
            };
            for layer in 0..layers {
                let m = rng.gen_range(1..=14usize);
                let mut pool: Vec<u32> = (0..140).collect();
                pool.shuffle(&mut rng);
                let mut ids: Vec<u32> = pool[..m].to_vec();
                if rng.gen_bool(0.5) {
                    ids.sort_unstable();
                }
                let rows = random_text_rows(&mut rng, m);
                let reader = text_rows(&round_dir.join(format!("l{layer}")), &rows);
                view = view.with_delta(reader, ids).unwrap();
            }
            let hidden_set: RoaringBitmap = (0..200u32).filter(|_| rng.gen_bool(0.25)).collect();
            let mut candidates: Vec<u32> = (0..200u32).filter(|_| rng.gen_bool(0.1)).collect();
            if rng.gen_bool(0.2) {
                candidates.clear();
            }
            let label = format!("round {round} mapped {mapped} layers {layers}");
            for token in MERGE_TOKENS.iter().chain(["never-indexed"].iter()) {
                // Cold: nothing resident yet, so the answer must be streamed.
                let cold = view.text_posting_at(token, &candidates, |id| hidden_set.contains(id));
                let full = view.text_postings_arc(token);
                match &full {
                    None => match cold {
                        None => assert!(
                            view.dense_base_only(),
                            "{label} {token}: None only on a dense base"
                        ),
                        Some(TextPostingAt::Sparse { df, ref hits }) => {
                            assert_eq!((df, hits.len()), (0, 0), "{label} {token}: absent token");
                        }
                        Some(TextPostingAt::Cached(_)) => {
                            panic!("{label} {token}: cold view reported Cached")
                        }
                    },
                    Some(full) => {
                        let want_df = full
                            .0
                            .iter()
                            .filter(|id| !hidden_set.contains(**id))
                            .count();
                        let want_hits: Vec<(u32, u32)> = full
                            .0
                            .iter()
                            .zip(&full.1)
                            .filter(|(id, _)| {
                                !hidden_set.contains(**id) && candidates.binary_search(id).is_ok()
                            })
                            .map(|(&id, &tf)| (id, tf))
                            .collect();
                        match cold {
                            Some(TextPostingAt::Sparse { df, hits }) => {
                                assert_eq!(df, want_df, "{label} {token}: df");
                                assert_eq!(hits, want_hits, "{label} {token}: candidate hits");
                                sparse_seen += 1;
                            }
                            other => panic!("{label} {token}: expected Sparse, got {other:?}"),
                        }
                        // Warm: the resident posting is handed back untouched.
                        match view.text_posting_at(token, &candidates, |id| hidden_set.contains(id)) {
                            Some(TextPostingAt::Cached(hit)) => assert!(Arc::ptr_eq(&hit, full), "{label} {token}: cached arc"),
                            other => panic!("{label} {token}: expected Cached after text_postings_arc, got {other:?}"),
                        }
                    }
                }
            }
        }
        assert!(
            sparse_seen > 100,
            "the fixture must exercise the sparse path ({sparse_seen})"
        );
    }

    #[test]
    fn coverage_driven_text_merge_keeps_torn_map_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let base = text(
            &dir.path().join("base"),
            &[Some(&[("shared", 1)]), Some(&[("shared", 2)])],
        );
        let delta = text(
            &dir.path().join("delta"),
            &[Some(&[("shared", 9)]), Some(&[("shared", 8)])],
        );
        let torn = Arc::new(DeltaLayer {
            reader: delta,
            ids: vec![1],
            local_by_global: BTreeMap::from([(1, 0)]),
            coverage: RoaringBitmap::from_iter([1u32]),
            private: false,
        });
        let view = ComposedSegmentReader {
            base,
            base_map: None,
            layers: vec![torn],
            n_docs: 2,
            has_catalog_base: true,
            distinct_terms: None,
            query_cache: Arc::default(),
        };
        assert!(view.merge_text_postings_reference("shared").is_none());
        assert!(view.merge_text_postings("shared").is_none());
        assert!(view.text_postings_arc("shared").is_none());
    }
}
