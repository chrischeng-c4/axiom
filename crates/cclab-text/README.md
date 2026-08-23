# Cclab Text

## Brief

Cclab Text is the Rust text-processing API surface for cclab crates.

It owns Chinese segmentation, keyword extraction, ranking, fuzzy matching,
markup parsing/query/transform, diff/patch formatting, and local template
rendering. The public contract is a Rust library API; this crate does not
expose a standalone CLI surface.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Chinese Segmentation And Keywords | - | Jieba-compatible tokenization, keyword extraction, and simplified/traditional conversion |
| Search Ranking And Fuzzy Matching | - | BM25/TF-IDF/TextRank ranking plus fuzzy string matching |
| Markup Query And Transform | - | HTML/XML parsing, DOM query, selector, XPath, and transform APIs |
| Diff Patch And Word Markup | - | Line/word diffing and unified patch parse/apply behavior |
| Template Rendering | - | Jinja-style render engine and parser APIs |

### Chinese Segmentation And Keywords

Cclab Text provides Jieba-compatible Chinese segmentation, keyword extraction,
and simplified/traditional conversion APIs for Rust callers.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API:
  `cclab_text::segment::{JiebaSegmenter, KeywordExtractor, ChineseConverter, TokenizeMode}`
- Gate — behavior: `cargo test -p cclab-text` - segmentation, token offsets,
  POS tags, keyword extraction, and script conversion
- Gate: `cargo test -p cclab-text`
- Evidence: `cargo test -p cclab-text`

### Search Ranking And Fuzzy Matching

Cclab Text provides search ranking primitives and fuzzy string matching APIs
for local text retrieval and candidate scoring.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API:
  `cclab_text::rank::{BM25Okapi, TfIdf, TextRank, Tokenizer}` +
  `cclab_text::fuzzy::{levenshtein, jaro_winkler, FuzzySearcher, extract_one}`
  - ranking and fuzzy matching entrypoints
- Gate — behavior: `cargo test -p cclab-text` - ranking models, tokenizer
  behavior, distance metrics, and fuzzy candidate selection
- Gate: `cargo test -p cclab-text`
- Evidence: `cargo test -p cclab-text`

### Markup Query And Transform

Cclab Text exposes HTML/XML parsing, query, and transform APIs for Rust-side
markup inspection and manipulation.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API:
  `cclab_text::markup::{parse_html, parse_xml, select, xpath, transform, Document}`
- Gate — behavior: `cargo test -p cclab-text` - HTML/XML parser, DOM traversal,
  CSS selectors, XPath, and transform behavior
- Gate: `cargo test -p cclab-text`
- Evidence: `cargo test -p cclab-text`

### Diff Patch And Word Markup

Cclab Text provides line and word diffing plus unified patch parsing,
formatting, and application APIs.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API:
  `cclab_text::diff::{diff_lines, unified_diff, parse_patch, apply_patch, diff_words, format_word_diff}`
- Gate — behavior: `cargo test -p cclab-text` - line diffs, word diffs, unified
  diff formatting, and patch application
- Gate: `cargo test -p cclab-text`
- Evidence: `cargo test -p cclab-text`

### Template Rendering

Cclab Text provides a Jinja-style template parser and render engine for Rust
applications that need local text templating.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API:
  `cclab_text::template::{render, Engine, Context, MapLoader, FileLoader, parse}`
- Gate — behavior: `cargo test -p cclab-text` - interpolation, filters,
  conditionals, loops, set blocks, inheritance, includes, and parser contracts
- Gate: `cargo test -p cclab-text`
- Evidence: `cargo test -p cclab-text`
