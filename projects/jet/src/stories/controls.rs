// <HANDWRITE gap="missing-generator:logic:604ec8f4" tracker="standardize-gap-projects-jet-src-stories-controls-rs" reason="Map a prop type to a control kind (bool->toggle, string->text, number->number, string-literal union->select with options, else text), then apply meta.argTypes overrides (control type/options/disable). Returns the resolved control descriptors for a story.">
//! Control inference + resolution for `jet stories` (B3).
//!
//! This module is pure (no I/O, no AST) so it is trivially unit-testable. It
//! turns the props of a component ([`super::prop_extractor::PropDef`]) plus the
//! story's `meta.argTypes` into an ordered list of [`Control`]s the manager UI
//! renders, each seeded with the story's current arg value.
//!
//! ## Inference rules ([`infer_control`])
//!
//! The control kind is inferred from the prop's TS type:
//!
//! | TS type                        | control          |
//! |--------------------------------|------------------|
//! | `boolean`                      | [`ControlKind::Toggle`] |
//! | `number`                       | [`ControlKind::Number`] |
//! | string-literal union `"a"\|"b"`| [`ControlKind::Select`] `{["a","b"]}` |
//! | `string`                       | [`ControlKind::Text`]   |
//! | anything else                  | [`ControlKind::Text`]   (fallback) |
//!
//! ## Override resolution ([`resolve_controls`])
//!
//! `meta.argTypes[name]` overrides inference. A recognized argType may:
//! - set `control: { type, options }` (e.g. `{ type: 'select', options: [...] }`),
//! - set `control: 'select'` (shorthand string form),
//! - set `control: false` to *disable* the control (the prop gets no widget),
//! - set `disable: true` (Storybook's hide-from-table flag) — also disabled.
//!
//! When an argType supplies `options` without a control type (or with
//! `type: 'select' | 'radio'`), the control becomes a [`ControlKind::Select`].

use std::collections::BTreeMap;

use super::csf::CsfValue;
use super::prop_extractor::PropDef;

/// The kind of editable widget a control renders as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlKind {
    /// A checkbox for a `boolean` prop.
    Toggle,
    /// A free-text input for a `string` (or unrecognized) prop.
    Text,
    /// A numeric input for a `number` prop.
    Number,
    /// A color picker for CSS color strings.
    Color,
    /// A date picker whose DOM value is an ISO date string.
    Date,
    /// A numeric range slider.
    Range {
        min: Option<String>,
        max: Option<String>,
        step: Option<String>,
    },
    /// A JSON textarea for object/array-like args.
    Object,
    /// A radio group.
    Radio { options: Vec<String>, inline: bool },
    /// A checkbox group.
    Check { options: Vec<String>, inline: bool },
    /// A multi-select list.
    MultiSelect { options: Vec<String> },
    /// A file input.
    File,
    /// A dropdown for a string-literal union / argType `options` list.
    Select {
        /// The selectable option values, in declared order.
        options: Vec<String>,
    },
}

/// One resolved control for a single prop, ready to render.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    /// The prop / arg name the control edits.
    pub name: String,
    /// The widget kind (after applying argType overrides).
    pub kind: ControlKind,
    /// The story's current value for this arg, if any (seeds the widget). This
    /// is the merged story arg ([`super::StoryEntry::args`]), not a default.
    pub current: Option<CsfValue>,
    /// Optional per-option labels from `argTypes[name].labels`.
    pub labels: BTreeMap<String, String>,
    /// Optional per-option value mapping from `argTypes[name].mapping`.
    pub mapping: BTreeMap<String, CsfValue>,
}

/// Infer a control kind from a prop's TS type text alone (no overrides).
///
/// String-literal unions (`"sm" | "lg"`) become a [`ControlKind::Select`] whose
/// options are the unquoted literals, in order. Everything non-`boolean`,
/// non-`number`, non-union falls back to [`ControlKind::Text`].
pub fn infer_control(prop: &PropDef) -> ControlKind {
    let ty = prop.type_text.trim();

    if ty == "boolean" {
        return ControlKind::Toggle;
    }
    if ty == "number" {
        return ControlKind::Number;
    }
    // String-literal union: `"sm" | "lg"` / `'sm' | 'lg'`. Require every member
    // to be a quoted string literal; a mixed union (`string | number`) is Text.
    if let Some(options) = string_literal_union(ty) {
        return ControlKind::Select { options };
    }
    if ty == "string" {
        return ControlKind::Text;
    }
    // TODO(#175 follow-up): enums, numeric-literal unions, object/array props,
    // and union types mixing literals with `undefined`/`null` fall back to Text.
    ControlKind::Text
}

