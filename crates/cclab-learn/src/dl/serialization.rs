//! Model serialization: save/load model weights in JSON and binary formats.
//!
//! Provides a format-agnostic [`ModelWeights`] container that stores
//! parameter tensors as named flat vectors with shapes, plus helpers
//! to serialize/deserialize from JSON text or a compact binary format.

use super::nn::Layer;
use super::tensor::Tensor;
use std::collections::BTreeMap;
use std::io::{self, Read, Write};

/// Error type for serialization operations.
#[derive(Debug)]
pub enum SerError {
    /// I/O error during read/write.
    Io(io::Error),
    /// JSON parsing error.
    Json(String),
    /// Data integrity error.
    InvalidData(String),
}

impl std::fmt::Display for SerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Json(e) => write!(f, "json error: {}", e),
            Self::InvalidData(e) => write!(f, "invalid data: {}", e),
        }
    }
}

impl std::error::Error for SerError {}

impl From<io::Error> for SerError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// A single named parameter (tensor data + shape).
#[derive(Debug, Clone)]
pub struct ParamEntry {
    pub name: String,
    pub shape: Vec<usize>,
    pub data: Vec<f64>,
}

/// Container for model weights — a named collection of parameter tensors.
///
/// ```text
///   ModelWeights {
///       "layer0.weight" -> { shape: [3, 4], data: [...] },
///       "layer0.bias"   -> { shape: [4],    data: [...] },
///       ...
///   }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ModelWeights {
    pub params: BTreeMap<String, ParamEntry>,
}

impl ModelWeights {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a parameter entry.
    pub fn insert(&mut self, name: impl Into<String>, shape: Vec<usize>, data: Vec<f64>) {
        let name = name.into();
        self.params
            .insert(name.clone(), ParamEntry { name, shape, data });
    }

    /// Get a parameter entry by name.
    pub fn get(&self, name: &str) -> Option<&ParamEntry> {
        self.params.get(name)
    }

    /// Number of stored parameters.
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Whether the container is empty.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Total number of scalar values across all parameters.
    pub fn total_elements(&self) -> usize {
        self.params.values().map(|p| p.data.len()).sum()
    }
}

// ============================================================================
// Extract weights from layers
// ============================================================================

/// Extract weights from a `Layer` into a `ModelWeights` container.
///
/// Parameter names are generated as `param_{index}`.
pub fn extract_weights(layer: &mut dyn Layer) -> ModelWeights {
    let mut weights = ModelWeights::new();
    let params = layer.parameters();
    for (i, tensor) in params.into_iter().enumerate() {
        let name = format!("param_{}", i);
        weights.insert(name, tensor.shape.clone(), tensor.data.clone());
    }
    weights
}

/// Load weights back into a `Layer` from a `ModelWeights` container.
///
/// Parameters are matched by order (param_0, param_1, ...).
pub fn load_weights(layer: &mut dyn Layer, weights: &ModelWeights) -> Result<(), SerError> {
    let params = layer.parameters();
    for (i, tensor) in params.into_iter().enumerate() {
        let name = format!("param_{}", i);
        let entry = weights
            .get(&name)
            .ok_or_else(|| SerError::InvalidData(format!("missing parameter: {}", name)))?;
        if tensor.shape != entry.shape {
            return Err(SerError::InvalidData(format!(
                "shape mismatch for {}: expected {:?}, got {:?}",
                name, tensor.shape, entry.shape
            )));
        }
        tensor.data = entry.data.clone();
    }
    Ok(())
}

// ============================================================================
// JSON serialization (hand-rolled, no serde needed)
// ============================================================================

