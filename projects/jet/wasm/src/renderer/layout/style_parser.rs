// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
// CODEGEN-BEGIN
//! Inline style-prop parser — `HashMap<String, String>` → `LayoutStyle`.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/layout-runtime.md#config
//!
//! The transpiler emits `style={{...}}` JSX as a Rust
//! `HashMap<String, String>` of CSS property → value (camel-case
//! JS keys are normalized to kebab-case before lookup). This module
//! consumes that map and returns a `LayoutStyle`.
//!
//! Three classes of property (R7):
//!   - **Recognized**: in the css-to-taffy mapping table — parsed
//!     into the corresponding `LayoutStyle` field.
//!   - **Silently ignored**: e.g. `color`, `background-color`,
//!     `font-size` — paint-only, not relevant to layout. No error.
//!   - **Validation error**: explicitly out-of-scope per R9 — e.g.
//!     `display:grid`, `position:absolute`. Parse fails with
//!     `ParseError`.
//!
//! Malformed values for a recognized property emit a `ParseError`
//! with the property name + offending value + accepted formats; the
//! style for that property falls back to taffy's default.

use std::collections::HashMap;

use super::{
    AlignItems, Dimension, DisplayKind, FlexDirection, JustifyContent, LayoutStyle, Rect4,
};

/// Parse error surfaced by [`parse_style`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub property: String,
    pub value: String,
    pub reason: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
impl ParseError {
    fn new(
        property: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            property: property.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }
}

/// Properties silently ignored — paint-only. Maintained in sync with
/// the layout-runtime.md silent_ignore_list.
const SILENT_IGNORE: &[&str] = &[
    "color",
    "background-color",
    "background",
    "font-size",
    "font-family",
    "font-weight",
    "line-height",
    "text-align",
    "letter-spacing",
    "opacity",
    "cursor",
    "box-sizing",
    "outline",
    "text-decoration",
    "white-space",
    "overflow-x",
    "overflow-y",
];

/// Properties whose use is an explicit validation error (R9 deferred).
const ERROR_PROPERTIES: &[&str] = &[
    "position",
    "float",
    "z-index",
    "writing-mode",
    "direction",
    "transform",
    "transition",
    "animation",
    "clip-path",
    "filter",
];

/// Property:value pairs whose use is an explicit validation error.
const ERROR_PROPERTY_VALUES: &[(&str, &[&str])] = &[
    (
        "display",
        &[
            "grid",
            "table",
            "table-row",
            "table-cell",
            "inline-flex",
            "inline-block",
        ],
    ),
    ("overflow", &["scroll", "auto"]),
];

/// Parse a `HashMap<String, String>` of CSS-property → value into a
/// `LayoutStyle`. Returns `(style, errors)` — the style is best-
/// effort with defaults for any field whose source value failed to
/// parse; errors carry the diagnostics.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
pub fn parse_style(map: &HashMap<String, String>) -> (LayoutStyle, Vec<ParseError>) {
    let mut style = LayoutStyle::default();
    let mut errors = Vec::new();

    for (raw_key, raw_value) in map {
        let key = normalize_key(raw_key);
        let value = raw_value.trim();

        // Validation-error properties (R9 deferred set).
        if ERROR_PROPERTIES.contains(&key.as_str()) {
            errors.push(ParseError::new(
                &key,
                value,
                "property is explicitly out of scope (R9)",
            ));
            continue;
        }
        // Validation-error property:value combinations.
        if let Some((_, bad_values)) = ERROR_PROPERTY_VALUES
            .iter()
            .find(|(k, _)| *k == key.as_str())
        {
            if bad_values.contains(&value) {
                errors.push(ParseError::new(
                    &key,
                    value,
                    "value is explicitly out of scope (R9)",
                ));
                continue;
            }
        }

        // Silent-ignore — paint-only properties.
        if SILENT_IGNORE.contains(&key.as_str()) {
            continue;
        }

        // Recognized properties.
        match key.as_str() {
            "display" => match parse_display(value) {
                Ok(d) => style.display = Some(d),
                Err(e) => errors.push(ParseError::new(&key, value, e)),
            },
            "width" => assign_dim(&mut style.width, &key, value, &mut errors),
            "height" => assign_dim(&mut style.height, &key, value, &mut errors),
            "min-width" => assign_dim(&mut style.min_width, &key, value, &mut errors),
            "min-height" => assign_dim(&mut style.min_height, &key, value, &mut errors),
            "max-width" => assign_dim(&mut style.max_width, &key, value, &mut errors),
            "max-height" => assign_dim(&mut style.max_height, &key, value, &mut errors),
            "padding" => assign_rect4(&mut style.padding, &key, value, &mut errors, false),
            "margin" => assign_rect4(&mut style.margin, &key, value, &mut errors, true),
            "border-width" => {
                assign_rect4(&mut style.border_width, &key, value, &mut errors, false)
            }
            "flex-direction" => match parse_flex_direction(value) {
                Ok(fd) => style.flex_direction = Some(fd),
                Err(e) => errors.push(ParseError::new(&key, value, e)),
            },
            "justify-content" => match parse_justify_content(value) {
                Ok(jc) => style.justify_content = Some(jc),
                Err(e) => errors.push(ParseError::new(&key, value, e)),
            },
            "align-items" => match parse_align_items(value) {
                Ok(ai) => style.align_items = Some(ai),
                Err(e) => errors.push(ParseError::new(&key, value, e)),
            },
            // Unknown property — silent-ignore per R7
            // unknown_property_handling: silent_ignore.
            _ => {}
        }
    }

    (style, errors)
}

