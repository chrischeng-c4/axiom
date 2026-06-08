pub mod span;

pub use span::{FileId, Span, Spanned};

/// A source file loaded into the compiler.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: FileId,
    pub name: String,
    pub source: String,
    line_starts: Vec<u32>,
}

impl SourceFile {
    pub fn new(id: FileId, name: String, source: String) -> Self {
        let line_starts = std::iter::once(0)
            .chain(source.match_indices('\n').map(|(i, _)| (i + 1) as u32))
            .collect();
        Self { id, name, source, line_starts }
    }

    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let line = self.line_starts.partition_point(|&s| s <= offset) - 1;
        let col = offset - self.line_starts[line];
        (line as u32 + 1, col + 1)
    }

    pub fn line_text(&self, line: u32) -> &str {
        let idx = (line - 1) as usize;
        if idx >= self.line_starts.len() {
            return "";
        }
        let start = self.line_starts[idx] as usize;
        let end = self.line_starts.get(idx + 1)
            .map(|&e| e as usize)
            .unwrap_or(self.source.len());
        &self.source[start..end].trim_end_matches('\n')
    }
}

/// Collection of all source files.
#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, name: String, source: String) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile::new(id, name, source));
        id
    }

    pub fn get_file(&self, id: FileId) -> &SourceFile {
        &self.files[id.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: tick-124 test-coverage — SourceFile::{new,line_col,line_text} + SourceMap round-trip
    #[test]
    fn test_source_file_line_col_and_source_map_round_trip() {
        // line_starts for "a\nbb\nccc" → [0, 2, 5] (after '\n' at offsets 1 and 4)
        let f = SourceFile::new(FileId(0), "t.py".into(), "a\nbb\nccc".into());

        // 1-based (line, col)
        assert_eq!(f.line_col(0), (1, 1));  // 'a'
        assert_eq!(f.line_col(2), (2, 1));  // 'b' at start of line 2
        assert_eq!(f.line_col(3), (2, 2));  // 'b' mid-line-2
        assert_eq!(f.line_col(5), (3, 1));  // 'c' at start of line 3

        // line_text trims trailing newline
        assert_eq!(f.line_text(1), "a");
        assert_eq!(f.line_text(2), "bb");
        assert_eq!(f.line_text(3), "ccc");

        // out-of-range line returns ""
        assert_eq!(f.line_text(99), "");

        // SourceMap round-trip
        let mut map = SourceMap::new();
        let id_a = map.add_file("a.py".into(), "x=1".into());
        let id_b = map.add_file("b.py".into(), "y=2".into());
        assert_eq!(id_a, FileId(0));
        assert_eq!(id_b, FileId(1));
        assert_eq!(map.get_file(id_a).name, "a.py");
        assert_eq!(map.get_file(id_b).source, "y=2");
    }
}
