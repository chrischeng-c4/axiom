//! Query execution for the 6 search modes.
//!
//! Each mode operates on a [`SearchIndex`] and returns `SearchResult`.

use std::collections::{HashSet, VecDeque};
use std::path::Path;

use crate::search::index::{IndexSymbolKind, SearchIndex, SymbolEntry};
use crate::type_inference::{
    CallDirection, MatchContext, MatchKind, SearchMatch, SearchQuery, SearchResult, SearchScope,
    SearchStats, Span,
};

// -- Shared helpers --

fn entry_in_scope(e: &SymbolEntry, scope: &SearchScope) -> bool {
    match scope {
        SearchScope::CurrentFile(f) => e.file == *f,
        SearchScope::Files(fs) => fs.contains(&e.file),
        SearchScope::Project | SearchScope::ProjectWithDeps => true,
    }
}

fn to_match_kind(kind: IndexSymbolKind) -> MatchKind {
    match kind {
        IndexSymbolKind::Function => MatchKind::FunctionDef,
        IndexSymbolKind::Class | IndexSymbolKind::Struct => MatchKind::ClassDef,
        IndexSymbolKind::Variable | IndexSymbolKind::Const | IndexSymbolKind::Static => {
            MatchKind::VariableAssignment
        }
        IndexSymbolKind::Import => MatchKind::Import,
        IndexSymbolKind::Interface | IndexSymbolKind::Trait => MatchKind::ClassDef,
        _ => MatchKind::FunctionDef,
    }
}

fn entry_to_match(e: &SymbolEntry, score: f64) -> SearchMatch {
    SearchMatch {
        file: e.file.clone(),
        span: Span {
            start: 0,
            end: 0,
            start_line: e.position.start_line as usize,
            start_col: e.position.start_col as usize,
            end_line: e.position.end_line as usize,
            end_col: e.position.end_col as usize,
        },
        symbol: None,
        kind: to_match_kind(e.kind),
        score,
        context: None,
    }
}

fn make_stats(files: &HashSet<std::path::PathBuf>, start: std::time::Instant) -> SearchStats {
    SearchStats {
        files_searched: files.len(),
        time_ms: start.elapsed().as_millis() as u64,
        truncated: false,
    }
}

// -- 1. ByTypeSignature --

fn parse_type_pattern(p: &str) -> (Vec<String>, Option<String>) {
    let p = p.trim();
    if let Some((params, ret)) = p.split_once("->") {
        let ps = parse_params(params.trim());
        let r = ret.trim().to_lowercase();
        (ps, if r.is_empty() { None } else { Some(r) })
    } else {
        (parse_params(p), None)
    }
}

fn parse_params(s: &str) -> Vec<String> {
    let s = s.trim().trim_start_matches('(').trim_end_matches(')');
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .collect()
}

pub fn search_by_type_signature(index: &SearchIndex, q: &SearchQuery, pat: &str) -> SearchResult {
    let (qp, qr) = parse_type_pattern(pat);
    let start = std::time::Instant::now();
    let mut matches = Vec::new();
    let mut fs = HashSet::new();
    for e in index.query_by_type(pat) {
        if !entry_in_scope(e, &q.scope) {
            continue;
        }
        fs.insert(e.file.clone());
        let sig = match &e.type_signature {
            Some(s) => s.to_lowercase(),
            None => continue,
        };
        let score = sig_score(&qp, &qr, &sig);
        if score > 0.0 {
            let mut m = entry_to_match(e, score);
            m.symbol = Some(sig);
            matches.push(m);
        }
    }
    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(q.max_results);
    let total = matches.len();
    SearchResult {
        matches,
        total_count: total,
        stats: make_stats(&fs, start),
    }
}

fn sig_score(qp: &[String], qr: &Option<String>, sig: &str) -> f64 {
    let (sp, sr) = parse_type_pattern(sig);
    let mut score = 0.0_f64;
    let mut n = 1_usize;
    if qp.len() == sp.len() {
        score += 1.0;
    } else if !qp.is_empty() {
        return 0.0;
    }
    for (a, b) in qp.iter().zip(sp.iter()) {
        if a == b {
            score += 1.0;
        } else if b.contains(a.as_str()) {
            score += 0.5;
        }
        n += 1;
    }
    if let Some(ref qret) = qr {
        if let Some(ref sret) = sr {
            if qret == sret {
                score += 1.5;
            } else if sret.contains(qret.as_str()) {
                score += 0.7;
            }
        }
        n += 1;
    }
    if n == 0 {
        0.0
    } else {
        score / n as f64
    }
}