fn normalize_key(raw: &str) -> String {
    // camelCase → kebab-case. The transpiler should already emit
    // kebab, but accept either to be defensive.
    let mut out = String::with_capacity(raw.len() + 4);
    for (i, c) in raw.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_display(value: &str) -> Result<DisplayKind, &'static str> {
    match value {
        "block" => Ok(DisplayKind::Block),
        "flex" => Ok(DisplayKind::Flex),
        "none" => Ok(DisplayKind::None),
        _ => Err("expected one of: block | flex | none"),
    }
}

fn parse_flex_direction(value: &str) -> Result<FlexDirection, &'static str> {
    match value {
        "row" => Ok(FlexDirection::Row),
        "column" => Ok(FlexDirection::Column),
        "row-reverse" => Ok(FlexDirection::RowReverse),
        "column-reverse" => Ok(FlexDirection::ColumnReverse),
        _ => Err("expected one of: row | column | row-reverse | column-reverse"),
    }
}

fn parse_justify_content(value: &str) -> Result<JustifyContent, &'static str> {
    match value {
        "flex-start" => Ok(JustifyContent::FlexStart),
        "flex-end" => Ok(JustifyContent::FlexEnd),
        "center" => Ok(JustifyContent::Center),
        "space-between" => Ok(JustifyContent::SpaceBetween),
        "space-around" => Ok(JustifyContent::SpaceAround),
        "space-evenly" => Ok(JustifyContent::SpaceEvenly),
        _ => Err("expected: flex-start|flex-end|center|space-between|space-around|space-evenly"),
    }
}

fn parse_align_items(value: &str) -> Result<AlignItems, &'static str> {
    match value {
        "flex-start" => Ok(AlignItems::FlexStart),
        "flex-end" => Ok(AlignItems::FlexEnd),
        "center" => Ok(AlignItems::Center),
        "baseline" => Ok(AlignItems::Baseline),
        "stretch" => Ok(AlignItems::Stretch),
        _ => Err("expected: flex-start|flex-end|center|baseline|stretch"),
    }
}

/// Parse a single dimension token: "Npx", "N%", or "auto".
fn parse_dimension(value: &str, allow_auto: bool) -> Result<Dimension, String> {
    let v = value.trim();
    if v == "auto" {
        if allow_auto {
            return Ok(Dimension::Auto);
        }
        return Err("auto is not valid for this property".to_string());
    }
    if let Some(num) = v.strip_suffix("px") {
        return num
            .trim()
            .parse::<f32>()
            .map(Dimension::Px)
            .map_err(|_| format!("expected <N>px, got {v:?}"));
    }
    if let Some(num) = v.strip_suffix('%') {
        return num
            .trim()
            .parse::<f32>()
            .map(Dimension::Pct)
            .map_err(|_| format!("expected <N>%, got {v:?}"));
    }
    Err(format!("expected <N>px, <N>%, or auto, got {v:?}"))
}

fn assign_dim(field: &mut Option<Dimension>, key: &str, value: &str, errors: &mut Vec<ParseError>) {
    match parse_dimension(value, true) {
        Ok(d) => *field = Some(d),
        Err(e) => errors.push(ParseError::new(key, value, e)),
    }
}

