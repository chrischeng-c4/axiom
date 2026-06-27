// <HANDWRITE gap="missing-generator:logic:a764f528" tracker="standardize-gap-projects-jet-src-stories-mod-rs" reason="New stories module root: StoryIndex/StoryEntry/StoryMeta types and discover(root) that globs *.stories.* and assembles the index + diagnostics.">
//! Story discovery + CSF parsing for `jet stories`.
//!
//! This module is the foundation that the manager (B2) and controls (B3)
//! consume. It does two things, no UI / no server:
//!
//! 1. [`discover`] globs `**/*.stories.@(ts|tsx|js|jsx)` under a root and
//!    parses each file's CSF structure ([`csf::parse_csf`]).
//! 2. It assembles a normalized [`StoryIndex`]: one [`StoryEntry`] per named
//!    story, each carrying its merged args, a stable id, and the sidebar title
//!    hierarchy derived from `meta.title` (with a path-based fallback).
//!
//! Per-file failures (parse errors, unreadable files, no default export) become
//! diagnostics on the index — they never abort discovery of the other files.

pub mod build;
pub mod controls;
pub mod csf;
pub mod deps;
pub mod hmr;
pub mod manager;
pub mod prop_extractor;
pub mod server;

pub use build::{build_stories_static, BuildStaticResult};
pub use server::start_stories_workbench;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::{DirEntry, WalkDir};

use csf::{CsfMeta, CsfValue, ParsedStoryFile};

/// Glob patterns that identify a CSF story file.
const STORY_GLOBS: &[&str] = &[
    "**/*.stories.ts",
    "**/*.stories.tsx",
    "**/*.stories.js",
    "**/*.stories.jsx",
];

/// Normalized meta for one story file (the default export).
///
/// Mirrors [`csf::CsfMeta`] but carries the resolved title hierarchy so B2 can
/// build the sidebar without re-splitting `title`.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryMeta {
    /// The story file this meta came from.
    pub file: PathBuf,
    /// `component:` reference (raw source, usually an identifier).
    pub component: Option<String>,
    /// `title:` field as authored (`Components/Button`), if present.
    pub title: Option<String>,
    /// Title split into a sidebar path (`["Components", "Button"]`). Derived
    /// from `title` when present, otherwise from the file path.
    pub title_path: Vec<String>,
    /// File-level default args (applied to every story in the file).
    pub args: BTreeMap<String, CsfValue>,
    /// `argTypes:` control metadata (consumed by B3).
    pub arg_types: BTreeMap<String, CsfValue>,
}

/// One renderable story (a named export), with args merged over the meta.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryEntry {
    /// Stable id: `slug(title)--slug(export_name)`. Unique within an index.
    pub id: String,
    /// Display name (the export identifier, e.g. `Primary`).
    pub name: String,
    /// The export identifier as it appears in source (same as `name` today,
    /// kept distinct so a future "name" override field can diverge from it).
    pub export_name: String,
    /// Args effective for this story = meta args overridden by story args.
    pub args: BTreeMap<String, CsfValue>,
    /// Whether the story declares its own `render:` function.
    pub has_render: bool,
    /// The story file this entry came from.
    pub file: PathBuf,
    /// The sidebar title path of this story's meta.
    pub title_path: Vec<String>,
}

/// The assembled, normalized story index.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StoryIndex {
    /// One meta per discovered (valid) story file.
    pub metas: Vec<StoryMeta>,
    /// One entry per named story across all files, sorted by id.
    pub stories: Vec<StoryEntry>,
    /// Human-readable per-file problems; never fatal to discovery.
    pub diagnostics: Vec<String>,
}

impl StoryIndex {
    /// The sidebar title hierarchy: a sorted, de-duplicated set of every
    /// title path prefix. `Components/Button` contributes `["Components"]` and
    /// `["Components", "Button"]`. Lets B2 build the tree without re-walking.
    pub fn title_hierarchy(&self) -> Vec<Vec<String>> {
        let mut set: std::collections::BTreeSet<Vec<String>> = std::collections::BTreeSet::new();
        for meta in &self.metas {
            for i in 1..=meta.title_path.len() {
                set.insert(meta.title_path[..i].to_vec());
            }
        }
        set.into_iter().collect()
    }
}