impl ModelWeights {
    /// Serialize to JSON string (compact, no external deps).
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\n");
        let total = self.params.len();
        for (idx, (name, entry)) in self.params.iter().enumerate() {
            out.push_str(&format!("  \"{}\": {{\n", escape_json(name)));
            // Shape
            out.push_str("    \"shape\": [");
            for (i, s) in entry.shape.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&s.to_string());
            }
            out.push_str("],\n");
            // Data
            out.push_str("    \"data\": [");
            for (i, v) in entry.data.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!("{:.8e}", v));
            }
            out.push(']');
            out.push_str("\n  }");
            if idx + 1 < total {
                out.push(',');
            }
            out.push('\n');
        }
        out.push('}');
        out
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self, SerError> {
        let mut weights = ModelWeights::new();
        let trimmed = json.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return Err(SerError::Json("expected JSON object".into()));
        }
        let inner = &trimmed[1..trimmed.len() - 1];
        // Parse each parameter block
        let mut pos = 0;
        let bytes = inner.as_bytes();
        while pos < bytes.len() {
            // Skip whitespace
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos >= bytes.len() {
                break;
            }
            // Parse name
            if bytes[pos] != b'"' {
                if bytes[pos] == b',' {
                    pos += 1;
                    continue;
                }
                return Err(SerError::Json(format!("expected '\"' at pos {}", pos)));
            }
            let name = parse_json_string(inner, &mut pos)?;
            // Skip colon
            skip_whitespace(inner, &mut pos);
            if pos >= inner.len() || inner.as_bytes()[pos] != b':' {
                return Err(SerError::Json("expected ':'".into()));
            }
            pos += 1;
            skip_whitespace(inner, &mut pos);
            // Parse object { "shape": [...], "data": [...] }
            if pos >= inner.len() || inner.as_bytes()[pos] != b'{' {
                return Err(SerError::Json("expected '{'".into()));
            }
            let obj_end = find_matching_brace(inner, pos)?;
            let obj_str = &inner[pos + 1..obj_end];
            pos = obj_end + 1;

            let shape = parse_field_array_usize(obj_str, "shape")?;
            let data = parse_field_array_f64(obj_str, "data")?;
            weights.insert(name, shape, data);
        }
        Ok(weights)
    }

    /// Save to a writer as JSON.
    pub fn save_json<W: Write>(&self, writer: &mut W) -> Result<(), SerError> {
        let json = self.to_json();
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Load from a reader as JSON.
    pub fn load_json<R: Read>(reader: &mut R) -> Result<Self, SerError> {
        let mut json = String::new();
        reader.read_to_string(&mut json)?;
        Self::from_json(&json)
    }
}

// ============================================================================
// Binary serialization
// ============================================================================

const BINARY_MAGIC: &[u8; 4] = b"CCLW"; // CCLab-Learn Weights
const BINARY_VERSION: u32 = 1;

impl ModelWeights {
    /// Save to a writer in compact binary format.
    ///
    /// Format:
    /// - 4 bytes: magic "CCLW"
    /// - 4 bytes: version (u32 LE)
    /// - 4 bytes: num_params (u32 LE)
    /// - For each parameter:
    ///   - 4 bytes: name length (u32 LE)
    ///   - N bytes: name (UTF-8)
    ///   - 4 bytes: ndim (u32 LE)
    ///   - ndim * 8 bytes: shape (u64 LE each)
    ///   - 4 bytes: data length (u32 LE)
    ///   - data_len * 8 bytes: data (f64 LE each)
    pub fn save_binary<W: Write>(&self, writer: &mut W) -> Result<(), SerError> {
        writer.write_all(BINARY_MAGIC)?;
        writer.write_all(&BINARY_VERSION.to_le_bytes())?;
        writer.write_all(&(self.params.len() as u32).to_le_bytes())?;

        for (name, entry) in &self.params {
            let name_bytes = name.as_bytes();
            writer.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
            writer.write_all(name_bytes)?;

            writer.write_all(&(entry.shape.len() as u32).to_le_bytes())?;
            for &s in &entry.shape {
                writer.write_all(&(s as u64).to_le_bytes())?;
            }

            writer.write_all(&(entry.data.len() as u32).to_le_bytes())?;
            for &v in &entry.data {
                writer.write_all(&v.to_le_bytes())?;
            }
        }
        Ok(())
    }

    /// Load from a reader in binary format.
    pub fn load_binary<R: Read>(reader: &mut R) -> Result<Self, SerError> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != BINARY_MAGIC {
            return Err(SerError::InvalidData("invalid magic bytes".into()));
        }

        let mut buf4 = [0u8; 4];
        reader.read_exact(&mut buf4)?;
        let version = u32::from_le_bytes(buf4);
        if version != BINARY_VERSION {
            return Err(SerError::InvalidData(format!(
                "unsupported version: {}",
                version
            )));
        }

        reader.read_exact(&mut buf4)?;
        let num_params = u32::from_le_bytes(buf4) as usize;