// -- 2. CallHierarchy --

/// Lightweight call graph for BFS traversal.
#[derive(Debug, Default)]
pub struct CallGraphIndex {
    pub calls: std::collections::HashMap<String, Vec<String>>,
    pub called_by: std::collections::HashMap<String, Vec<String>>,
}

pub fn search_call_hierarchy(
    index: &SearchIndex,
    cg: &CallGraphIndex,
    q: &SearchQuery,
    symbol: &str,
    _file: &Path,
    dir: CallDirection,
    max_depth: usize,
) -> SearchResult {
    let start = std::time::Instant::now();
    let mut matches = Vec::new();
    let mut visited = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    queue.push_back((symbol.to_string(), 0));
    while let Some((cur, depth)) = queue.pop_front() {
        if !visited.insert(cur.clone()) || depth > max_depth {
            continue;
        }
        if matches.len() >= q.max_results {
            break;
        }
        let nbrs = match dir {
            CallDirection::Callers => cg.called_by.get(&cur),
            CallDirection::Callees => cg.calls.get(&cur),
        };
        if let Some(names) = nbrs {
            for name in names {
                for e in index.query_by_name(name) {
                    if !entry_in_scope(e, &q.scope) {
                        continue;
                    }
                    let mut m = entry_to_match(e, 1.0 - depth as f64 * 0.1);
                    m.symbol = Some(name.clone());
                    m.kind = MatchKind::Call;
                    matches.push(m);
                }
                queue.push_back((name.clone(), depth + 1));
            }
        }
    }
    matches.truncate(q.max_results);
    let total = matches.len();
    SearchResult {
        matches,
        total_count: total,
        stats: make_stats(&HashSet::new(), start),
    }
}

// -- 3. Implementations --

pub fn search_implementations(index: &SearchIndex, q: &SearchQuery, proto: &str) -> SearchResult {
    let start = std::time::Instant::now();
    let mut matches = Vec::new();
    let mut fs = HashSet::new();
    for e in &index.query_by_name(proto) {
        if !entry_in_scope(e, &q.scope) {
            continue;
        }
        fs.insert(e.file.clone());
        if matches!(
            e.kind,
            IndexSymbolKind::Impl | IndexSymbolKind::Class | IndexSymbolKind::Struct
        ) {
            let mut m = entry_to_match(e, 1.0);
            m.symbol = Some(proto.to_string());
            matches.push(m);
        }
    }
    for e in index.query_by_type(proto) {
        if !entry_in_scope(e, &q.scope) {
            continue;
        }
        fs.insert(e.file.clone());
        if matches!(
            e.kind,
            IndexSymbolKind::Impl
                | IndexSymbolKind::Class
                | IndexSymbolKind::Struct
                | IndexSymbolKind::Trait
                | IndexSymbolKind::Interface
        ) {
            let mut m = entry_to_match(e, 0.8);
            m.symbol = Some(proto.to_string());
            matches.push(m);
        }
    }
    matches.truncate(q.max_results);
    let total = matches.len();
    SearchResult {
        matches,
        total_count: total,
        stats: make_stats(&fs, start),
    }
}

// -- 4. Usages --

pub fn search_usages(
    index: &SearchIndex,
    q: &SearchQuery,
    sym: &str,
    _file: &Path,
) -> SearchResult {
    let start = std::time::Instant::now();
    let mut matches = Vec::new();
    let mut fs = HashSet::new();
    for e in index.query_by_name(sym) {
        if !entry_in_scope(e, &q.scope) {
            continue;
        }
        fs.insert(e.file.clone());
        let mut m = entry_to_match(e, 1.0);
        m.symbol = Some(sym.to_string());
        matches.push(m);
    }
    matches.truncate(q.max_results);
    let total = matches.len();
    SearchResult {
        matches,
        total_count: total,
        stats: make_stats(&fs, start),
    }
}

// -- 5. SimilarCode --

