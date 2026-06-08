//! Diff and patch module.
//!
//! Provides:
//! - Line-level diff with unified diff format
//! - Word-level diff with inline markup
//! - Patch parsing and application

pub mod line_diff;
pub mod patch;
pub mod word_diff;

pub use line_diff::{diff_lines, unified_diff, DiffHunk, DiffOp};
pub use patch::{
    apply_patch, apply_patch_text, parse_patch, FilePatch, Patch, PatchHunk, PatchLine,
};
pub use word_diff::{diff_words, format_word_diff, format_word_diff_html, WordDiffOp};
