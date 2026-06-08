// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! Source map generation with VLQ encoding.
//!
//! Generates V3 source maps (.map files) for bundled output.
//! Supports external file emission and inline data URLs.

use std::path::Path;

/// A generated source map.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// Original source file path.
    pub source: String,
    /// Source map JSON content.
    pub json: String,
}

/// Source map configuration.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceMapMode {
    /// No source maps.
    None,
    /// External .map file with //# sourceMappingURL comment.
    External,
    /// Inline data URL in the bundle.
    Inline,
    /// External .map file without the URL comment (hidden).
    Hidden,
}

/// Generate a V3 source map for a bundle.
///
/// `output_file` is the name of the output JS file (e.g. "main.abc123.js").
/// `sources` maps original file paths to their source content.
/// `output_code` is the final bundled code.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn generate_source_map(
    output_file: &str,
    sources: &[(String, String)],
    output_code: &str,
) -> SourceMap {
    let source_names: Vec<&str> = sources.iter().map(|(name, _)| name.as_str()).collect();
    let source_contents: Vec<&str> = sources
        .iter()
        .map(|(_, content)| content.as_str())
        .collect();

    // Build simple line-to-line mappings
    let mappings = build_mappings(output_code, sources.len());

    // Escape JSON strings
    let sources_json: Vec<String> = source_names
        .iter()
        .map(|s| format!("\"{}\"", escape_json(s)))
        .collect();
    let contents_json: Vec<String> = source_contents
        .iter()
        .map(|s| format!("\"{}\"", escape_json(s)))
        .collect();

    let json = format!(
        r#"{{"version":3,"file":"{}","sources":[{}],"sourcesContent":[{}],"mappings":"{}"}}"#,
        escape_json(output_file),
        sources_json.join(","),
        contents_json.join(","),
        mappings,
    );

    SourceMap {
        source: output_file.to_string(),
        json,
    }
}

/// Append source map reference to code.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn append_source_map_url(code: &str, map_filename: &str) -> String {
    format!("{}\n//# sourceMappingURL={}\n", code, map_filename)
}

/// Generate inline source map as data URL.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn inline_source_map(code: &str, map_json: &str) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(map_json.as_bytes());
    format!(
        "{}\n//# sourceMappingURL=data:application/json;base64,{}\n",
        code, encoded
    )
}

/// Write external source map file.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn write_external_map(
    output_dir: &Path,
    map_filename: &str,
    map_json: &str,
) -> std::io::Result<()> {
    std::fs::write(output_dir.join(map_filename), map_json)
}

/// Build VLQ-encoded mappings string.
///
/// Simple strategy: each output line maps to its corresponding source line
/// in source index 0. For bundled output, this provides a reasonable mapping.
fn build_mappings(output_code: &str, source_count: usize) -> String {
    if source_count == 0 {
        return String::new();
    }

    let lines: Vec<&str> = output_code.lines().collect();
    let mut segments: Vec<String> = Vec::new();

    let mut prev_source = 0i64;
    let mut prev_source_line = 0i64;
    let mut prev_source_col = 0i64;
    let mut prev_gen_col = 0i64;

    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            segments.push(String::new());
            continue;
        }

        // Map each line start: gen_col=0, source=0, source_line=i, source_col=0
        let gen_col = 0i64 - prev_gen_col;
        let source = 0i64 - prev_source;
        let source_line = i as i64 - prev_source_line;
        let source_col = 0i64 - prev_source_col;

        let mut seg = String::new();
        vlq_encode(&mut seg, gen_col);
        vlq_encode(&mut seg, source);
        vlq_encode(&mut seg, source_line);
        vlq_encode(&mut seg, source_col);

        prev_gen_col = 0;
        prev_source = 0;
        prev_source_line = i as i64;
        prev_source_col = 0;

        segments.push(seg);
    }

    segments.join(";")
}

