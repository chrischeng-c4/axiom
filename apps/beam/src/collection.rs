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
//! [`add`]: Collection::add
//! [`add_with_payload`]: Collection::add_with_payload

use crate::payload::Payload;

/// Distance / similarity metric a collection is scored under.
///
/// The numeric [`Metric::code`] is the convention shared verbatim by the CPU
/// oracle and the GPU WGSL kernel, so their per-row scores — and therefore their
/// top-k — agree bit-for-bit in intent (float rounding aside).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// A named, in-memory vector collection with a fixed dimension and metric.
#[derive(Debug, Clone)]
pub struct Collection {
    /// Human/external identifier for the collection itself.
    pub id: String,
    dim: usize,
    metric: Metric,
    /// Row-major vectors: `n * dim` f32s, row `i` at `[i*dim .. (i+1)*dim]`.
    data: Vec<f32>,
    /// External id per row, `external_ids[i]` pairs with row `i`.
    external_ids: Vec<String>,
    /// Attribute payload per row, `payloads[i]` pairs with row `i` (empty for
    /// rows added without one). The row-aligned metadata filtered search reads.
    payloads: Vec<Payload>,
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
        if vector.len() != self.dim {
            anyhow::bail!(
                "vector dim mismatch: collection dim is {}, got {}",
                self.dim,
                vector.len()
            );
        }
        match self.metric {
            Metric::Cosine => self.data.extend_from_slice(&l2_normalize(vector)),
            _ => self.data.extend_from_slice(vector),
        }
        self.external_ids.push(external_id.into());
        self.payloads.push(payload);
        Ok(())
    }

    /// Replace the payload of an existing row (used to assign deterministic
    /// attributes to a pre-built corpus in tests and the bench). Panics if `i`
    /// is out of range.
    pub fn set_payload(&mut self, i: usize, payload: Payload) {
        self.payloads[i] = payload;
    }

    /// Number of stored vectors (`n`).
    pub fn len(&self) -> usize {
        self.external_ids.len()
    }

    /// Whether the collection holds zero vectors.
    pub fn is_empty(&self) -> bool {
        self.external_ids.is_empty()
    }

    /// The vector dimension every row has.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// The collection's metric.
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// The full row-major `n * dim` backing buffer — the exact slice uploaded to
    /// the GPU storage buffer and scanned by the CPU oracle.
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
}