pub fn search_similar_code(index: &SearchIndex, q: &SearchQuery, pattern: &str) -> SearchResult {
    let start = std::time::Instant::now();
    let mut matches = Vec::new();
    let mut fs = HashSet::new();
    let pat_lower = pattern.to_lowercase();
    let candidates = index.query_by_name(pattern);
    for e in &candidates {
        if !entry_in_scope(e, &q.scope) {
            continue;
        }
        fs.insert(e.file.clone());
        let hint = e.type_signature.as_deref().unwrap_or("unknown");
        let score = name_similarity(pattern, hint);
        if score > 0.3 {
            let mut m = entry_to_match(e, score);
            m.symbol = Some(hint.to_string());
            matches.push(m);
        }
    }
    if candidates
        .first()
        .and_then(|e| e.type_signature.as_ref())
        .is_some()
    {
        for e in index.query_by_type(pattern) {
            if !entry_in_scope(e, &q.scope) {
                continue;
            }
            fs.insert(e.file.clone());
            let sig = e.type_signature.as_deref().unwrap_or("");
            if sig.to_lowercase().contains(&pat_lower) {
                let mut m = entry_to_match(e, 0.6);
                m.symbol = Some(sig.to_string());
                matches.push(m);
            }
        }
    }
    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(q.max_results);
    let total = matches.len();
    SearchResult {
        matches,
        total_count: total,
        stats: make_stats(&fs, start),
    }
}

fn name_similarity(a: &str, b: &str) -> f64 {
    let (al, bl) = (a.to_lowercase(), b.to_lowercase());
    if al == bl {
        return 1.0;
    }
    if al.is_empty() || bl.is_empty() {
        return 0.0;
    }
    let ac: HashSet<char> = al.chars().collect();
    let bc: HashSet<char> = bl.chars().collect();
    let i = ac.intersection(&bc).count();
    let u = ac.union(&bc).count();
    if u == 0 {
        0.0
    } else {
        i as f64 / u as f64
    }
}

// -- 6. DocumentationSearch --

pub fn search_documentation(index: &SearchIndex, q: &SearchQuery, kw: &str) -> SearchResult {
    let start = std::time::Instant::now();
    let mut matches = Vec::new();
    let mut fs = HashSet::new();
    let kw_lower = kw.to_lowercase();
    for e in index.query_docs(kw) {
        if !entry_in_scope(e, &q.scope) {
            continue;
        }
        fs.insert(e.file.clone());
        let doc = e.documentation.as_deref().unwrap_or("");
        let score = doc_score(&kw_lower, doc);
        let lines: Vec<String> = doc.lines().map(|l| l.to_string()).collect();
        let mut m = entry_to_match(e, score);
        m.kind = MatchKind::Documentation;
        m.context = Some(MatchContext {
            before: vec![],
            matched: lines,
            after: vec![],
        });
        matches.push(m);
    }
    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(q.max_results);
    let total = matches.len();
    SearchResult {
        matches,
        total_count: total,
        stats: make_stats(&fs, start),
    }
}

fn doc_score(kw: &str, doc: &str) -> f64 {
    let dl = doc.to_lowercase();
    let mut s = 0.5_f64;
    if dl.starts_with(kw) {
        s += 0.3;
    } else if dl.find(kw).unwrap_or(usize::MAX) < 80 {
        s += 0.15;
    }
    if dl
        .split_whitespace()
        .any(|w| w.trim_matches(|c: char| !c.is_alphanumeric()) == kw)
    {
        s += 0.1;
    }
    s.min(0.95)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Position, Range};
    use crate::semantic::{Symbol, SymbolId, SymbolKind};
    use crate::type_inference::SearchKind;

    fn make_idx(name: &str, doc: Option<&str>) -> SearchIndex {
        let mut idx = SearchIndex::new();
        let sym = Symbol {
            id: SymbolId(0),
            name: name.to_string(),
            kind: SymbolKind::Function,
            location: Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 10,
                },
            },
            type_info: None,
            doc: doc.map(|d| d.to_string()),
            scope_id: 0,
        };
        idx.insert(Path::new("/test.py"), &sym);
        idx
    }

    fn pq(max: usize) -> SearchQuery {
        SearchQuery {
            kind: SearchKind::ByDocumentation {
                query: String::new(),
            },
            scope: SearchScope::Project,
            max_results: max,
        }
    }

    #[test]
    fn test_doc_search() {
        let r = search_documentation(
            &make_idx("calc", Some("Calculate total price")),
            &pq(10),
            "price",
        );
        assert!(!r.matches.is_empty());
    }

    #[test]
    fn test_usages() {
        let r = search_usages(
            &make_idx("my_func", None),
            &pq(10),
            "my_func",
            Path::new("/test.py"),
        );
        assert_eq!(r.matches.len(), 1);
    }

    #[test]
    fn test_similarity() {
        assert!(name_similarity("foo", "foo") > 0.9);
        assert!(name_similarity("abc", "xyz") < 0.2);
    }
}