/// Encode a single VLQ value and append to output.
fn vlq_encode(out: &mut String, value: i64) {
    const VLQ_BASE_SHIFT: u32 = 5;
    const VLQ_BASE: i64 = 1 << VLQ_BASE_SHIFT; // 32
    const VLQ_BASE_MASK: i64 = VLQ_BASE - 1; // 31
    const VLQ_CONTINUATION_BIT: i64 = VLQ_BASE; // 32

    let mut vlq = if value < 0 {
        ((-value) << 1) + 1
    } else {
        value << 1
    };

    loop {
        let mut digit = vlq & VLQ_BASE_MASK;
        vlq >>= VLQ_BASE_SHIFT;
        if vlq > 0 {
            digit |= VLQ_CONTINUATION_BIT;
        }
        out.push(vlq_char(digit as u8));
        if vlq == 0 {
            break;
        }
    }
}

/// Map a 6-bit value to a Base64 character.
fn vlq_char(value: u8) -> char {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    CHARS[value as usize] as char
}

/// A single decoded mapping entry.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct MappingEntry {
    /// Generated line (0-based).
    pub gen_line: usize,
    /// Generated column (0-based).
    pub gen_col: usize,
    /// Source index.
    pub source: usize,
    /// Original line (0-based).
    pub orig_line: usize,
    /// Original column (0-based).
    pub orig_col: usize,
}

/// Decode a VLQ-encoded mappings string into a list of mapping entries.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn decode_mappings(mappings: &str) -> Vec<MappingEntry> {
    let mut entries = Vec::new();
    let mut gen_line = 0usize;
    #[allow(unused_assignments)]
    let mut prev_gen_col = 0i64;
    let mut prev_source = 0i64;
    let mut prev_orig_line = 0i64;
    let mut prev_orig_col = 0i64;

    for line_str in mappings.split(';') {
        if !line_str.is_empty() {
            prev_gen_col = 0; // gen_col resets per line
            for segment in line_str.split(',') {
                let values = vlq_decode(segment);
                if values.len() >= 4 {
                    prev_gen_col += values[0];
                    prev_source += values[1];
                    prev_orig_line += values[2];
                    prev_orig_col += values[3];

                    entries.push(MappingEntry {
                        gen_line,
                        gen_col: prev_gen_col as usize,
                        source: prev_source as usize,
                        orig_line: prev_orig_line as usize,
                        orig_col: prev_orig_col as usize,
                    });
                }
            }
        }
        gen_line += 1;
    }

    entries
}

/// Encode mapping entries back into a VLQ mappings string.
fn encode_mappings(entries: &[MappingEntry], max_gen_line: usize) -> String {
    let mut segments: Vec<String> = Vec::new();
    #[allow(unused_assignments)]
    let mut prev_gen_col = 0i64;
    let mut prev_source = 0i64;
    let mut prev_orig_line = 0i64;
    let mut prev_orig_col = 0i64;

    let total_lines = if entries.is_empty() {
        max_gen_line + 1
    } else {
        let max_entry_line = entries.iter().map(|e| e.gen_line).max().unwrap_or(0);
        std::cmp::max(max_gen_line + 1, max_entry_line + 1)
    };

    for line in 0..total_lines {
        let line_entries: Vec<&MappingEntry> =
            entries.iter().filter(|e| e.gen_line == line).collect();

        if line_entries.is_empty() {
            segments.push(String::new());
        } else {
            prev_gen_col = 0; // reset per line
            let mut line_segs: Vec<String> = Vec::new();

            for entry in line_entries {
                let mut seg = String::new();
                vlq_encode(&mut seg, entry.gen_col as i64 - prev_gen_col);
                vlq_encode(&mut seg, entry.source as i64 - prev_source);
                vlq_encode(&mut seg, entry.orig_line as i64 - prev_orig_line);
                vlq_encode(&mut seg, entry.orig_col as i64 - prev_orig_col);

                prev_gen_col = entry.gen_col as i64;
                prev_source = entry.source as i64;
                prev_orig_line = entry.orig_line as i64;
                prev_orig_col = entry.orig_col as i64;

                line_segs.push(seg);
            }
            segments.push(line_segs.join(","));
        }
    }

    segments.join(";")
}