        let mut weights = ModelWeights::new();
        for _ in 0..num_params {
            // Name
            reader.read_exact(&mut buf4)?;
            let name_len = u32::from_le_bytes(buf4) as usize;
            let mut name_buf = vec![0u8; name_len];
            reader.read_exact(&mut name_buf)?;
            let name = String::from_utf8(name_buf)
                .map_err(|e| SerError::InvalidData(format!("invalid UTF-8: {}", e)))?;

            // Shape
            reader.read_exact(&mut buf4)?;
            let ndim = u32::from_le_bytes(buf4) as usize;
            let mut shape = Vec::with_capacity(ndim);
            let mut buf8 = [0u8; 8];
            for _ in 0..ndim {
                reader.read_exact(&mut buf8)?;
                shape.push(u64::from_le_bytes(buf8) as usize);
            }

            // Data
            reader.read_exact(&mut buf4)?;
            let data_len = u32::from_le_bytes(buf4) as usize;
            let mut data = Vec::with_capacity(data_len);
            for _ in 0..data_len {
                reader.read_exact(&mut buf8)?;
                data.push(f64::from_le_bytes(buf8));
            }

            weights.insert(name, shape, data);
        }
        Ok(weights)
    }

    /// Save to a file (auto-detect format from extension: .json or .bin).
    pub fn save_to_file(&self, path: &str) -> Result<(), SerError> {
        let mut file = std::fs::File::create(path)?;
        if path.ends_with(".json") {
            self.save_json(&mut file)
        } else {
            self.save_binary(&mut file)
        }
    }

    /// Load from a file (auto-detect format from extension: .json or .bin).
    pub fn load_from_file(path: &str) -> Result<Self, SerError> {
        let mut file = std::fs::File::open(path)?;
        if path.ends_with(".json") {
            Self::load_json(&mut file)
        } else {
            Self::load_binary(&mut file)
        }
    }
}

// ============================================================================
// JSON parsing helpers
// ============================================================================

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn skip_whitespace(s: &str, pos: &mut usize) {
    let bytes = s.as_bytes();
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn parse_json_string(s: &str, pos: &mut usize) -> Result<String, SerError> {
    let bytes = s.as_bytes();
    if *pos >= bytes.len() || bytes[*pos] != b'"' {
        return Err(SerError::Json("expected '\"'".into()));
    }
    *pos += 1;
    let start = *pos;
    while *pos < bytes.len() && bytes[*pos] != b'"' {
        if bytes[*pos] == b'\\' {
            *pos += 1;
        }
        *pos += 1;
    }
    if *pos >= bytes.len() {
        return Err(SerError::Json("unterminated string".into()));
    }
    let result = s[start..*pos].to_string();
    *pos += 1; // skip closing quote
    Ok(result)
}

fn find_matching_brace(s: &str, start: usize) -> Result<usize, SerError> {
    let bytes = s.as_bytes();
    let mut depth = 0;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(SerError::Json("unmatched brace".into()))
}

fn parse_field_array_usize(obj: &str, field: &str) -> Result<Vec<usize>, SerError> {
    let key = format!("\"{}\"", field);
    let idx = obj
        .find(&key)
        .ok_or_else(|| SerError::Json(format!("missing field: {}", field)))?;
    let after_key = idx + key.len();
    let rest = &obj[after_key..];
    let colon_idx = rest
        .find(':')
        .ok_or_else(|| SerError::Json("expected ':'".into()))?;
    let after_colon = &rest[colon_idx + 1..];
    let bracket_start = after_colon
        .find('[')
        .ok_or_else(|| SerError::Json("expected '['".into()))?;
    let bracket_end = after_colon
        .find(']')
        .ok_or_else(|| SerError::Json("expected ']'".into()))?;
    let arr_str = &after_colon[bracket_start + 1..bracket_end];
    if arr_str.trim().is_empty() {
        return Ok(Vec::new());
    }
    arr_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<usize>()
                .map_err(|e| SerError::Json(format!("invalid usize: {}", e)))
        })
        .collect()
}

fn parse_field_array_f64(obj: &str, field: &str) -> Result<Vec<f64>, SerError> {
    let key = format!("\"{}\"", field);
    let idx = obj
        .find(&key)
        .ok_or_else(|| SerError::Json(format!("missing field: {}", field)))?;
    let after_key = idx + key.len();
    let rest = &obj[after_key..];
    let colon_idx = rest
        .find(':')
        .ok_or_else(|| SerError::Json("expected ':'".into()))?;
    let after_colon = &rest[colon_idx + 1..];
    let bracket_start = after_colon
        .find('[')
        .ok_or_else(|| SerError::Json("expected '['".into()))?;
    let bracket_end = after_colon
        .rfind(']')
        .ok_or_else(|| SerError::Json("expected ']'".into()))?;
    let arr_str = &after_colon[bracket_start + 1..bracket_end];
    if arr_str.trim().is_empty() {
        return Ok(Vec::new());
    }
    arr_str
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<f64>()
                .map_err(|e| SerError::Json(format!("invalid f64: {}", e)))
        })
        .collect()
}