/// Parse a `"a" | "b" | "c"` string-literal union into its unquoted options.
///
/// Returns `None` unless there are ≥2 members and *every* member is a quoted
/// string literal (so `string | undefined` and `"a" | number` are not selects).
fn string_literal_union(ty: &str) -> Option<Vec<String>> {
    if !ty.contains('|') {
        return None;
    }
    let mut options = Vec::new();
    for raw in ty.split('|') {
        let member = raw.trim();
        let unquoted = strip_string_literal(member)?;
        options.push(unquoted);
    }
    if options.len() >= 2 {
        Some(options)
    } else {
        None
    }
}

/// Unquote a `"x"` / `'x'` string literal; `None` if `member` is not a quoted
/// string literal.
fn strip_string_literal(member: &str) -> Option<String> {
    let bytes = member.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return Some(member[1..member.len() - 1].to_string());
        }
    }
    None
}

/// Resolve the ordered controls for a story: infer from props, override from
/// `arg_types`, seed each with the story's current arg value.
///
/// Props whose argType *disables* the control (`control: false` /
/// `disable: true`) are omitted entirely. Order follows the props order so the
/// panel mirrors the component's declared prop order.
pub fn resolve_controls(
    props: &[PropDef],
    arg_types: &BTreeMap<String, CsfValue>,
    args: &BTreeMap<String, CsfValue>,
) -> Vec<Control> {
    let mut out = Vec::new();
    for prop in props {
        let arg_type = arg_types.get(&prop.name);

        // A disabled argType removes the control entirely.
        if arg_type.map(arg_type_is_disabled).unwrap_or(false) {
            continue;
        }

        let metadata = arg_type.map(control_metadata_from_arg_type);
        // argType override wins over inference; otherwise infer from the type.
        let kind = metadata
            .as_ref()
            .and_then(|m| m.kind.clone())
            .unwrap_or_else(|| infer_control(prop));

        out.push(Control {
            name: prop.name.clone(),
            kind,
            current: args.get(&prop.name).cloned(),
            labels: metadata
                .as_ref()
                .map(|m| m.labels.clone())
                .unwrap_or_default(),
            mapping: metadata.map(|m| m.mapping).unwrap_or_default(),
        });
    }
    out
}

#[derive(Debug, Clone, Default)]
struct ControlMetadata {
    kind: Option<ControlKind>,
    labels: BTreeMap<String, String>,
    mapping: BTreeMap<String, CsfValue>,
}

/// True when an argType disables its control: `control: false`, or
/// `disable: true` (Storybook's hide flag), or `table: { disable: true }`.
fn arg_type_is_disabled(arg_type: &CsfValue) -> bool {
    let CsfValue::Object(map) = arg_type else {
        return false;
    };
    if matches!(map.get("control"), Some(CsfValue::Bool(false))) {
        return true;
    }
    if matches!(map.get("disable"), Some(CsfValue::Bool(true))) {
        return true;
    }
    if let Some(CsfValue::Object(table)) = map.get("table") {
        if matches!(table.get("disable"), Some(CsfValue::Bool(true))) {
            return true;
        }
    }
    false
}