/// Decode a single VLQ segment into a vector of values.
fn vlq_decode(segment: &str) -> Vec<i64> {
    const VLQ_BASE_SHIFT: u32 = 5;
    const VLQ_BASE: i64 = 1 << VLQ_BASE_SHIFT;
    const VLQ_BASE_MASK: i64 = VLQ_BASE - 1;
    const VLQ_CONTINUATION_BIT: i64 = VLQ_BASE;

    let mut values = Vec::new();
    let mut shift = 0u32;
    let mut result: i64 = 0;

    for ch in segment.chars() {
        let digit = vlq_decode_char(ch);
        if digit < 0 {
            continue;
        }
        let digit = digit as i64;
        result += (digit & VLQ_BASE_MASK) << shift;
        shift += VLQ_BASE_SHIFT;

        if digit & VLQ_CONTINUATION_BIT == 0 {
            // Final digit
            let is_negative = (result & 1) == 1;
            let value = result >> 1;
            values.push(if is_negative { -value } else { value });
            result = 0;
            shift = 0;
        }
    }

    values
}

/// Decode a Base64 character to its 6-bit value, or -1 if invalid.
fn vlq_decode_char(ch: char) -> i8 {
    match ch {
        'A'..='Z' => (ch as i8) - ('A' as i8),
        'a'..='z' => (ch as i8) - ('a' as i8) + 26,
        '0'..='9' => (ch as i8) - ('0' as i8) + 52,
        '+' => 62,
        '/' => 63,
        _ => -1,
    }
}

/// Format the warning emitted when `compose_source_maps` cannot parse one
/// of its two map inputs. Names which side (`"input"` or `"output"`) failed,
/// preserves the underlying serde error verbatim, and carries `GH #3536` so
/// users grepping logs for "source map broken" / "devtools points at bundle"
/// can find this line. Extracted for unit-test pinning.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn format_sourcemap_parse_warn(which: &str, err: &serde_json::Error) -> String {
    format!(
        "GH #3536 compose_source_maps failed to parse {which} map JSON: {err}; source-map composition will be SKIPPED and the output map will be returned unchanged. Browser devtools will point at the intermediate/bundled file instead of the original source. Check that the {which} .map file is valid JSON (look for truncation, BOM, or a stray non-JSON line)."
    )
}

/// GH #3813 — fallback string used when a source-map JSON field is
/// absent (legitimate degenerate-map case). Named for call-site/test
/// pin-equality.
pub(crate) const SOURCEMAP_FIELD_ABSENT_FALLBACK: &str = "";

/// GH #3813 — warn shown when a source-map JSON field is present but
/// not a string (e.g., `mappings` is a number, `file` is an object).
/// The prior code silently collapsed this onto `""` together with the
/// legitimate-absent case, producing a degenerate composed map with no
/// breadcrumb pointing at the malformed input.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn format_sourcemap_field_wrong_shape_warn(
    which: &str,
    name: &str,
    value: &serde_json::Value,
) -> String {
    let kind = match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    };
    format!(
        "gh3813 compose_source_maps received {which} map with field {name:?} \
         present but not a string (kind={kind}); routing through the stringified \
         lossy form so the composed map carries a visible breadcrumb instead of \
         collapsing onto an empty string. Check that the {which} .map field {name:?} \
         is a JSON string."
    )
}

