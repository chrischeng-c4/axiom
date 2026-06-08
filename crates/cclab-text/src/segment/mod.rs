//! Text segmentation module (Jieba-like).
//!
//! Features:
//! - Precise/Full/Search segmentation modes
//! - HMM-based unknown word recognition
//! - TF-IDF and TextRank keyword extraction
//! - Simplified/Traditional Chinese conversion
//! - Parallel batch processing (with `parallel` feature)

mod dict;
mod error;
mod hmm;
mod keyword;
mod segmenter;
mod traditional;
mod trie;

pub use error::{JiebaError, Result};
pub use keyword::{Keyword, KeywordAlgorithm, KeywordExtractor};
pub use segmenter::{JiebaSegmenter, Token, TokenizeMode};
pub use traditional::ChineseConverter;