/// Derive control metadata from an argType's `control` / `options` / `labels` /
/// `mapping` fields. Missing `kind` means "fall back to type inference".
fn control_metadata_from_arg_type(arg_type: &CsfValue) -> ControlMetadata {
    let CsfValue::Object(map) = arg_type else {
        return ControlMetadata::default();
    };

    // `options` may live at the argType root (`{ options: [...], control: 'select' }`).
    let root_options = map.get("options").and_then(csf_string_list);
    let labels = map
        .get("labels")
        .and_then(csf_string_map)
        .unwrap_or_default();
    let mapping = map
        .get("mapping")
        .and_then(csf_value_map)
        .unwrap_or_default();

    let kind = match map.get("control") {
        // `control: 'text' | 'boolean' | 'number' | 'select' | 'radio'`.
        Some(CsfValue::Str(kind)) => Some(control_from_type_name(kind, root_options, None)),
        // `control: { type: 'select', options: [...] }`.
        Some(CsfValue::Object(control)) => {
            let inner_options = control.get("options").and_then(csf_string_list);
            let options = inner_options.or(root_options);
            let range = RangeConfig {
                min: control.get("min").and_then(csf_scalar_string),
                max: control.get("max").and_then(csf_scalar_string),
                step: control.get("step").and_then(csf_scalar_string),
            };
            match control.get("type") {
                Some(CsfValue::Str(kind)) => {
                    Some(control_from_type_name(kind, options, Some(range)))
                }
                // No explicit type but options present → a select.
                _ => options.map(|opts| ControlKind::Select { options: opts }),
            }
        }
        // No `control` field but root `options` present → a select.
        _ => root_options.map(|opts| ControlKind::Select { options: opts }),
    };

    ControlMetadata {
        kind,
        labels,
        mapping,
    }
}

/// Map an argType control type *name* to a [`ControlKind`].
///
/// `select` / `radio` / `check` / `inline-radio` need options; if none are
/// supplied the control degrades to [`ControlKind::Text`] rather than an empty
/// dropdown.
fn control_from_type_name(
    name: &str,
    options: Option<Vec<String>>,
    range: Option<RangeConfig>,
) -> ControlKind {
    match name {
        "boolean" => ControlKind::Toggle,
        "number" => ControlKind::Number,
        "range" => {
            let range = range.unwrap_or_default();
            ControlKind::Range {
                min: range.min,
                max: range.max,
                step: range.step,
            }
        }
        "text" => ControlKind::Text,
        "color" => ControlKind::Color,
        "date" => ControlKind::Date,
        "object" => ControlKind::Object,
        "file" => ControlKind::File,
        "select" => options
            .filter(|opts| !opts.is_empty())
            .map(|options| ControlKind::Select { options })
            .unwrap_or(ControlKind::Text),
        "radio" | "inline-radio" => options
            .filter(|opts| !opts.is_empty())
            .map(|options| ControlKind::Radio {
                options,
                inline: name == "inline-radio",
            })
            .unwrap_or(ControlKind::Text),
        "check" | "inline-check" => options
            .filter(|opts| !opts.is_empty())
            .map(|options| ControlKind::Check {
                options,
                inline: name == "inline-check",
            })
            .unwrap_or(ControlKind::Text),
        "multi-select" => options
            .filter(|opts| !opts.is_empty())
            .map(|options| ControlKind::MultiSelect { options })
            .unwrap_or(ControlKind::Text),
        // Unknown control type → text input (safe default).
        _ => ControlKind::Text,
    }
}

#[derive(Debug, Clone, Default)]
struct RangeConfig {
    min: Option<String>,
    max: Option<String>,
    step: Option<String>,
}

/// Read a `CsfValue` as a list of string options. Accepts a raw array literal
/// (`["sm","lg"]`, kept as [`CsfValue::Raw`] by the CSF parser) or an object of
/// values. Returns `None` when it can't be read as a list.
fn csf_string_list(value: &CsfValue) -> Option<Vec<String>> {
    match value {
        // The CSF parser keeps array literals as Raw source (`["sm", "lg"]`).
        CsfValue::Raw(raw) => parse_array_literal(raw),
        _ => None,
    }
}

fn csf_string_map(value: &CsfValue) -> Option<BTreeMap<String, String>> {
    let CsfValue::Object(map) = value else {
        return None;
    };
    let mut out = BTreeMap::new();
    for (key, value) in map {
        out.insert(key.clone(), csf_scalar_string(value)?);
    }
    Some(out)
}

fn csf_value_map(value: &CsfValue) -> Option<BTreeMap<String, CsfValue>> {
    let CsfValue::Object(map) = value else {
        return None;
    };
    Some(map.clone())
}

fn csf_scalar_string(value: &CsfValue) -> Option<String> {
    match value {
        CsfValue::Str(s) => Some(s.clone()),
        CsfValue::Bool(b) => Some(b.to_string()),
        CsfValue::Number(n) => Some(n.clone()),
        CsfValue::Raw(raw) => Some(raw.trim_matches(['"', '\'']).to_string()),
        CsfValue::Null | CsfValue::Object(_) => None,
    }
}