/// Discover and parse every story file under `root`, assembling a [`StoryIndex`].
///
/// Discovery never aborts on a bad file: a parse error, an unreadable file, or
/// a missing default export is recorded as a diagnostic and the walk continues.
pub fn discover(root: &Path) -> StoryIndex {
    let mut index = StoryIndex::default();

    let globset = match build_globset(STORY_GLOBS) {
        Ok(g) => g,
        Err(err) => {
            index
                .diagnostics
                .push(format!("failed to build story globset: {err}"));
            return index;
        }
    };

    let mut files = discover_files(root, &globset, &mut index.diagnostics);
    // Deterministic order so ids / hierarchy are stable across runs.
    files.sort();

    for file in files {
        let rel = file.strip_prefix(root).unwrap_or(&file).to_path_buf();
        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(err) => {
                index
                    .diagnostics
                    .push(format!("{}: failed to read: {err}", rel.display()));
                continue;
            }
        };
        let is_tsx = matches!(
            file.extension().and_then(|e| e.to_str()),
            Some("tsx") | Some("jsx")
        );
        match csf::parse_csf(&source, is_tsx) {
            Ok(parsed) => assemble_file(&mut index, &file, parsed),
            Err(err) => index
                .diagnostics
                .push(format!("{}: parse error: {err}", rel.display())),
        }
    }

    index.stories.sort_by(|a, b| a.id.cmp(&b.id));
    index.metas.sort_by(|a, b| a.file.cmp(&b.file));
    index
}

/// Fold one parsed story file into the index.
///
/// Re-exported stories (`export { Primary } from './sibling'`) are resolved
/// against the importing `file`: the sibling is parsed and the named story is
/// pulled in under this file's title. Unresolvable re-exports become a
/// diagnostic and are skipped — they never abort discovery.
fn assemble_file(index: &mut StoryIndex, file: &Path, parsed: ParsedStoryFile) {
    let ParsedStoryFile {
        meta,
        stories,
        re_exports,
    } = parsed;
    let title_path = resolve_title_path(&meta, file);

    let story_meta = StoryMeta {
        file: file.to_path_buf(),
        component: meta.component.clone(),
        title: meta.title.clone(),
        title_path: title_path.clone(),
        args: meta.args.clone(),
        arg_types: meta.arg_types.clone(),
    };

    let title_slug = slug(&title_path.join("/"));
    for story in &stories {
        push_story(index, file, &title_slug, &title_path, &meta.args, story);
    }

    // Resolve each re-exported story against its sibling file.
    for re in &re_exports {
        match resolve_re_export(file, re) {
            Ok(sibling) => {
                // The re-export keeps THIS file's title, but adopts the sibling
                // story's args; the local story is renamed to the exported name.
                if let Some(src_story) = sibling
                    .stories
                    .iter()
                    .find(|s| s.export_name == re.local_name)
                {
                    let renamed = csf::CsfStory {
                        export_name: re.exported_name.clone(),
                        args: src_story.args.clone(),
                        has_render: src_story.has_render,
                    };
                    // Story-level args still merge over the sibling meta args so
                    // an inherited default is not lost.
                    push_story(
                        index,
                        file,
                        &title_slug,
                        &title_path,
                        &sibling.meta.args,
                        &renamed,
                    );
                } else {
                    index.diagnostics.push(format!(
                        "{}: re-export `{}` not found in `{}`",
                        rel_display(index, file),
                        re.local_name,
                        re.relative_source
                    ));
                }
            }
            Err(err) => index.diagnostics.push(format!(
                "{}: re-export from `{}` unresolved: {err}",
                rel_display(index, file),
                re.relative_source
            )),
        }
    }

    index.metas.push(story_meta);
}

/// Push one story into the index, merging `base_args` (meta/sibling defaults)
/// under the story's own args.
fn push_story(
    index: &mut StoryIndex,
    file: &Path,
    title_slug: &str,
    title_path: &[String],
    base_args: &BTreeMap<String, CsfValue>,
    story: &csf::CsfStory,
) {
    // Merge: base args first, story args override per key.
    let mut merged = base_args.clone();
    for (k, v) in &story.args {
        merged.insert(k.clone(), v.clone());
    }
    let id = format!("{title_slug}--{}", slug(&story.export_name));
    index.stories.push(StoryEntry {
        id,
        name: story.export_name.clone(),
        export_name: story.export_name.clone(),
        args: merged,
        has_render: story.has_render,
        file: file.to_path_buf(),
        title_path: title_path.to_vec(),
    });
}

