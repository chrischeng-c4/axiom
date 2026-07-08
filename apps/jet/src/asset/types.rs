// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
// CODEGEN-BEGIN
use std::path::PathBuf;

/// Asset processing options
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
#[derive(Debug, Clone)]
pub struct AssetOptions {
    /// Enable image optimization
    pub optimize_images: bool,

    /// Add content hash to filenames
    pub hash_filenames: bool,

    /// Maximum image size (in bytes)
    pub max_image_size: usize,
}

/// Processed asset
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
#[derive(Debug, Clone)]
pub struct ProcessedAsset {
    /// Original file path
    pub original_path: PathBuf,

    /// Processed content
    pub content: Vec<u8>,

    /// Output filename
    pub filename: String,

    /// Content hash
    pub hash: String,

    /// Asset type
    pub asset_type: AssetType,
}

/// Asset type
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    /// Image file
    Image,

    /// Font file
    Font,

    /// Other asset
    Other,
}
// CODEGEN-END
