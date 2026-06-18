# Cclab Text

## Brief

Cclab Text is the Rust text-processing API surface for cclab crates.

It owns Chinese segmentation, keyword extraction, ranking, fuzzy matching,
markup parsing/query/transform, diff/patch formatting, and local template
rendering. The public contract is a Rust library API; this crate does not
expose a standalone CLI surface.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Chinese Segmentation And Keywords | - | implemented | passing | conformance | not_ready | Jieba-compatible tokenization, keyword extraction, and simplified/traditional conversion |
| Search Ranking And Fuzzy Matching | - | implemented | passing | conformance | not_ready | BM25/TF-IDF/TextRank ranking plus fuzzy string matching |
| Markup Query And Transform | - | implemented | passing | conformance | not_ready | HTML/XML parsing, DOM query, selector, XPath, and transform APIs |
| Diff Patch And Word Markup | - | implemented | passing | conformance | not_ready | Line/word diffing and unified patch parse/apply behavior |
| Template Rendering | - | implemented | passing | smoke | not_ready | Jinja-style render engine and parser APIs |

### Chinese Segmentation And Keywords

ID: chinese-segmentation-and-keywords
Type: DeveloperTool
Surfaces: Rust API: `cclab_text::segment::{JiebaSegmenter, KeywordExtractor, ChineseConverter, TokenizeMode}`
EC Dimensions: behavior: `cargo test -p cclab-text` - segmentation, token offsets, POS tags, keyword extraction, and script conversion
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Text provides Jieba-compatible Chinese segmentation, keyword extraction, and simplified/traditional conversion APIs for Rust callers.
Gate Inventory: `cargo test -p cclab-text`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Segmentation and keyword API contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-text` |

### Search Ranking And Fuzzy Matching

ID: search-ranking-and-fuzzy-matching
Type: DeveloperTool
Surfaces: Rust API: `cclab_text::rank::{BM25Okapi, TfIdf, TextRank, Tokenizer}` + `cclab_text::fuzzy::{levenshtein, jaro_winkler, FuzzySearcher, extract_one}` - ranking and fuzzy matching entrypoints
EC Dimensions: behavior: `cargo test -p cclab-text` - ranking models, tokenizer behavior, distance metrics, and fuzzy candidate selection
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Text provides search ranking primitives and fuzzy string matching APIs for local text retrieval and candidate scoring.
Gate Inventory: `cargo test -p cclab-text`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Ranking and fuzzy API contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-text` |

### Markup Query And Transform

ID: markup-query-and-transform
Type: DeveloperTool
Surfaces: Rust API: `cclab_text::markup::{parse_html, parse_xml, select, xpath, transform, Document}`
EC Dimensions: behavior: `cargo test -p cclab-text` - HTML/XML parser, DOM traversal, CSS selectors, XPath, and transform behavior
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Text exposes HTML/XML parsing, query, and transform APIs for Rust-side markup inspection and manipulation.
Gate Inventory: `cargo test -p cclab-text`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Markup parser and query contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-text` |

### Diff Patch And Word Markup

ID: diff-patch-and-word-markup
Type: DeveloperTool
Surfaces: Rust API: `cclab_text::diff::{diff_lines, unified_diff, parse_patch, apply_patch, diff_words, format_word_diff}`
EC Dimensions: behavior: `cargo test -p cclab-text` - line diffs, word diffs, unified diff formatting, and patch application
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Text provides line and word diffing plus unified patch parsing, formatting, and application APIs.
Gate Inventory: `cargo test -p cclab-text`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Diff and patch API contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-text` |

### Template Rendering

ID: template-rendering
Type: DeveloperTool
Surfaces: Rust API: `cclab_text::template::{render, Engine, Context, MapLoader, FileLoader, parse}`
EC Dimensions: behavior: `cargo test -p cclab-text` - interpolation, filters, conditionals, loops, set blocks, inheritance, includes, and parser contracts
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Text provides a Jinja-style template parser and render engine for Rust applications that need local text templating.
Gate Inventory: `cargo test -p cclab-text`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Template render API contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-text` |
