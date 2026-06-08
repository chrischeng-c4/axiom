// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css-plugins.md#schema
// CODEGEN-BEGIN
//! CSS plugin emitter registry.
//!
//! Implements the `PluginEmitter` trait and dispatches to concrete native
//! Rust emitters based on plugin name.  No JS execution required.

pub mod animate;
pub mod typography;

use std::collections::HashSet;

// ─── PluginEmitter trait ──────────────────────────────────────────────────────

/// A native Rust emitter for a Tailwind CSS plugin.
pub trait PluginEmitter: Send + Sync {
    /// The plugin name as it appears in `tailwind.config.js plugins: [...]`.
    fn name(&self) -> &str;

    /// Emit CSS for all classes in `used_classes` that belong to this plugin.
    ///
    /// Returns the CSS string to append to the utilities layer.  Returns an
    /// empty string if no relevant classes are used.
    fn emit(&self, used_classes: &HashSet<String>) -> String;
}

// ─── built-in emitters ────────────────────────────────────────────────────────

struct AnimateEmitter;
struct TypographyEmitter;

/// @spec .aw/tech-design/projects/jet/semantic/jet-css-plugins.md#schema
impl PluginEmitter for AnimateEmitter {
    fn name(&self) -> &str {
        "tailwindcss-animate"
    }

    fn emit(&self, used_classes: &HashSet<String>) -> String {
        animate::emit(used_classes)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css-plugins.md#schema
impl PluginEmitter for TypographyEmitter {
    fn name(&self) -> &str {
        "@tailwindcss/typography"
    }

    fn emit(&self, used_classes: &HashSet<String>) -> String {
        typography::emit(used_classes)
    }
}

// ─── plugin registry ──────────────────────────────────────────────────────────

/// Registry of all known plugin emitters.
static EMITTERS: &[&(dyn PluginEmitter + Sync)] = &[&AnimateEmitter, &TypographyEmitter];

/// Run all plugin emitters that match a name in `enabled_plugins` and return
/// the concatenated CSS.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-plugins.md#schema
pub fn emit_plugins(enabled_plugins: &[String], used_classes: &HashSet<String>) -> String {
    let mut out = String::new();

    for emitter in EMITTERS {
        if enabled_plugins.iter().any(|p| p == emitter.name()) {
            let css = emitter.emit(used_classes);
            if !css.is_empty() {
                out.push_str(&css);
                out.push('\n');
            }
        }
    }

    out
}
// CODEGEN-END