// ============================================================================
// Convenience: create Tensor from ParamEntry
// ============================================================================

impl ParamEntry {
    /// Convert to a Tensor.
    pub fn to_tensor(&self) -> Tensor {
        Tensor::new(self.data.clone(), self.shape.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_weights_basic() {
        let mut w = ModelWeights::new();
        w.insert("weight", vec![3, 4], vec![1.0; 12]);
        w.insert("bias", vec![4], vec![0.0; 4]);
        assert_eq!(w.len(), 2);
        assert_eq!(w.total_elements(), 16);
        assert!(!w.is_empty());
    }

    #[test]
    fn test_json_roundtrip() {
        let mut w = ModelWeights::new();
        w.insert("w0", vec![2, 3], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        w.insert("b0", vec![3], vec![0.1, 0.2, 0.3]);

        let json = w.to_json();
        let loaded = ModelWeights::from_json(&json).unwrap();
        assert_eq!(loaded.len(), 2);

        let w0 = loaded.get("w0").unwrap();
        assert_eq!(w0.shape, vec![2, 3]);
        assert_eq!(w0.data.len(), 6);
        for (a, b) in w0.data.iter().zip([1.0, 2.0, 3.0, 4.0, 5.0, 6.0].iter()) {
            assert!((a - b).abs() < 1e-6, "expected {}, got {}", b, a);
        }

        let b0 = loaded.get("b0").unwrap();
        assert_eq!(b0.shape, vec![3]);
    }

    #[test]
    fn test_binary_roundtrip() {
        let mut w = ModelWeights::new();
        w.insert("layer.weight", vec![4, 5], vec![0.42; 20]);
        w.insert("layer.bias", vec![5], vec![-0.1; 5]);

        let mut buf = Vec::new();
        w.save_binary(&mut buf).unwrap();

        let loaded = ModelWeights::load_binary(&mut &buf[..]).unwrap();
        assert_eq!(loaded.len(), 2);

        let lw = loaded.get("layer.weight").unwrap();
        assert_eq!(lw.shape, vec![4, 5]);
        assert_eq!(lw.data.len(), 20);
        for &v in &lw.data {
            assert!((v - 0.42).abs() < 1e-15);
        }
    }

    #[test]
    fn test_extract_and_load_weights() {
        use super::super::nn::Linear;

        let mut layer = Linear::new(3, 2, 42);
        let original_weight = layer.weight.data.clone();
        let original_bias = layer.bias.data.clone();

        let weights = extract_weights(&mut layer);
        assert_eq!(weights.len(), 2);

        // Modify layer
        layer.weight.data = vec![0.0; 6];
        layer.bias.data = vec![0.0; 2];

        // Load back
        load_weights(&mut layer, &weights).unwrap();
        assert_eq!(layer.weight.data, original_weight);
        assert_eq!(layer.bias.data, original_bias);
    }

    #[test]
    fn test_file_roundtrip_json() {
        let mut w = ModelWeights::new();
        w.insert("test", vec![2], vec![3.14, 2.72]);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.json");
        let path_str = path.to_str().unwrap();

        w.save_to_file(path_str).unwrap();
        let loaded = ModelWeights::load_from_file(path_str).unwrap();
        let t = loaded.get("test").unwrap();
        assert!((t.data[0] - 3.14).abs() < 1e-6);
        assert!((t.data[1] - 2.72).abs() < 1e-6);
    }

    #[test]
    fn test_file_roundtrip_binary() {
        let mut w = ModelWeights::new();
        w.insert("p0", vec![3], vec![1.0, 2.0, 3.0]);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.bin");
        let path_str = path.to_str().unwrap();

        w.save_to_file(path_str).unwrap();
        let loaded = ModelWeights::load_from_file(path_str).unwrap();
        let p0 = loaded.get("p0").unwrap();
        assert_eq!(p0.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_param_entry_to_tensor() {
        let entry = ParamEntry {
            name: "test".into(),
            shape: vec![2, 3],
            data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        };
        let t = entry.to_tensor();
        assert_eq!(t.shape, vec![2, 3]);
        assert_eq!(t.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_binary_invalid_magic() {
        let buf = b"BAD!";
        let result = ModelWeights::load_binary(&mut &buf[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_empty_weights() {
        let w = ModelWeights::new();
        let json = w.to_json();
        let loaded = ModelWeights::from_json(&json).unwrap();
        assert!(loaded.is_empty());
    }
}