/// GH #3813 — coerce a source-map JSON string field into a `Cow<str>`.
///
/// - Field absent: silent `Cow::Borrowed("")` (preserves historical
///   degenerate-map behaviour).
/// - Field present + string: silent `Cow::Borrowed(s)`.
/// - Field present + wrong-shape: emit a `tracing::warn!` and return
///   `Cow::Owned(stringified)` so the breadcrumb is visible downstream.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn coerce_sourcemap_string_field_or_warn<'a>(
    map: &'a serde_json::Value,
    which: &str,
    name: &str,
) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    match map.get(name) {
        None => Cow::Borrowed(SOURCEMAP_FIELD_ABSENT_FALLBACK),
        Some(v) => match v.as_str() {
            Some(s) => Cow::Borrowed(s),
            None => {
                tracing::warn!(
                    target: "jet::bundler::sourcemap",
                    which = which,
                    field = name,
                    "{}",
                    format_sourcemap_field_wrong_shape_warn(which, name, v)
                );
                Cow::Owned(v.to_string())
            }
        },
    }
}

/// Compose (chain) two source maps: an input map and an output map.
///
/// The input map maps intermediate positions to original positions
/// (e.g., TS -> JS). The output map maps final positions to intermediate
/// positions (e.g., bundled/minified -> JS). The result maps final
/// positions directly to original positions.
///
/// `sourcesContent` from the input map is preserved in the result.
///
/// GH #3536 — prior `.unwrap_or(serde_json::Value::Null)` calls dropped
/// parse errors and silently fell back to the unchanged output map. We
/// now emit a structured warn per side that failed so a developer chasing
/// "devtools points at the bundle, not the source" can grep logs and find
/// the offending unparseable .map.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn compose_source_maps(input_map_json: &str, output_map_json: &str) -> String {
    // Parse both maps
    let input_map: serde_json::Value = match serde_json::from_str(input_map_json) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(
                target: "jet::bundler::sourcemap",
                side = "input",
                error = %err,
                "{}",
                format_sourcemap_parse_warn("input", &err)
            );
            serde_json::Value::Null
        }
    };
    let output_map: serde_json::Value = match serde_json::from_str(output_map_json) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(
                target: "jet::bundler::sourcemap",
                side = "output",
                error = %err,
                "{}",
                format_sourcemap_parse_warn("output", &err)
            );
            serde_json::Value::Null
        }
    };

    if input_map.is_null() || output_map.is_null() {
        return output_map_json.to_string();
    }

    let input_mappings_cow = coerce_sourcemap_string_field_or_warn(&input_map, "input", "mappings");
    let output_mappings_cow =
        coerce_sourcemap_string_field_or_warn(&output_map, "output", "mappings");

    let input_entries = decode_mappings(input_mappings_cow.as_ref());
    let output_entries = decode_mappings(output_mappings_cow.as_ref());

    // For each output entry, look up the intermediate position in the input map
    // to find the original position
    let mut composed_entries: Vec<MappingEntry> = Vec::new();

    for out_entry in &output_entries {
        // out_entry maps: final (gen_line, gen_col) -> intermediate (orig_line, orig_col)
        // We need to find an input entry that maps intermediate -> original
        let intermediate_line = out_entry.orig_line;
        let intermediate_col = out_entry.orig_col;

        // Find the best matching input entry for this intermediate position
        if let Some(input_entry) =
            find_mapping_for(&input_entries, intermediate_line, intermediate_col)
        {
            composed_entries.push(MappingEntry {
                gen_line: out_entry.gen_line,
                gen_col: out_entry.gen_col,
                source: input_entry.source,
                orig_line: input_entry.orig_line,
                orig_col: input_entry.orig_col,
            });
        } else {
            // No input mapping found — keep the output mapping as-is
            composed_entries.push(out_entry.clone());
        }
    }

    // Encode the composed mappings
    let max_gen_line = composed_entries
        .iter()
        .map(|e| e.gen_line)
        .max()
        .unwrap_or(0);
    let composed_mappings = encode_mappings(&composed_entries, max_gen_line);

    // Build the result using the input map's sources/sourcesContent
    // and the output map's file
    let file = output_map
        .get("file")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let sources = input_map
        .get("sources")
        .cloned()
        .unwrap_or(serde_json::Value::Array(Vec::new()));
    let sources_content = input_map
        .get("sourcesContent")
        .cloned()
        .unwrap_or(serde_json::Value::Array(Vec::new()));

    let result = serde_json::json!({
        "version": 3,
        "file": file,
        "sources": sources,
        "sourcesContent": sources_content,
        "mappings": composed_mappings,
    });

    serde_json::to_string(&result).unwrap_or_else(|_| output_map_json.to_string())
}