/// Resolve a re-export's relative source to a sibling `*.stories.*` file and
/// parse it.
///
/// The specifier is resolved relative to the importing file's directory. The
/// `.stories.<ext>` extension is usually omitted in the import, so we probe the
/// four CSF extensions (and the literal path, in case it is spelled out).
fn resolve_re_export(file: &Path, re: &csf::CsfReExport) -> std::io::Result<ParsedStoryFile> {
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    let base = dir.join(&re.relative_source);

    // Candidate paths: the literal spelling first, then the four CSF extensions
    // appended to the (extension-less) specifier.
    let mut candidates: Vec<PathBuf> = vec![base.clone()];
    for ext in ["ts", "tsx", "js", "jsx"] {
        let mut p = base.clone();
        let new_name = match base.file_name().and_then(|n| n.to_str()) {
            Some(name) => format!("{name}.{ext}"),
            None => continue,
        };
        p.set_file_name(new_name);
        candidates.push(p);
    }

    let resolved = candidates
        .into_iter()
        .find(|p| p.is_file())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no sibling file for `{}`", re.relative_source),
            )
        })?;

    let source = std::fs::read_to_string(&resolved)?;
    let is_tsx = matches!(
        resolved.extension().and_then(|e| e.to_str()),
        Some("tsx") | Some("jsx")
    );
    csf::parse_csf(&source, is_tsx)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

/// Best-effort relative display of `file` for a diagnostic. We do not have the
/// discovery root here, so fall back to the file name.
fn rel_display(_index: &StoryIndex, file: &Path) -> String {
    file.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| file.display().to_string())
}

/// The sidebar path for a meta: split `meta.title` on `/`, else derive from the
/// file path (drop the `.stories.<ext>` suffix, title-case nothing — keep the
/// stem so the fallback is predictable).
fn resolve_title_path(meta: &CsfMeta, file: &Path) -> Vec<String> {
    if let Some(title) = &meta.title {
        let parts: Vec<String> = title
            .split('/')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        if !parts.is_empty() {
            return parts;
        }
    }
    // Fallback: use the file stem with the `.stories` infix removed.
    let stem = file
        .file_name()
        .and_then(|n| n.to_str())
        .map(strip_story_suffix)
        .unwrap_or_else(|| "Story".to_string());
    vec![stem]
}

/// `Button.stories.tsx` -> `Button`.
fn strip_story_suffix(name: &str) -> String {
    name.strip_suffix(".stories.ts")
        .or_else(|| name.strip_suffix(".stories.tsx"))
        .or_else(|| name.strip_suffix(".stories.js"))
        .or_else(|| name.strip_suffix(".stories.jsx"))
        .unwrap_or(name)
        .to_string()
}

/// Lower-case, replace any run of non-alphanumeric chars with a single `-`,
/// trim leading/trailing `-`. `Components/Button` -> `components-button`.
fn slug(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "story".to_string()
    } else {
        out
    }
}

/// Walk `root` and collect every file matching the story globs.
fn discover_files(root: &Path, globset: &GlobSet, diagnostics: &mut Vec<String>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Never filter the walk root (tempdir names start with a dot on macOS),
            // and skip dot-dirs/node_modules so discovery stays cheap.
            e.depth() == 0 || (!is_hidden(e) && !is_node_modules(e))
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                diagnostics.push(format!("walk error: {err}"));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path);
        if globset.is_match(rel) {
            out.push(path.to_path_buf());
        }
    }
    out
}

fn build_globset(patterns: &[&str]) -> Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for raw in patterns {
        builder.add(Glob::new(raw)?);
    }
    builder.build()
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s != "." && s.starts_with('.'))
        .unwrap_or(false)
}

fn is_node_modules(entry: &DirEntry) -> bool {
    entry.file_type().is_dir() && entry.file_name() == "node_modules"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_normalizes() {
        assert_eq!(slug("Components/Button"), "components-button");
        assert_eq!(slug("Primary"), "primary");
        assert_eq!(slug("With Footer!!"), "with-footer");
        assert_eq!(slug(""), "story");
    }

    #[test]
    fn strip_story_suffix_handles_all_exts() {
        assert_eq!(strip_story_suffix("Button.stories.tsx"), "Button");
        assert_eq!(strip_story_suffix("Card.stories.js"), "Card");
        assert_eq!(strip_story_suffix("Plain.tsx"), "Plain.tsx");
    }

    #[test]
    fn resolve_title_path_prefers_meta_title() {
        let mut meta = CsfMeta::default();
        meta.title = Some("Components/Button".to_string());
        let path = resolve_title_path(&meta, Path::new("/x/Button.stories.tsx"));
        assert_eq!(path, vec!["Components".to_string(), "Button".to_string()]);
    }

    #[test]
    fn resolve_title_path_falls_back_to_file_stem() {
        let meta = CsfMeta::default();
        let path = resolve_title_path(&meta, Path::new("/x/Widget.stories.tsx"));
        assert_eq!(path, vec!["Widget".to_string()]);
    }
}
// </HANDWRITE>