/// Parse a JS array-literal *source slice* (`["sm", "lg"]` / `['a','b']`) into
/// its string elements. Only string-literal elements are extracted; non-string
/// elements are kept by their trimmed source text so numeric option lists still
/// round-trip as labels.
fn parse_array_literal(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    let mut out = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let value = strip_string_literal(part).unwrap_or_else(|| part.to_string());
        out.push(value);
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prop(name: &str, ty: &str, optional: bool) -> PropDef {
        PropDef {
            name: name.to_string(),
            type_text: ty.to_string(),
            optional,
            description: String::new(),
            default_value: None,
        }
    }

    #[test]
    fn infers_basic_kinds() {
        assert_eq!(
            infer_control(&prop("p", "boolean", false)),
            ControlKind::Toggle
        );
        assert_eq!(
            infer_control(&prop("p", "string", false)),
            ControlKind::Text
        );
        assert_eq!(
            infer_control(&prop("p", "number", false)),
            ControlKind::Number
        );
    }

    #[test]
    fn infers_select_from_string_literal_union() {
        let kind = infer_control(&prop("size", "\"sm\" | \"lg\"", false));
        assert_eq!(
            kind,
            ControlKind::Select {
                options: vec!["sm".to_string(), "lg".to_string()]
            }
        );
    }

    #[test]
    fn mixed_union_falls_back_to_text() {
        assert_eq!(
            infer_control(&prop("p", "string | number", false)),
            ControlKind::Text
        );
        assert_eq!(
            infer_control(&prop("p", "\"a\" | number", false)),
            ControlKind::Text
        );
    }

    #[test]
    fn resolve_seeds_current_values() {
        let props = vec![
            prop("primary", "boolean", false),
            prop("label", "string", false),
        ];
        let mut args = BTreeMap::new();
        args.insert("primary".to_string(), CsfValue::Bool(true));
        args.insert("label".to_string(), CsfValue::Str("Hi".into()));
        let controls = resolve_controls(&props, &BTreeMap::new(), &args);
        assert_eq!(controls.len(), 2);
        assert_eq!(controls[0].current, Some(CsfValue::Bool(true)));
        assert_eq!(controls[1].current, Some(CsfValue::Str("Hi".into())));
    }

    #[test]
    fn arg_type_object_control_overrides_inference() {
        // size inferred as Text (plain string), but argType forces a select.
        let props = vec![prop("size", "string", false)];
        let mut control_obj = BTreeMap::new();
        control_obj.insert("type".to_string(), CsfValue::Str("select".into()));
        control_obj.insert(
            "options".to_string(),
            CsfValue::Raw("[\"sm\", \"lg\"]".into()),
        );
        let mut arg_types = BTreeMap::new();
        arg_types.insert(
            "size".to_string(),
            CsfValue::Object({
                let mut m = BTreeMap::new();
                m.insert("control".to_string(), CsfValue::Object(control_obj));
                m
            }),
        );
        let controls = resolve_controls(&props, &arg_types, &BTreeMap::new());
        assert_eq!(
            controls[0].kind,
            ControlKind::Select {
                options: vec!["sm".to_string(), "lg".to_string()]
            }
        );
    }

    #[test]
    fn arg_type_control_false_disables() {
        let props = vec![
            prop("hidden", "boolean", false),
            prop("shown", "string", false),
        ];
        let mut arg_types = BTreeMap::new();
        arg_types.insert(
            "hidden".to_string(),
            CsfValue::Object({
                let mut m = BTreeMap::new();
                m.insert("control".to_string(), CsfValue::Bool(false));
                m
            }),
        );
        let controls = resolve_controls(&props, &arg_types, &BTreeMap::new());
        assert_eq!(controls.len(), 1, "disabled control omitted");
        assert_eq!(controls[0].name, "shown");
    }

    #[test]
    fn arg_type_full_control_types_labels_mapping_and_range_config() {
        let props = vec![
            prop("color", "string", false),
            prop("date", "string", false),
            prop("range", "number", false),
            prop("object", "Record<string, string>", false),
            prop("radio", "string", false),
            prop("check", "string[]", false),
            prop("multi", "string[]", false),
            prop("file", "string", false),
        ];
        let mut arg_types = BTreeMap::new();
        arg_types.insert("color".into(), arg_type_with_control("color"));
        arg_types.insert("date".into(), arg_type_with_control("date"));
        arg_types.insert(
            "range".into(),
            CsfValue::Object({
                let mut control = BTreeMap::new();
                control.insert("type".into(), CsfValue::Str("range".into()));
                control.insert("min".into(), CsfValue::Number("0".into()));
                control.insert("max".into(), CsfValue::Number("10".into()));
                control.insert("step".into(), CsfValue::Number("2".into()));
                let mut arg_type = BTreeMap::new();
                arg_type.insert("control".into(), CsfValue::Object(control));
                arg_type
            }),
        );
        arg_types.insert("object".into(), arg_type_with_control("object"));
        arg_types.insert(
            "radio".into(),
            choice_arg_type("inline-radio", "[\"sm\", \"lg\"]"),
        );
        arg_types.insert("check".into(), choice_arg_type("check", "[\"a\", \"b\"]"));
        arg_types.insert(
            "multi".into(),
            choice_arg_type("multi-select", "[\"x\", \"y\"]"),
        );
        arg_types.insert("file".into(), arg_type_with_control("file"));

        let controls = resolve_controls(&props, &arg_types, &BTreeMap::new());
        let kinds: Vec<_> = controls.iter().map(|c| &c.kind).collect();
        assert!(matches!(kinds[0], ControlKind::Color));
        assert!(matches!(kinds[1], ControlKind::Date));
        assert_eq!(
            kinds[2],
            &ControlKind::Range {
                min: Some("0".into()),
                max: Some("10".into()),
                step: Some("2".into())
            }
        );
        assert!(matches!(kinds[3], ControlKind::Object));
        assert_eq!(
            kinds[4],
            &ControlKind::Radio {
                options: vec!["sm".into(), "lg".into()],
                inline: true
            }
        );
        assert_eq!(
            kinds[5],
            &ControlKind::Check {
                options: vec!["a".into(), "b".into()],
                inline: false
            }
        );
        assert_eq!(
            kinds[6],
            &ControlKind::MultiSelect {
                options: vec!["x".into(), "y".into()]
            }
        );
        assert!(matches!(kinds[7], ControlKind::File));
    }

    #[test]
    fn arg_type_labels_and_mapping_are_preserved() {
        let props = vec![prop("size", "string", false)];
        let mut labels = BTreeMap::new();
        labels.insert("sm".into(), CsfValue::Str("Small".into()));
        labels.insert("lg".into(), CsfValue::Str("Large".into()));
        let mut mapping = BTreeMap::new();
        mapping.insert("sm".into(), CsfValue::Number("1".into()));
        mapping.insert("lg".into(), CsfValue::Number("2".into()));

        let mut arg_type = BTreeMap::new();
        arg_type.insert("control".into(), CsfValue::Str("radio".into()));
        arg_type.insert("options".into(), CsfValue::Raw("[\"sm\", \"lg\"]".into()));
        arg_type.insert("labels".into(), CsfValue::Object(labels));
        arg_type.insert("mapping".into(), CsfValue::Object(mapping));
        let mut arg_types = BTreeMap::new();
        arg_types.insert("size".into(), CsfValue::Object(arg_type));

        let controls = resolve_controls(&props, &arg_types, &BTreeMap::new());
        assert_eq!(
            controls[0].labels.get("sm").map(String::as_str),
            Some("Small")
        );
        assert_eq!(
            controls[0].mapping.get("lg"),
            Some(&CsfValue::Number("2".into()))
        );
    }

    fn arg_type_with_control(kind: &str) -> CsfValue {
        CsfValue::Object({
            let mut m = BTreeMap::new();
            m.insert("control".into(), CsfValue::Str(kind.into()));
            m
        })
    }

    fn choice_arg_type(kind: &str, options: &str) -> CsfValue {
        CsfValue::Object({
            let mut m = BTreeMap::new();
            m.insert("control".into(), CsfValue::Str(kind.into()));
            m.insert("options".into(), CsfValue::Raw(options.into()));
            m
        })
    }
}
// </HANDWRITE>