/// Find the best matching input mapping entry for a given line/column.
///
/// Uses binary-search-like approach: finds the entry on the target line
/// whose column is closest to (but not exceeding) the target column.
fn find_mapping_for(
    entries: &[MappingEntry],
    target_line: usize,
    target_col: usize,
) -> Option<&MappingEntry> {
    // Filter to entries on the target line
    let line_entries: Vec<&MappingEntry> = entries
        .iter()
        .filter(|e| e.gen_line == target_line)
        .collect();

    if line_entries.is_empty() {
        return None;
    }

    // Find the entry with the largest gen_col <= target_col
    let mut best: Option<&MappingEntry> = None;
    for entry in &line_entries {
        if entry.gen_col <= target_col {
            match best {
                None => best = Some(entry),
                Some(b) if entry.gen_col > b.gen_col => best = Some(entry),
                _ => {}
            }
        }
    }

    // If no exact match, return the first entry on the line
    best.or_else(|| line_entries.first().copied())
}

/// Escape a string for JSON output.
fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlq_encode_zero() {
        let mut out = String::new();
        vlq_encode(&mut out, 0);
        assert_eq!(out, "A"); // 0 → 'A'
    }

    #[test]
    fn test_vlq_encode_positive() {
        let mut out = String::new();
        vlq_encode(&mut out, 1);
        assert_eq!(out, "C"); // 1 → shift left → 2 → 'C'
    }

    #[test]
    fn test_vlq_encode_negative() {
        let mut out = String::new();
        vlq_encode(&mut out, -1);
        assert_eq!(out, "D"); // -1 → (1<<1)+1 = 3 → 'D'
    }

    #[test]
    fn test_generate_source_map_structure() {
        let sources = vec![("app.js".to_string(), "const x = 1;".to_string())];
        let output_code = "const x = 1;";
        let map = generate_source_map("bundle.js", &sources, output_code);

        assert!(map.json.contains("\"version\":3"));
        assert!(map.json.contains("\"file\":\"bundle.js\""));
        assert!(map.json.contains("\"sources\":[\"app.js\"]"));
        assert!(map.json.contains("\"mappings\":"));
    }

    #[test]
    fn test_append_source_map_url() {
        let code = "const x = 1;";
        let result = append_source_map_url(code, "bundle.js.map");
        assert!(result.contains("//# sourceMappingURL=bundle.js.map"));
    }

    #[test]
    fn test_inline_source_map() {
        let code = "const x = 1;";
        let map_json = r#"{"version":3}"#;
        let result = inline_source_map(code, map_json);
        assert!(result.contains("data:application/json;base64,"));
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("he\"llo"), "he\\\"llo");
        assert_eq!(escape_json("line\nbreak"), "line\\nbreak");
    }

    #[test]
    fn test_empty_sources() {
        let sources: Vec<(String, String)> = vec![];
        let map = generate_source_map("out.js", &sources, "");
        assert!(map.json.contains("\"sources\":[]"));
    }

    // ──────────────────────────────────────────────────────────────────
    // VLQ decode tests
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_vlq_decode_a() {
        let values = vlq_decode("A");
        assert_eq!(values, vec![0]);
    }

    #[test]
    fn test_vlq_decode_c() {
        let values = vlq_decode("C");
        assert_eq!(values, vec![1]);
    }

    #[test]
    fn test_vlq_decode_d() {
        let values = vlq_decode("D");
        assert_eq!(values, vec![-1]);
    }

    #[test]
    fn test_vlq_roundtrip() {
        for v in [-100, -10, -1, 0, 1, 10, 100] {
            let mut encoded = String::new();
            vlq_encode(&mut encoded, v);
            let decoded = vlq_decode(&encoded);
            assert_eq!(decoded, vec![v], "VLQ roundtrip failed for {}", v);
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Source map chaining tests (R11 / T16)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_compose_source_maps_chaining() {
        // T16: Input map: line 5 -> original line 10
        //       Output map: line 3 -> bundled line 5
        //       Result: line 3 -> original line 10

        // Build input map: maps generated line 5 to original line 10
        let input_entries = vec![MappingEntry {
            gen_line: 5,
            gen_col: 0,
            source: 0,
            orig_line: 10,
            orig_col: 0,
        }];
        let input_mappings = encode_mappings(&input_entries, 5);

        let input_map = serde_json::json!({
            "version": 3,
            "file": "intermediate.js",
            "sources": ["original.ts"],
            "sourcesContent": ["// original TypeScript source"],
            "mappings": input_mappings,
        });

        // Build output map: maps final line 3 to intermediate line 5
        let output_entries = vec![MappingEntry {
            gen_line: 3,
            gen_col: 0,
            source: 0,
            orig_line: 5,
            orig_col: 0,
        }];
        let output_mappings = encode_mappings(&output_entries, 3);

        let output_map = serde_json::json!({
            "version": 3,
            "file": "final.js",
            "sources": ["intermediate.js"],
            "sourcesContent": ["// intermediate JS"],
            "mappings": output_mappings,
        });

        let result_json = compose_source_maps(
            &serde_json::to_string(&input_map).unwrap(),
            &serde_json::to_string(&output_map).unwrap(),
        );

        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();

        // The result should map to the original source
        assert_eq!(result["version"], 3);
        assert_eq!(result["sources"][0], "original.ts");

        // Verify sourcesContent is preserved from input map
        assert_eq!(result["sourcesContent"][0], "// original TypeScript source");

        // Decode the composed mappings and verify the chain
        let composed_mappings = result["mappings"].as_str().unwrap();
        let composed_entries = decode_mappings(composed_mappings);

        // Find the entry for gen_line 3
        let entry = composed_entries
            .iter()
            .find(|e| e.gen_line == 3)
            .expect("Should have mapping for gen_line 3");

        assert_eq!(
            entry.orig_line, 10,
            "Chained mapping should map line 3 -> original line 10, got line {}",
            entry.orig_line
        );
    }

    #[test]
    fn test_compose_source_maps_preserves_sources() {
        let input_map = serde_json::json!({
            "version": 3,
            "file": "app.js",
            "sources": ["app.ts", "utils.ts"],
            "sourcesContent": ["// app.ts source", "// utils.ts source"],
            "mappings": "AAAA",
        });

        let output_map = serde_json::json!({
            "version": 3,
            "file": "bundle.js",
            "sources": ["app.js"],
            "sourcesContent": ["// app.js content"],
            "mappings": "AAAA",
        });

        let result_json = compose_source_maps(
            &serde_json::to_string(&input_map).unwrap(),
            &serde_json::to_string(&output_map).unwrap(),
        );

        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
        // Sources should come from the input map (original)
        assert_eq!(result["sources"][0], "app.ts");
        assert_eq!(result["sources"][1], "utils.ts");
        assert_eq!(result["sourcesContent"][0], "// app.ts source");
    }

    // ──────────────────────────────────────────────────────────────────
    // GH #3536 — silent JSON parse swallow surfaces a structured warn
    // ──────────────────────────────────────────────────────────────────

    fn make_serde_err() -> serde_json::Error {
        serde_json::from_str::<serde_json::Value>("{not json").unwrap_err()
    }

    #[test]
    fn gh3536_format_sourcemap_parse_warn_names_side_and_error_and_issue() {
        let err = make_serde_err();
        let msg = format_sourcemap_parse_warn("input", &err);
        assert!(
            msg.contains("input"),
            "warning must name which side failed (input/output): {msg}"
        );
        assert!(
            msg.contains("GH #3536"),
            "warning must carry the GH #3536 tag so users can grep their logs: {msg}"
        );
        assert!(
            msg.contains("composition will be SKIPPED") || msg.contains("SKIPPED"),
            "warning must announce that composition is skipped: {msg}"
        );
    }

    #[test]
    fn gh3536_format_sourcemap_parse_warn_hints_at_devtools_symptom() {
        let err = make_serde_err();
        let msg = format_sourcemap_parse_warn("output", &err);
        // Pin the keywords a user grepping for "devtools points at bundle" hits.
        assert!(
            msg.contains("devtools") || msg.contains("bundle") || msg.contains("original"),
            "warning must mention the user-visible symptom (devtools/bundle/original): {msg}"
        );
        assert!(
            msg.contains("output"),
            "warning must name which side failed: {msg}"
        );
    }

    #[test]
    fn gh3536_compose_source_maps_returns_output_unchanged_on_bad_input_json() {
        // End-to-end pin: when the input map is unparseable, compose_source_maps
        // falls back to returning the output map JSON verbatim. The contract
        // this PR locks is: the fallback path is taken *and* a warn line is
        // emitted (the warn line is enforced via the helper-shape tests above
        // since tracing capture in unit tests is brittle).
        let bad_input = "{not json";
        let good_output = r#"{"version":3,"file":"out.js","sources":["app.js"],"sourcesContent":["// app"],"mappings":""}"#;
        let result = compose_source_maps(bad_input, good_output);
        assert_eq!(
            result, good_output,
            "with unparseable input map, compose must return the output map verbatim"
        );
    }

    #[test]
    fn gh3536_compose_source_maps_returns_output_unchanged_on_bad_output_json() {
        let good_input = r#"{"version":3,"file":"i.js","sources":["a.ts"],"sourcesContent":["// ts"],"mappings":""}"#;
        let bad_output = "not json at all";
        let result = compose_source_maps(good_input, bad_output);
        assert_eq!(
            result, bad_output,
            "with unparseable output map, compose must return the output map verbatim (the fallback path)"
        );
    }

    #[test]
    fn test_decode_mappings_empty() {
        let entries = decode_mappings("");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_decode_encode_roundtrip() {
        let original = vec![
            MappingEntry {
                gen_line: 0,
                gen_col: 0,
                source: 0,
                orig_line: 0,
                orig_col: 0,
            },
            MappingEntry {
                gen_line: 1,
                gen_col: 4,
                source: 0,
                orig_line: 2,
                orig_col: 8,
            },
        ];

        let encoded = encode_mappings(&original, 1);
        let decoded = decode_mappings(&encoded);

        assert_eq!(decoded.len(), original.len());
        for (d, o) in decoded.iter().zip(original.iter()) {
            assert_eq!(d.gen_line, o.gen_line);
            assert_eq!(d.gen_col, o.gen_col);
            assert_eq!(d.source, o.source);
            assert_eq!(d.orig_line, o.orig_line);
            assert_eq!(d.orig_col, o.orig_col);
        }
    }
}

#[cfg(test)]
mod gh3813_sourcemap_field_wrong_shape_warn_tests {
    use super::*;

    fn map_with(fields: &[(&str, serde_json::Value)]) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        for (k, v) in fields {
            obj.insert(k.to_string(), v.clone());
        }
        serde_json::Value::Object(obj)
    }

    #[test]
    fn string_field_borrows_silently() {
        let m = map_with(&[("mappings", serde_json::Value::String("AAAA".into()))]);
        let cow = coerce_sourcemap_string_field_or_warn(&m, "input", "mappings");
        assert_eq!(cow.as_ref(), "AAAA");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn absent_field_falls_back_to_named_constant_silently() {
        let m = map_with(&[]);
        let cow = coerce_sourcemap_string_field_or_warn(&m, "input", "mappings");
        assert_eq!(cow.as_ref(), SOURCEMAP_FIELD_ABSENT_FALLBACK);
        assert_eq!(cow.as_ref(), "");
    }

    #[test]
    fn wrong_shape_number_warns_and_returns_owned_lossy_form() {
        let m = map_with(&[("mappings", serde_json::json!(42))]);
        let cow = coerce_sourcemap_string_field_or_warn(&m, "input", "mappings");
        assert!(
            !cow.as_ref().is_empty(),
            "wrong-shape must not collapse to empty"
        );
        assert_eq!(cow.as_ref(), "42");
        assert!(matches!(cow, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn wrong_shape_object_warns_and_returns_owned_lossy_form() {
        let m = map_with(&[("file", serde_json::json!({"nested": 1}))]);
        let cow = coerce_sourcemap_string_field_or_warn(&m, "output", "file");
        assert!(!cow.as_ref().is_empty());
        assert!(cow.as_ref().contains("nested"));
        assert!(matches!(cow, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn wrong_shape_array_warns_and_returns_owned_lossy_form() {
        let m = map_with(&[("mappings", serde_json::json!([1, 2, 3]))]);
        let cow = coerce_sourcemap_string_field_or_warn(&m, "input", "mappings");
        assert!(!cow.as_ref().is_empty());
        assert!(matches!(cow, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn helpers_pinned_for_discoverability() {
        let _: fn(&str, &str, &serde_json::Value) -> String =
            format_sourcemap_field_wrong_shape_warn;
        let _: for<'a> fn(&'a serde_json::Value, &str, &str) -> std::borrow::Cow<'a, str> =
            coerce_sourcemap_string_field_or_warn;
        assert_eq!(SOURCEMAP_FIELD_ABSENT_FALLBACK, "");
    }

    #[test]
    fn warn_string_carries_gh3813_tag_and_names_field() {
        let msg =
            format_sourcemap_field_wrong_shape_warn("input", "mappings", &serde_json::json!(42));
        assert!(msg.contains("gh3813"), "warn lacks tag: {msg}");
        assert!(msg.contains("mappings"), "warn lacks field name: {msg}");
        assert!(msg.contains("input"), "warn lacks side name: {msg}");
        assert!(msg.contains("number"), "warn lacks kind: {msg}");
    }

    #[test]
    fn warn_distinct_for_different_kinds() {
        let num = format_sourcemap_field_wrong_shape_warn("input", "f", &serde_json::json!(1));
        let obj =
            format_sourcemap_field_wrong_shape_warn("input", "f", &serde_json::json!({"x": 1}));
        let arr = format_sourcemap_field_wrong_shape_warn("input", "f", &serde_json::json!([1]));
        assert_ne!(num, obj);
        assert_ne!(num, arr);
        assert_ne!(obj, arr);
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let msg =
            format_sourcemap_field_wrong_shape_warn("input", "mappings", &serde_json::json!(42));
        for prior in [
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
            "gh3805", "gh3807", "gh3809", "gh3811",
        ] {
            assert!(!msg.contains(prior), "warn collides with {prior}: {msg}");
        }
    }

    #[test]
    fn null_field_takes_wrong_shape_branch_not_absent_branch() {
        // serde_json::Value::Null is structurally present but not a string;
        // this is wrong-shape, not absent.
        let m = map_with(&[("mappings", serde_json::Value::Null)]);
        let cow = coerce_sourcemap_string_field_or_warn(&m, "input", "mappings");
        assert_eq!(cow.as_ref(), "null");
        assert!(matches!(cow, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn compose_source_maps_handles_wrong_shape_mappings_without_panic() {
        let input = r#"{"version":3,"file":"a.js","sources":["a.ts"],"mappings":42}"#;
        let output = r#"{"version":3,"file":"a.min.js","sources":["a.js"],"mappings":"AAAA"}"#;
        let result = compose_source_maps(input, output);
        assert!(
            !result.is_empty(),
            "must produce some output even with wrong-shape mappings"
        );
    }
}
// CODEGEN-END