/// Parse a 1-, 2-, 3-, or 4-value shorthand into a `Rect4`.
fn parse_rect4_shorthand(value: &str, allow_auto: bool) -> Result<Rect4, String> {
    let parts: Vec<&str> = value.split_ascii_whitespace().collect();
    let dims: Result<Vec<Dimension>, String> = parts
        .iter()
        .map(|p| parse_dimension(p, allow_auto))
        .collect();
    let dims = dims?;
    Ok(match dims.as_slice() {
        [a] => Rect4 {
            top: *a,
            right: *a,
            bottom: *a,
            left: *a,
        },
        [v, h] => Rect4 {
            top: *v,
            right: *h,
            bottom: *v,
            left: *h,
        },
        [t, h, b] => Rect4 {
            top: *t,
            right: *h,
            bottom: *b,
            left: *h,
        },
        [t, r, b, l] => Rect4 {
            top: *t,
            right: *r,
            bottom: *b,
            left: *l,
        },
        _ => return Err("expected 1–4 space-separated values".to_string()),
    })
}

fn assign_rect4(
    field: &mut Option<Rect4>,
    key: &str,
    value: &str,
    errors: &mut Vec<ParseError>,
    allow_auto: bool,
) {
    match parse_rect4_shorthand(value, allow_auto) {
        Ok(r) => *field = Some(r),
        Err(e) => errors.push(ParseError::new(key, value, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn parses_basic_block() {
        let m = map(&[
            ("display", "block"),
            ("width", "200px"),
            ("height", "100px"),
        ]);
        let (s, errs) = parse_style(&m);
        assert!(errs.is_empty(), "errors: {errs:?}");
        assert_eq!(s.display, Some(DisplayKind::Block));
        assert_eq!(s.width, Some(Dimension::Px(200.0)));
        assert_eq!(s.height, Some(Dimension::Px(100.0)));
    }

    #[test]
    fn camel_case_normalizes() {
        let m = map(&[("flexDirection", "column"), ("justifyContent", "center")]);
        let (s, errs) = parse_style(&m);
        assert!(errs.is_empty(), "errors: {errs:?}");
        assert_eq!(s.flex_direction, Some(FlexDirection::Column));
        assert_eq!(s.justify_content, Some(JustifyContent::Center));
    }

    #[test]
    fn padding_shorthand_4() {
        let m = map(&[("padding", "10px 20px 30px 40px")]);
        let (s, errs) = parse_style(&m);
        assert!(errs.is_empty(), "errors: {errs:?}");
        let p = s.padding.unwrap();
        assert_eq!(p.top, Dimension::Px(10.0));
        assert_eq!(p.right, Dimension::Px(20.0));
        assert_eq!(p.bottom, Dimension::Px(30.0));
        assert_eq!(p.left, Dimension::Px(40.0));
    }

    #[test]
    fn margin_auto_ok() {
        let m = map(&[("margin", "auto")]);
        let (s, errs) = parse_style(&m);
        assert!(errs.is_empty(), "errors: {errs:?}");
        let mg = s.margin.unwrap();
        assert_eq!(mg.top, Dimension::Auto);
    }

    #[test]
    fn padding_auto_rejected() {
        let m = map(&[("padding", "auto")]);
        let (_, errs) = parse_style(&m);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].property, "padding");
    }

    #[test]
    fn paint_only_silent_ignore() {
        let m = map(&[("color", "red"), ("font-size", "14px")]);
        let (_, errs) = parse_style(&m);
        assert!(errs.is_empty());
    }

    #[test]
    fn deferred_property_error() {
        let m = map(&[("position", "absolute")]);
        let (_, errs) = parse_style(&m);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].property, "position");
    }

    #[test]
    fn deferred_display_value_error() {
        let m = map(&[("display", "grid")]);
        let (_, errs) = parse_style(&m);
        assert_eq!(errs.len(), 1);
        assert!(errs[0].reason.contains("R9"));
    }

    #[test]
    fn unknown_property_silent() {
        let m = map(&[("some-vendor-prop", "value")]);
        let (_, errs) = parse_style(&m);
        assert!(errs.is_empty());
    }

    #[test]
    fn malformed_value_emits_error_keeps_default() {
        let m = map(&[("width", "abc")]);
        let (s, errs) = parse_style(&m);
        assert_eq!(errs.len(), 1);
        assert_eq!(s.width, None);
    }
}
// CODEGEN-END
