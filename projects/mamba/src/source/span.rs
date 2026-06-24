use std::fmt;

/// Unique identifier for a source file in the SourceMap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FileId(pub u32);

/// A byte-range span within a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file: FileId, start: u32, end: u32) -> Self {
        Self { file, start, end }
    }

    pub fn dummy() -> Self {
        Self::default()
    }

    pub fn merge(self, other: Span) -> Span {
        debug_assert_eq!(self.file, other.file);
        Span {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A node annotated with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: tick-120 test-coverage — Span::{new,merge,len,is_empty} + Display + Spanned::map
    #[test]
    fn test_span_ops_and_spanned_map() {
        let f = FileId(7);
        let a = Span::new(f, 3, 10);
        assert_eq!(a.len(), 7);
        assert!(!a.is_empty());
        assert_eq!(format!("{}", a), "3..10");

        let empty = Span::new(f, 5, 5);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        // merge: min(start), max(end); commutative
        let b = Span::new(f, 8, 15);
        let m1 = a.merge(b);
        let m2 = b.merge(a);
        assert_eq!(m1, Span::new(f, 3, 15));
        assert_eq!(m1, m2);

        // Span::dummy is zero-file zero-range
        let d = Span::dummy();
        assert_eq!(d.file, FileId::default());
        assert!(d.is_empty());

        // Spanned::map preserves span, transforms node
        let s = Spanned::new(42u32, a);
        let s2 = s.map(|n| n.to_string());
        assert_eq!(s2.node, "42");
        assert_eq!(s2.span, a);
    }
}
