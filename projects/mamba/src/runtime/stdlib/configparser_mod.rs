use super::super::dict_ops::DictKey;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// configparser module for Mamba (#mamba-stdlib).
///
/// A self-contained, native implementation of CPython 3.12's `configparser`
/// surface used by the behavior fixtures:
///
///   * `ConfigParser` / `RawConfigParser` / `SafeConfigParser` — instance with
///     `read_string` / `read_file` / `read` / `read_dict`, `sections` /
///     `add_section` / `has_section` / `options` / `has_option`, `get` /
///     `getint` / `getfloat` / `getboolean`, `items`, `set`, `remove_option` /
///     `remove_section`, `defaults`, `write`, `clear`, plus the mapping
///     protocol (`__getitem__` / `__setitem__` / `__contains__` / `__iter__` /
///     `__len__`).
///   * `SectionProxy` — `cp["section"]` returns a live proxy supporting
///     `[]` access, `get` / `getint` / `getfloat` / `getboolean`, `__contains__`,
///     `name`, and `<Section: name>` repr, plus synthesized converter accessors.
///   * `BasicInterpolation` (`%(name)s`) and `ExtendedInterpolation`
///     (`${section:name}`) — selected via the `interpolation=` kwarg.
///   * Constructor kwargs: `delimiters`, `comment_prefixes`,
///     `inline_comment_prefixes`, `allow_no_value`, `strict`,
///     `default_section`, `interpolation`, `defaults`.
///
/// Instance methods are registered as runtime classes via `mb_class_register`,
/// so `cp.read_string(...)` / `cp["x"]` dispatch through the normal MRO path
/// with no class.rs changes.
use std::collections::HashMap;

const PARSER_CLASS: &str = "ConfigParser";
const PROXY_CLASS: &str = "SectionProxy";

// ── Small helpers ──────────────────────────────────────────────────────────

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}
fn new_list(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}
fn new_dict() -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn seq_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

fn is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Split a method's variadic args list into (positional, kwargs-dict).
/// When any kwarg is supplied the runtime appends a trailing dict.
fn split_args(args: MbValue) -> (Vec<MbValue>, MbValue) {
    let items = seq_items(args);
    if let Some(last) = items.last() {
        if is_dict(*last) {
            let n = items.len();
            return (items[..n - 1].to_vec(), *last);
        }
    }
    (items, MbValue::none())
}

fn kw_get(kwargs: MbValue, name: &str) -> Option<MbValue> {
    if let Some(ptr) = kwargs.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                return lock.read().unwrap().get(name).copied();
            }
        }
    }
    None
}

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn instance_class(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

// ── Ordered string-keyed dict access (sections / options are mamba Dicts) ──

/// Read the value for `key` (a Str key) from a mamba Dict.
fn dict_get_str(d: MbValue, key: &str) -> Option<MbValue> {
    d.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn dict_set_str(d: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = d.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = lock
                    .write()
                    .unwrap()
                    .insert(DictKey::Str(key.to_string()), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn dict_remove_str(d: MbValue, key: &str) -> bool {
    if let Some(ptr) = d.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let removed = lock
                    .write()
                    .unwrap()
                    .shift_remove(&DictKey::Str(key.to_string()));
                if let Some(p) = removed {
                    super::super::rc::release_if_ptr(p);
                    return true;
                }
            }
        }
    }
    false
}

fn dict_contains_str(d: MbValue, key: &str) -> bool {
    d.as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().contains_key(key)
            } else {
                false
            }
        })
        .unwrap_or(false)
}

/// Ordered list of Str keys in a mamba Dict.
fn dict_str_keys(d: MbValue) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(ptr) = d.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                for k in lock.read().unwrap().keys() {
                    if let DictKey::Str(s) = k {
                        out.push(s.clone());
                    }
                }
            }
        }
    }
    out
}

/// Snapshot a section's options (folded-key → value-string) preserving order.
fn dict_str_pairs(d: MbValue) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    if let Some(ptr) = d.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                for (k, v) in lock.read().unwrap().iter() {
                    if let DictKey::Str(s) = k {
                        let val = if v.is_none() { None } else { extract_str(*v) };
                        out.push((s.clone(), val));
                    }
                }
            }
        }
    }
    out
}

// ── Errors ──────────────────────────────────────────────────────────────────

fn raise_named(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
    MbValue::none()
}
fn raise_type_error(msg: &str) -> MbValue {
    raise_named("TypeError", msg)
}
fn raise_value_error(msg: &str) -> MbValue {
    raise_named("ValueError", msg)
}

// ── optionxform ─────────────────────────────────────────────────────────────

fn optionxform(key: &str) -> String {
    key.to_lowercase()
}

// ── Constructor configuration ──────────────────────────────────────────────

fn default_section_name(parser: MbValue) -> String {
    get_field(parser, "_default_section")
        .and_then(extract_str)
        .unwrap_or_else(|| "DEFAULT".to_string())
}
fn interp_kind(parser: MbValue) -> String {
    get_field(parser, "_interp")
        .and_then(extract_str)
        .unwrap_or_else(|| "basic".to_string())
}
fn allow_no_value(parser: MbValue) -> bool {
    get_field(parser, "_allow_no_value")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
fn is_strict(parser: MbValue) -> bool {
    get_field(parser, "_strict")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

fn delimiters(parser: MbValue) -> Vec<String> {
    match get_field(parser, "_delimiters") {
        Some(v) => {
            let mut out: Vec<String> = seq_items(v)
                .iter()
                .filter_map(|x| extract_str(*x))
                .collect();
            if out.is_empty() {
                out = vec!["=".to_string(), ":".to_string()];
            }
            out
        }
        None => vec!["=".to_string(), ":".to_string()],
    }
}
fn comment_prefixes(parser: MbValue) -> Vec<String> {
    match get_field(parser, "_comment_prefixes") {
        Some(v) => seq_items(v)
            .iter()
            .filter_map(|x| extract_str(*x))
            .collect(),
        None => vec!["#".to_string(), ";".to_string()],
    }
}
fn inline_comment_prefixes(parser: MbValue) -> Vec<String> {
    match get_field(parser, "_inline_comment_prefixes") {
        Some(v) => seq_items(v)
            .iter()
            .filter_map(|x| extract_str(*x))
            .collect(),
        None => Vec::new(),
    }
}

// ── Sections store ─────────────────────────────────────────────────────────

fn sections_dict(parser: MbValue) -> MbValue {
    get_field(parser, "_sections").unwrap_or_else(MbValue::none)
}
fn defaults_dict(parser: MbValue) -> MbValue {
    get_field(parser, "_defaults").unwrap_or_else(MbValue::none)
}

/// Get the option-dict for a section name (DEFAULT routes to `_defaults`).
/// Returns None if the section does not exist.
fn section_options(parser: MbValue, sec: &str) -> Option<MbValue> {
    if sec == default_section_name(parser) {
        return Some(defaults_dict(parser));
    }
    dict_get_str(sections_dict(parser), sec)
}

fn has_section(parser: MbValue, sec: &str) -> bool {
    if sec == default_section_name(parser) {
        return false;
    }
    dict_contains_str(sections_dict(parser), sec)
}

/// Effective options for `sec`: DEFAULT options overlaid with own options.
fn merged_options(parser: MbValue, sec: &str) -> Vec<(String, Option<String>)> {
    let dflt = default_section_name(parser);
    let mut map: indexmap::IndexMap<String, Option<String>> = indexmap::IndexMap::new();
    for (k, v) in dict_str_pairs(defaults_dict(parser)) {
        map.insert(k, v);
    }
    if sec != dflt {
        if let Some(d) = dict_get_str(sections_dict(parser), sec) {
            for (k, v) in dict_str_pairs(d) {
                map.insert(k, v);
            }
        }
    }
    map.into_iter().collect()
}

fn lookup_raw(parser: MbValue, sec: &str, key_folded: &str) -> Option<Option<String>> {
    for (k, v) in merged_options(parser, sec) {
        if k == key_folded {
            return Some(v);
        }
    }
    None
}

// ── Interpolation ──────────────────────────────────────────────────────────

fn opts_map(parser: MbValue, sec: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for (k, v) in merged_options(parser, sec) {
        if let Some(s) = v {
            m.insert(k, s);
        }
    }
    m
}

/// BasicInterpolation: `%(name)s`, `%%` → `%`.
fn interpolate_basic(
    value: &str,
    opts: &HashMap<String, String>,
    depth: usize,
) -> Result<String, (&'static str, String)> {
    if depth > 10 {
        return Err((
            "InterpolationDepthError",
            "interpolation reached the maximum depth".to_string(),
        ));
    }
    if !value.contains('%') {
        return Ok(value.to_string());
    }
    let bytes = value.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 1 >= bytes.len() {
                return Err((
                    "InterpolationSyntaxError",
                    "'%' must be followed by '%' or '(', found: '%'".to_string(),
                ));
            }
            match bytes[i + 1] {
                b'%' => {
                    out.push('%');
                    i += 2;
                }
                b'(' => {
                    let rest = &value[i + 2..];
                    if let Some(close) = rest.find(')') {
                        let name = &rest[..close];
                        let after = i + 2 + close + 1;
                        if after >= bytes.len() || bytes[after] != b's' {
                            return Err((
                                "InterpolationSyntaxError",
                                "bad interpolation variable reference".to_string(),
                            ));
                        }
                        let folded = optionxform(name);
                        match opts.get(&folded) {
                            Some(referent) => {
                                let expanded = interpolate_basic(referent, opts, depth + 1)?;
                                out.push_str(&expanded);
                            }
                            None => {
                                return Err(("InterpolationMissingOptionError",
                                    format!("Bad value substitution: option references nonexistent option '{}'", folded)));
                            }
                        }
                        i = after + 1;
                    } else {
                        return Err((
                            "InterpolationSyntaxError",
                            "bad interpolation variable reference".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err((
                        "InterpolationSyntaxError",
                        format!(
                            "'%' must be followed by '%' or '(', found: {:?}",
                            &value[i..(i + 2).min(value.len())]
                        ),
                    ));
                }
            }
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'%' {
                i += 1;
            }
            out.push_str(&value[start..i]);
        }
    }
    Ok(out)
}

/// ExtendedInterpolation: `${name}` (same section) / `${section:name}`, `$$` → `$`.
fn interpolate_extended(
    parser: MbValue,
    cur_sec: &str,
    value: &str,
    depth: usize,
) -> Result<String, (&'static str, String)> {
    if depth > 10 {
        return Err((
            "InterpolationDepthError",
            "interpolation reached the maximum depth".to_string(),
        ));
    }
    if !value.contains('$') {
        return Ok(value.to_string());
    }
    let bytes = value.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            if i + 1 >= bytes.len() {
                return Err((
                    "InterpolationSyntaxError",
                    "'$' must be followed by '$' or '{', found: '$'".to_string(),
                ));
            }
            match bytes[i + 1] {
                b'$' => {
                    out.push('$');
                    i += 2;
                }
                b'{' => {
                    let rest = &value[i + 2..];
                    if let Some(close) = rest.find('}') {
                        let inner = &rest[..close];
                        let (ref_sec, ref_key) = match inner.find(':') {
                            Some(p) => (inner[..p].to_string(), inner[p + 1..].to_string()),
                            None => (cur_sec.to_string(), inner.to_string()),
                        };
                        let folded = optionxform(&ref_key);
                        match lookup_raw(parser, &ref_sec, &folded) {
                            Some(Some(referent)) => {
                                let expanded =
                                    interpolate_extended(parser, &ref_sec, &referent, depth + 1)?;
                                out.push_str(&expanded);
                            }
                            Some(None) => {}
                            None => {
                                return Err(("InterpolationMissingOptionError",
                                    format!("Bad value substitution: option references nonexistent option '{}'", folded)));
                            }
                        }
                        i = i + 2 + close + 1;
                    } else {
                        return Err((
                            "InterpolationSyntaxError",
                            "bad interpolation variable reference".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err((
                        "InterpolationSyntaxError",
                        format!(
                            "'$' must be followed by '$' or '{{', found: {:?}",
                            &value[i..(i + 2).min(value.len())]
                        ),
                    ));
                }
            }
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'$' {
                i += 1;
            }
            out.push_str(&value[start..i]);
        }
    }
    Ok(out)
}

/// Resolve a raw option value with the parser's interpolation engine.
fn do_interpolate(parser: MbValue, sec: &str, raw: &str) -> Result<String, (&'static str, String)> {
    match interp_kind(parser).as_str() {
        "none" | "raw" => Ok(raw.to_string()),
        "extended" => interpolate_extended(parser, sec, raw, 1),
        _ => interpolate_basic(raw, &opts_map(parser, sec), 1),
    }
}

// ── INI parser ─────────────────────────────────────────────────────────────

/// Strip an inline comment from a value given the inline prefixes. CPython
/// strips at a prefix that is *preceded by whitespace* (or at the very start).
fn strip_inline_comment(value: &str, prefixes: &[String]) -> String {
    if prefixes.is_empty() {
        return value.to_string();
    }
    let bytes = value.as_bytes();
    let mut cut: Option<usize> = None;
    let mut idx = 0;
    while idx < value.len() {
        for p in prefixes {
            if value[idx..].starts_with(p.as_str()) {
                let preceded_by_ws = idx == 0 || bytes[idx - 1] == b' ' || bytes[idx - 1] == b'\t';
                if preceded_by_ws {
                    cut = Some(idx);
                    break;
                }
            }
        }
        if cut.is_some() {
            break;
        }
        idx += 1;
    }
    match cut {
        Some(c) => value[..c].trim_end().to_string(),
        None => value.to_string(),
    }
}

fn is_full_comment(line: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|p| line.starts_with(p.as_str()))
}

fn first_delimiter(line: &str, delims: &[String]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for d in delims {
        if let Some(pos) = line.find(d.as_str()) {
            best = Some(best.map_or(pos, |b| b.min(pos)));
        }
    }
    best
}

/// Parse INI `text` into the parser. `source` names the file for errors.
fn parse_string(parser: MbValue, text: &str, source: &str) -> MbValue {
    let dflt = default_section_name(parser);
    let comment_pref = comment_prefixes(parser);
    let inline_pref = inline_comment_prefixes(parser);
    let delims = delimiters(parser);
    let allow_nv = allow_no_value(parser);
    let strict = is_strict(parser);

    let secs = sections_dict(parser);
    let defaults = defaults_dict(parser);

    let mut cur_sec: Option<String> = None;
    let mut cur_opts: Option<MbValue> = None;
    // For multiline continuation.
    let mut last_key: Option<String> = None;

    // Track seen sections/options for strict-mode duplicate detection.
    let mut seen_sections: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_options: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();

    let lines: Vec<&str> = text.split('\n').collect();
    let n = lines.len();
    for (idx0, raw_line) in lines.iter().enumerate() {
        // Drop a trailing '\r' (universal newlines).
        let raw_line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let lineno = idx0 + 1;
        // The very last element of split('\n') is "" for a trailing newline.
        if idx0 == n - 1 && raw_line.is_empty() {
            continue;
        }

        let stripped = raw_line.trim();

        // Full-line comment / blank.
        if stripped.is_empty() || is_full_comment(stripped, &comment_pref) {
            // A blank line ends a multiline value continuation.
            if stripped.is_empty() {
                last_key = None;
            }
            continue;
        }

        // Multiline continuation: line begins with whitespace and we have an
        // option pending.
        let indented = raw_line.starts_with(' ') || raw_line.starts_with('\t');
        if indented && last_key.is_some() && cur_opts.is_some() {
            let cont = strip_inline_comment(stripped, &inline_pref);
            let opts = cur_opts.unwrap();
            let key = last_key.clone().unwrap();
            if let Some(existing) = dict_get_str(opts, &key) {
                let mut s = extract_str(existing).unwrap_or_default();
                s.push('\n');
                s.push_str(&cont);
                dict_set_str(opts, &key, new_str(&s));
            }
            cur_opts = Some(opts);
            continue;
        }

        // Section header.
        if stripped.starts_with('[') {
            if let Some(close) = stripped.find(']') {
                let name = stripped[1..close].to_string();
                if strict && seen_sections.contains(&name) {
                    return raise_named(
                        "DuplicateSectionError",
                        &format!(
                            "While reading from {:?} [line {:2}]: section {:?} already exists",
                            source, lineno, name
                        ),
                    );
                }
                seen_sections.insert(name.clone());
                last_key = None;
                if name == dflt {
                    cur_sec = Some(name.clone());
                    cur_opts = Some(defaults);
                } else {
                    if !dict_contains_str(secs, &name) {
                        dict_set_str(secs, &name, new_dict());
                    }
                    cur_sec = Some(name.clone());
                    cur_opts = dict_get_str(secs, &name);
                }
                continue;
            } else {
                return raise_named(
                    "MissingSectionHeaderError",
                    &format!(
                        "File contains no section headers.\nfile: {:?}, line: {}\n{:?}",
                        source, lineno, raw_line
                    ),
                );
            }
        }

        // Content line before any section header.
        if cur_opts.is_none() {
            return raise_named(
                "MissingSectionHeaderError",
                &format!(
                    "File contains no section headers.\nfile: {:?}, line: {}\n{:?}",
                    source, lineno, raw_line
                ),
            );
        }

        // key = value (delimiter split). Strip inline comment first.
        let value_line = strip_inline_comment(stripped, &inline_pref);
        if let Some(dpos) = first_delimiter(&value_line, &delims) {
            let raw_key = value_line[..dpos].trim();
            let key = optionxform(raw_key);
            let val = value_line[dpos + 1..].trim().to_string();
            let opts = cur_opts.unwrap();
            let sec_name = cur_sec.clone().unwrap_or_default();
            if strict {
                let set = seen_options.entry(sec_name.clone()).or_default();
                if set.contains(&key) {
                    return raise_named("DuplicateOptionError",
                        &format!("While reading from {:?} [line {:2}]: option {:?} in section {:?} already exists", source, lineno, key, sec_name));
                }
                set.insert(key.clone());
            }
            dict_set_str(opts, &key, new_str(&val));
            last_key = Some(key);
            cur_opts = Some(opts);
        } else if allow_nv {
            // Key with no value (allow_no_value).
            let key = optionxform(value_line.trim());
            let opts = cur_opts.unwrap();
            let sec_name = cur_sec.clone().unwrap_or_default();
            if strict {
                let set = seen_options.entry(sec_name.clone()).or_default();
                if set.contains(&key) {
                    return raise_named("DuplicateOptionError",
                        &format!("While reading from {:?} [line {:2}]: option {:?} in section {:?} already exists", source, lineno, key, sec_name));
                }
                set.insert(key.clone());
            }
            dict_set_str(opts, &key, MbValue::none());
            last_key = Some(key);
            cur_opts = Some(opts);
        } else {
            return raise_named(
                "ParsingError",
                &format!(
                    "Source contains parsing errors: {:?}\n\t[line {:2}]: {:?}",
                    source, lineno, raw_line
                ),
            );
        }
    }
    MbValue::none()
}

// ── get / coercion ─────────────────────────────────────────────────────────

/// Core resolution shared by parser.get and proxy.get. Returns Ok(value) or
/// raises (returning None) when a missing section/option without fallback.
fn resolve_get(
    parser: MbValue,
    sec: &str,
    key: &str,
    raw: bool,
    fallback: Option<MbValue>,
) -> MbValue {
    let folded = optionxform(key);
    if sec != default_section_name(parser) && !has_section(parser, sec) {
        if let Some(fb) = fallback {
            return fb;
        }
        return raise_named("NoSectionError", &format!("No section: {}", pyrepr(&sec)));
    }
    match lookup_raw(parser, sec, &folded) {
        Some(Some(rawval)) => {
            if raw {
                return new_str(&rawval);
            }
            match do_interpolate(parser, sec, &rawval) {
                Ok(s) => new_str(&s),
                Err((exc, msg)) => raise_named(exc, &msg),
            }
        }
        Some(None) => MbValue::none(),
        None => {
            if let Some(fb) = fallback {
                return fb;
            }
            raise_named(
                "NoOptionError",
                &format!("No option {} in section: {}", pyrepr(&folded), pyrepr(sec)),
            )
        }
    }
}

fn coerce_int(s: &str) -> MbValue {
    match s.trim().parse::<i64>() {
        Ok(i) => MbValue::from_int(i),
        Err(_) => raise_value_error(&format!(
            "invalid literal for int() with base 10: {}",
            pyrepr(s)
        )),
    }
}
fn coerce_float(s: &str) -> MbValue {
    match s.trim().parse::<f64>() {
        Ok(f) => MbValue::from_float(f),
        Err(_) => raise_value_error(&format!("could not convert string to float: {}", pyrepr(s))),
    }
}
fn coerce_bool(s: &str) -> MbValue {
    match s.trim().to_lowercase().as_str() {
        "1" | "yes" | "true" | "on" => MbValue::from_bool(true),
        "0" | "no" | "false" | "off" => MbValue::from_bool(false),
        _ => raise_value_error(&format!("Not a boolean: {}", s)),
    }
}

// ── Constructor ────────────────────────────────────────────────────────────

fn interp_kind_from_value(v: MbValue) -> Option<String> {
    // A configparser.XInterpolation() instance reaches us as a marker string
    // (registered class name) or an instance whose class name encodes it.
    if v.is_none() {
        return Some("none".to_string());
    }
    if let Some(s) = extract_str(v) {
        return Some(
            match s.as_str() {
                "ExtendedInterpolation" => "extended",
                "BasicInterpolation" => "basic",
                "LegacyInterpolation" => "basic",
                "Interpolation" => "basic",
                "_UNSET" => "basic",
                _ => "basic",
            }
            .to_string(),
        );
    }
    if let Some(cn) = instance_class(v) {
        return Some(
            match cn.as_str() {
                "ExtendedInterpolation" => "extended",
                "LegacyInterpolation" => "basic",
                _ => "basic",
            }
            .to_string(),
        );
    }
    Some("basic".to_string())
}

fn make_parser(class_name: &str, kwargs: MbValue, raw_default: bool) -> MbValue {
    let p = MbValue::from_ptr(MbObject::new_instance(PARSER_CLASS.to_string()));
    set_field(p, "_class", new_str(class_name));
    set_field(p, "_sections", new_dict());
    set_field(p, "_defaults", new_dict());
    set_field(p, "_converters", new_dict());

    // delimiters / comment prefixes
    if let Some(v) = kw_get(kwargs, "delimiters") {
        set_field(p, "_delimiters", v);
    }
    if let Some(v) = kw_get(kwargs, "comment_prefixes") {
        set_field(p, "_comment_prefixes", v);
    }
    if let Some(v) = kw_get(kwargs, "inline_comment_prefixes") {
        set_field(p, "_inline_comment_prefixes", v);
    }

    // allow_no_value
    let anv = kw_get(kwargs, "allow_no_value")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    set_field(p, "_allow_no_value", MbValue::from_bool(anv));

    // strict (default True for ConfigParser/RawConfigParser in 3.12)
    let strict = kw_get(kwargs, "strict")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    set_field(p, "_strict", MbValue::from_bool(strict));

    // default_section
    let dsec = kw_get(kwargs, "default_section")
        .and_then(extract_str)
        .unwrap_or_else(|| "DEFAULT".to_string());
    set_field(p, "_default_section", new_str(&dsec));

    // interpolation
    let interp = if raw_default {
        "none".to_string()
    } else {
        match kw_get(kwargs, "interpolation") {
            Some(v) => interp_kind_from_value(v).unwrap_or_else(|| "basic".to_string()),
            None => "basic".to_string(),
        }
    };
    set_field(p, "_interp", new_str(&interp));

    // defaults mapping
    if let Some(dv) = kw_get(kwargs, "defaults") {
        if !dv.is_none() {
            let defaults = defaults_dict(p);
            for (k, v) in dict_str_pairs(dv) {
                let val = v.unwrap_or_default();
                dict_set_str(defaults, &optionxform(&k), new_str(&val));
            }
        }
    }

    p
}

// ── Registered native dispatchers ──────────────────────────────────────────

fn ctor_dispatch(
    args_ptr: *const MbValue,
    nargs: usize,
    class_name: &str,
    raw_default: bool,
) -> MbValue {
    let items = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let kwargs = if let Some(last) = items.last() {
        if is_dict(*last) {
            *last
        } else {
            MbValue::none()
        }
    } else {
        MbValue::none()
    };
    make_parser(class_name, kwargs, raw_default)
}

unsafe extern "C" fn dispatch_ConfigParser(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    ctor_dispatch(args_ptr, nargs, "ConfigParser", false)
}
unsafe extern "C" fn dispatch_RawConfigParser(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    ctor_dispatch(args_ptr, nargs, "RawConfigParser", true)
}
unsafe extern "C" fn dispatch_SafeConfigParser(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    ctor_dispatch(args_ptr, nargs, "SafeConfigParser", false)
}
unsafe extern "C" fn dispatch_interp_factory(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    // Marker factories. The returned class-name string is what the constructor
    // interprets via interp_kind_from_value.
    MbValue::none()
}
unsafe extern "C" fn dispatch_BasicInterpolation(_a: *const MbValue, _n: usize) -> MbValue {
    new_str("BasicInterpolation")
}
unsafe extern "C" fn dispatch_ExtendedInterpolation(_a: *const MbValue, _n: usize) -> MbValue {
    new_str("ExtendedInterpolation")
}
unsafe extern "C" fn dispatch_LegacyInterpolation(_a: *const MbValue, _n: usize) -> MbValue {
    new_str("LegacyInterpolation")
}
/// `configparser.MutableMapping` is re-exported from `collections.abc` in
/// CPython (an `ABCMeta`, hence callable). We expose a callable stub so the
/// surface probes (`hasattr` / `callable`) match; it is not constructed or
/// isinstance-checked by the configparser implementation.
unsafe extern "C" fn dispatch_MutableMapping(_a: *const MbValue, _n: usize) -> MbValue {
    new_str("MutableMapping")
}

// ── Parser methods (self, args_list) ───────────────────────────────────────

unsafe extern "C" fn m_read_string(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let text = match pos.first().and_then(|v| extract_str(*v)) {
        Some(s) => s,
        None => return raise_type_error("read_string() argument must be str"),
    };
    let source = pos
        .get(1)
        .and_then(|v| extract_str(*v))
        .unwrap_or_else(|| "<string>".to_string());
    parse_string(self_v, &text, &source)
}

/// Convert a "file-like or iterable of lines" argument into one text blob.
fn lines_from_arg(arg: MbValue) -> Option<String> {
    // str
    if let Some(s) = extract_str(arg) {
        return Some(s);
    }
    if let Some(ptr) = arg.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => {
                    return Some(join_line_items(&lock.read().unwrap()));
                }
                ObjData::Tuple(items) => {
                    return Some(join_line_items(items));
                }
                ObjData::Bytes(b) => {
                    return Some(String::from_utf8_lossy(b).into_owned());
                }
                _ => {}
            }
        }
    }
    // Object with .read(): call it.
    let rd = super::super::class::mb_call_method(arg, new_str("read"), new_list(vec![]));
    if super::super::exception::mb_has_exception()
        .as_bool()
        .unwrap_or(false)
    {
        super::super::exception::mb_clear_exception();
        return iter_to_text(arg);
    }
    if let Some(s) = extract_str(rd) {
        return Some(s);
    }
    iter_to_text(arg)
}

fn join_line_items(items: &[MbValue]) -> String {
    let mut out = String::new();
    for item in items {
        if let Some(s) = extract_str(*item) {
            out.push_str(&s);
        } else if let Some(ptr) = item.as_ptr() {
            unsafe {
                if let ObjData::Bytes(ref b) = (*ptr).data {
                    out.push_str(&String::from_utf8_lossy(b));
                }
            }
        }
    }
    out
}

fn iter_to_text(arg: MbValue) -> Option<String> {
    let it = super::super::iter::mb_iter(arg);
    if it.is_none() {
        return None;
    }
    let mut out = String::new();
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 1_000_000 {
            break;
        }
        let item = super::super::iter::mb_next_or_stop(it);
        if item.is_stop_iter_sentinel() {
            break;
        }
        if let Some(s) = extract_str(item) {
            out.push_str(&s);
        } else {
            break;
        }
    }
    super::super::exception::mb_clear_exception();
    Some(out)
}

unsafe extern "C" fn m_read_file(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let arg = match pos.first() {
        Some(v) => *v,
        None => return raise_type_error("read_file() missing argument"),
    };
    let source = pos
        .get(1)
        .and_then(|v| extract_str(*v))
        .unwrap_or_else(|| "<???>".to_string());
    match lines_from_arg(arg) {
        Some(text) => parse_string(self_v, &text, &source),
        None => raise_type_error("read_file() argument must be iterable of lines"),
    }
}

unsafe extern "C" fn m_read(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let arg = match pos.first() {
        Some(v) => *v,
        None => return new_list(vec![]),
    };
    // read(filenames) — accept a single path or a list of paths.
    let paths: Vec<String> = if let Some(s) = extract_str(arg) {
        vec![s]
    } else {
        seq_items(arg)
            .iter()
            .filter_map(|v| extract_str(*v))
            .collect()
    };
    let mut read_ok: Vec<MbValue> = Vec::new();
    for p in paths {
        match std::fs::read_to_string(&p) {
            Ok(text) => {
                let r = parse_string(self_v, &text, &p);
                if !super::super::exception::mb_has_exception()
                    .as_bool()
                    .unwrap_or(false)
                {
                    read_ok.push(new_str(&p));
                }
                let _ = r;
            }
            Err(_) => {}
        }
    }
    new_list(read_ok)
}

unsafe extern "C" fn m_read_dict(self_v: MbValue, args: MbValue) -> MbValue {
    // The single positional arg is itself a mapping; do NOT treat it as kwargs.
    let pos = seq_items(args);
    let dict = match pos.first() {
        Some(v) => *v,
        None => return MbValue::none(),
    };
    let secs = sections_dict(self_v);
    let dflt = default_section_name(self_v);
    // Iterate the outer mapping (section -> options-mapping).
    if let Some(ptr) = dict.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let pairs: Vec<(String, MbValue)> = lock
                    .read()
                    .unwrap()
                    .iter()
                    .filter_map(|(k, v)| k.as_str().map(|s| (s.to_string(), *v)))
                    .collect();
                for (sec, opts_map_val) in pairs {
                    let opts = if sec == dflt {
                        defaults_dict(self_v)
                    } else {
                        if !dict_contains_str(secs, &sec) {
                            dict_set_str(secs, &sec, new_dict());
                        }
                        dict_get_str(secs, &sec).unwrap_or_else(MbValue::none)
                    };
                    for (k, v) in dict_str_pairs(opts_map_val) {
                        let folded = optionxform(&k);
                        let valstr = v.unwrap_or_default();
                        dict_set_str(opts, &folded, new_str(&valstr));
                    }
                }
            }
        }
    }
    MbValue::none()
}

unsafe extern "C" fn m_sections(self_v: MbValue, _args: MbValue) -> MbValue {
    let names = dict_str_keys(sections_dict(self_v));
    new_list(names.iter().map(|s| new_str(s)).collect())
}

unsafe extern "C" fn m_add_section(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let name = match pos.first().and_then(|v| extract_str(*v)) {
        Some(s) => s,
        None => return raise_type_error("add_section() argument must be str"),
    };
    if name == default_section_name(self_v) {
        return raise_value_error(&format!("Invalid section name: {}", pyrepr(&name)));
    }
    let secs = sections_dict(self_v);
    if dict_contains_str(secs, &name) {
        return raise_named(
            "DuplicateSectionError",
            &format!("Section {} already exists", pyrepr(&name)),
        );
    }
    dict_set_str(secs, &name, new_dict());
    MbValue::none()
}

unsafe extern "C" fn m_has_section(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let name = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    MbValue::from_bool(has_section(self_v, &name))
}

unsafe extern "C" fn m_options(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    if sec != default_section_name(self_v) && !has_section(self_v, &sec) {
        return raise_named("NoSectionError", &format!("No section: {}", pyrepr(&sec)));
    }
    let keys: Vec<MbValue> = merged_options(self_v, &sec)
        .into_iter()
        .map(|(k, _)| new_str(&k))
        .collect();
    new_list(keys)
}

unsafe extern "C" fn m_has_option(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let key = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let dflt = default_section_name(self_v);
    if sec != dflt && !has_section(self_v, &sec) {
        return MbValue::from_bool(false);
    }
    let folded = optionxform(&key);
    MbValue::from_bool(lookup_raw(self_v, &sec, &folded).is_some())
}

unsafe extern "C" fn m_get(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let key = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let raw = kw_get(kw, "raw").and_then(|v| v.as_bool()).unwrap_or(false);
    let fallback = kw_get(kw, "fallback").or_else(|| pos.get(2).copied());
    resolve_get(self_v, &sec, &key, raw, fallback)
}

fn typed_get(self_v: MbValue, args: MbValue, conv: fn(&str) -> MbValue) -> MbValue {
    let (pos, kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let key = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let fallback = kw_get(kw, "fallback").or_else(|| pos.get(2).copied());
    let raw = kw_get(kw, "raw").and_then(|v| v.as_bool()).unwrap_or(false);
    // Resolve without fallback first; if it raised NoOption/NoSection and a
    // fallback exists, return the fallback verbatim (not coerced).
    let val = resolve_get(self_v, &sec, &key, raw, None);
    if super::super::exception::mb_has_exception()
        .as_bool()
        .unwrap_or(false)
    {
        if let Some(fb) = fallback {
            super::super::exception::mb_clear_exception();
            return fb;
        }
        return MbValue::none();
    }
    let s = extract_str(val).unwrap_or_default();
    conv(&s)
}

unsafe extern "C" fn m_getint(self_v: MbValue, args: MbValue) -> MbValue {
    typed_get(self_v, args, coerce_int)
}
unsafe extern "C" fn m_getfloat(self_v: MbValue, args: MbValue) -> MbValue {
    typed_get(self_v, args, coerce_float)
}
unsafe extern "C" fn m_getboolean(self_v: MbValue, args: MbValue) -> MbValue {
    typed_get(self_v, args, coerce_bool)
}

unsafe extern "C" fn m_items(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    match pos.first().and_then(|v| extract_str(*v)) {
        Some(sec) => {
            // items(section) -> list of (key, interpolated value).
            if sec != default_section_name(self_v) && !has_section(self_v, &sec) {
                return raise_named("NoSectionError", &format!("No section: {}", pyrepr(&sec)));
            }
            let mut out = Vec::new();
            for (k, v) in merged_options(self_v, &sec) {
                let value = match v {
                    Some(rawval) => match do_interpolate(self_v, &sec, &rawval) {
                        Ok(s) => new_str(&s),
                        Err((exc, msg)) => return raise_named(exc, &msg),
                    },
                    None => MbValue::none(),
                };
                out.push(MbValue::from_ptr(MbObject::new_tuple(vec![
                    new_str(&k),
                    value,
                ])));
            }
            new_list(out)
        }
        None => {
            // items() -> (section_name, proxy) for every section incl. default.
            let mut out = Vec::new();
            let dflt = default_section_name(self_v);
            out.push(MbValue::from_ptr(MbObject::new_tuple(vec![
                new_str(&dflt),
                make_proxy(self_v, &dflt),
            ])));
            for sec in dict_str_keys(sections_dict(self_v)) {
                out.push(MbValue::from_ptr(MbObject::new_tuple(vec![
                    new_str(&sec),
                    make_proxy(self_v, &sec),
                ])));
            }
            new_list(out)
        }
    }
}

unsafe extern "C" fn m_set(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let key = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let value = pos.get(2).copied().unwrap_or_else(MbValue::none);
    let dflt = default_section_name(self_v);
    let opts = match section_options(self_v, &sec) {
        Some(o) => o,
        None => return raise_named("NoSectionError", &format!("No section: {}", pyrepr(&sec))),
    };
    let _ = dflt;
    let folded = optionxform(&key);
    if value.is_none() {
        if allow_no_value(self_v) {
            dict_set_str(opts, &folded, MbValue::none());
            return MbValue::none();
        }
        return raise_type_error("option values must be strings");
    }
    let v = match extract_str(value) {
        Some(s) => s,
        None => return raise_type_error("option values must be strings"),
    };
    dict_set_str(opts, &folded, new_str(&v));
    MbValue::none()
}

unsafe extern "C" fn m_remove_option(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let key = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let opts = match section_options(self_v, &sec) {
        Some(o) => o,
        None => return raise_named("NoSectionError", &format!("No section: {}", pyrepr(&sec))),
    };
    let folded = optionxform(&key);
    MbValue::from_bool(dict_remove_str(opts, &folded))
}

unsafe extern "C" fn m_remove_section(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    MbValue::from_bool(dict_remove_str(sections_dict(self_v), &sec))
}

unsafe extern "C" fn m_defaults(self_v: MbValue, _args: MbValue) -> MbValue {
    // Return the defaults mapping (a live dict).
    defaults_dict(self_v)
}

unsafe extern "C" fn m_clear(self_v: MbValue, _args: MbValue) -> MbValue {
    set_field(self_v, "_sections", new_dict());
    set_field(self_v, "_defaults", new_dict());
    MbValue::none()
}

unsafe extern "C" fn m_write(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, kw) = split_args(args);
    let fp = match pos.first() {
        Some(v) => *v,
        None => return raise_type_error("write() missing file argument"),
    };
    let space_around = kw_get(kw, "space_around_delimiters")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let delim = delimiters(self_v)
        .first()
        .cloned()
        .unwrap_or_else(|| "=".to_string());
    let sep = if space_around {
        format!(" {} ", delim)
    } else {
        delim.clone()
    };

    let mut out = String::new();
    let dflt = default_section_name(self_v);
    // DEFAULT section first (only if non-empty).
    let dflt_pairs = dict_str_pairs(defaults_dict(self_v));
    if !dflt_pairs.is_empty() {
        out.push_str(&format!("[{}]\n", dflt));
        for (k, v) in &dflt_pairs {
            write_kv(&mut out, k, v, &sep);
        }
        out.push('\n');
    }
    for sec in dict_str_keys(sections_dict(self_v)) {
        out.push_str(&format!("[{}]\n", sec));
        if let Some(d) = dict_get_str(sections_dict(self_v), &sec) {
            for (k, v) in dict_str_pairs(d) {
                write_kv(&mut out, &k, &v, &sep);
            }
        }
        out.push('\n');
    }

    // Write to the file object via .write(str).
    super::super::class::mb_call_method(fp, new_str("write"), new_list(vec![new_str(&out)]));
    MbValue::none()
}

fn write_kv(out: &mut String, k: &str, v: &Option<String>, sep: &str) {
    match v {
        Some(val) => {
            let val = val.replace('\n', "\n\t");
            out.push_str(&format!("{}{}{}\n", k, sep, val));
        }
        None => out.push_str(&format!("{}\n", k)),
    }
}

// Mapping protocol on the parser.

unsafe extern "C" fn m_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    if sec != default_section_name(self_v) && !has_section(self_v, &sec) {
        return raise_named("KeyError", &format!("{}", pyrepr(&sec)));
    }
    make_proxy(self_v, &sec)
}

unsafe extern "C" fn m_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    // value is a mapping; iterate raw items so the trailing dict isn't
    // mistaken for kwargs.
    let pos = seq_items(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let value = pos.get(1).copied().unwrap_or_else(MbValue::none);
    let dflt = default_section_name(self_v);
    let opts = if sec == dflt {
        // Replace defaults contents.
        set_field(self_v, "_defaults", new_dict());
        defaults_dict(self_v)
    } else {
        let secs = sections_dict(self_v);
        // Fresh section (cp[x] = {...} replaces).
        dict_set_str(secs, &sec, new_dict());
        dict_get_str(secs, &sec).unwrap_or_else(MbValue::none)
    };
    // value is a mapping.
    for (k, v) in dict_str_pairs(value) {
        let folded = optionxform(&k);
        let valstr = v.unwrap_or_default();
        dict_set_str(opts, &folded, new_str(&valstr));
    }
    MbValue::none()
}

unsafe extern "C" fn m_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let sec = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    MbValue::from_bool(sec == default_section_name(self_v) || has_section(self_v, &sec))
}

unsafe extern "C" fn m_iter(self_v: MbValue, _args: MbValue) -> MbValue {
    // Iterating a parser yields DEFAULT first, then sections.
    let mut names = vec![new_str(&default_section_name(self_v))];
    for s in dict_str_keys(sections_dict(self_v)) {
        names.push(new_str(&s));
    }
    super::super::iter::mb_iter(new_list(names))
}

unsafe extern "C" fn m_len(self_v: MbValue, _args: MbValue) -> MbValue {
    // len(parser) counts sections plus DEFAULT.
    MbValue::from_int((dict_str_keys(sections_dict(self_v)).len() + 1) as i64)
}

unsafe extern "C" fn m_parser_getattr(self_v: MbValue, name_v: MbValue) -> MbValue {
    // Synthesized converter accessors getX, and `.converters`.
    let name = extract_str(name_v).unwrap_or_default();
    if name == "converters" {
        return get_field(self_v, "_converters").unwrap_or_else(new_dict_v);
    }
    if let Some(conv_name) = name.strip_prefix("get") {
        let convs = get_field(self_v, "_converters").unwrap_or_else(MbValue::none);
        if dict_contains_str(convs, conv_name) {
            // Return a bound accessor: a SectionProxy-style helper. We build a
            // small instance carrying parser + conv name and dispatch via a
            // registered __call__.
            let acc = MbValue::from_ptr(MbObject::new_instance("_ConverterAccessor".to_string()));
            set_field(acc, "_parser", self_v);
            set_field(acc, "_conv", new_str(conv_name));
            set_field(acc, "_proxy", MbValue::none());
            return acc;
        }
    }
    raise_named(
        "AttributeError",
        &format!("'{}' object has no attribute '{}'", "ConfigParser", name),
    )
}

fn new_dict_v() -> MbValue {
    new_dict()
}

// ── Converter accessor (__call__) ──────────────────────────────────────────

unsafe extern "C" fn acc_call(self_v: MbValue, args: MbValue) -> MbValue {
    let parser = get_field(self_v, "_parser").unwrap_or_else(MbValue::none);
    let conv = get_field(self_v, "_conv")
        .and_then(extract_str)
        .unwrap_or_default();
    let proxy = get_field(self_v, "_proxy").unwrap_or_else(MbValue::none);
    let convs = get_field(parser, "_converters").unwrap_or_else(MbValue::none);
    let func = match dict_get_str(convs, &conv) {
        Some(f) => f,
        None => {
            return raise_named(
                "AttributeError",
                &format!("getconverter '{}' missing", conv),
            )
        }
    };
    let (pos, kw) = split_args(args);
    // For a parser accessor: (section, option[, fallback]). For a proxy
    // accessor: (option[, fallback]) where section is fixed.
    let (sec, key, fallback) = if proxy.is_none() {
        let sec = pos
            .first()
            .and_then(|v| extract_str(*v))
            .unwrap_or_default();
        let key = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
        let fb = kw_get(kw, "fallback").or_else(|| pos.get(2).copied());
        (sec, key, fb)
    } else {
        let sec = get_field(proxy, "_name")
            .and_then(extract_str)
            .unwrap_or_default();
        let key = pos
            .first()
            .and_then(|v| extract_str(*v))
            .unwrap_or_default();
        let fb = kw_get(kw, "fallback").or_else(|| pos.get(1).copied());
        (sec, key, fb)
    };
    let val = resolve_get(parser, &sec, &key, false, None);
    if super::super::exception::mb_has_exception()
        .as_bool()
        .unwrap_or(false)
    {
        if let Some(fb) = fallback {
            super::super::exception::mb_clear_exception();
            return fb;
        }
        return MbValue::none();
    }
    // Call the converter function on the string value.
    super::super::builtins::mb_call_spread(func, new_list(vec![val]))
}

// ── SectionProxy ───────────────────────────────────────────────────────────

fn make_proxy(parser: MbValue, sec: &str) -> MbValue {
    let p = MbValue::from_ptr(MbObject::new_instance(PROXY_CLASS.to_string()));
    set_field(p, "_parser", parser);
    set_field(p, "_name", new_str(sec));
    p
}

fn proxy_parser(self_v: MbValue) -> MbValue {
    get_field(self_v, "_parser").unwrap_or_else(MbValue::none)
}
fn proxy_name(self_v: MbValue) -> String {
    get_field(self_v, "_name")
        .and_then(extract_str)
        .unwrap_or_default()
}

unsafe extern "C" fn p_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let key = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    let val = resolve_get(parser, &sec, &key, false, None);
    if super::super::exception::mb_has_exception()
        .as_bool()
        .unwrap_or(false)
    {
        // proxy[key] raises KeyError, not NoOptionError.
        super::super::exception::mb_clear_exception();
        return raise_named("KeyError", &format!("{}", pyrepr(&optionxform(&key))));
    }
    val
}

unsafe extern "C" fn p_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let key = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let value = pos.get(1).copied().unwrap_or_else(MbValue::none);
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    if let Some(opts) = section_options(parser, &sec) {
        let folded = optionxform(&key);
        if value.is_none() {
            dict_set_str(opts, &folded, MbValue::none());
        } else {
            let v = extract_str(value).unwrap_or_default();
            dict_set_str(opts, &folded, new_str(&v));
        }
    }
    MbValue::none()
}

unsafe extern "C" fn p_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, _kw) = split_args(args);
    let key = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    let folded = optionxform(&key);
    MbValue::from_bool(lookup_raw(parser, &sec, &folded).is_some())
}

unsafe extern "C" fn p_iter(self_v: MbValue, _args: MbValue) -> MbValue {
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    let names: Vec<MbValue> = merged_options(parser, &sec)
        .into_iter()
        .map(|(k, _)| new_str(&k))
        .collect();
    super::super::iter::mb_iter(new_list(names))
}

unsafe extern "C" fn p_len(self_v: MbValue, _args: MbValue) -> MbValue {
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    MbValue::from_int(merged_options(parser, &sec).len() as i64)
}

unsafe extern "C" fn p_get(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, kw) = split_args(args);
    let key = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    let raw = kw_get(kw, "raw").and_then(|v| v.as_bool()).unwrap_or(false);
    // proxy.get(option, fallback=None) — fallback defaults to None, not raise.
    let fallback = kw_get(kw, "fallback")
        .or_else(|| pos.get(1).copied())
        .or(Some(MbValue::none()));
    resolve_get(parser, &sec, &key, raw, fallback)
}

fn proxy_typed_get(self_v: MbValue, args: MbValue, conv: fn(&str) -> MbValue) -> MbValue {
    let (pos, kw) = split_args(args);
    let key = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    let fallback = kw_get(kw, "fallback").or_else(|| pos.get(1).copied());
    let val = resolve_get(parser, &sec, &key, false, None);
    if super::super::exception::mb_has_exception()
        .as_bool()
        .unwrap_or(false)
    {
        super::super::exception::mb_clear_exception();
        return fallback.unwrap_or_else(MbValue::none);
    }
    conv(&extract_str(val).unwrap_or_default())
}

unsafe extern "C" fn p_getint(self_v: MbValue, args: MbValue) -> MbValue {
    proxy_typed_get(self_v, args, coerce_int)
}
unsafe extern "C" fn p_getfloat(self_v: MbValue, args: MbValue) -> MbValue {
    proxy_typed_get(self_v, args, coerce_float)
}
unsafe extern "C" fn p_getboolean(self_v: MbValue, args: MbValue) -> MbValue {
    proxy_typed_get(self_v, args, coerce_bool)
}

unsafe extern "C" fn p_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    new_list(
        merged_options(parser, &sec)
            .into_iter()
            .map(|(k, _)| new_str(&k))
            .collect(),
    )
}

unsafe extern "C" fn p_items(self_v: MbValue, _args: MbValue) -> MbValue {
    let parser = proxy_parser(self_v);
    let sec = proxy_name(self_v);
    let mut out = Vec::new();
    for (k, v) in merged_options(parser, &sec) {
        let value = match v {
            Some(rawval) => match do_interpolate(parser, &sec, &rawval) {
                Ok(s) => new_str(&s),
                Err((exc, msg)) => return raise_named(exc, &msg),
            },
            None => MbValue::none(),
        };
        out.push(MbValue::from_ptr(MbObject::new_tuple(vec![
            new_str(&k),
            value,
        ])));
    }
    new_list(out)
}

unsafe extern "C" fn p_repr(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str(&format!("<Section: {}>", proxy_name(self_v)))
}

unsafe extern "C" fn p_name(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str(&proxy_name(self_v))
}

unsafe extern "C" fn p_getattr(self_v: MbValue, name_v: MbValue) -> MbValue {
    let name = extract_str(name_v).unwrap_or_default();
    if name == "name" {
        return new_str(&proxy_name(self_v));
    }
    if name == "parser" {
        return proxy_parser(self_v);
    }
    if let Some(conv_name) = name.strip_prefix("get") {
        let parser = proxy_parser(self_v);
        let convs = get_field(parser, "_converters").unwrap_or_else(MbValue::none);
        if dict_contains_str(convs, conv_name) {
            let acc = MbValue::from_ptr(MbObject::new_instance("_ConverterAccessor".to_string()));
            set_field(acc, "_parser", parser);
            set_field(acc, "_conv", new_str(conv_name));
            set_field(acc, "_proxy", self_v);
            return acc;
        }
    }
    raise_named(
        "AttributeError",
        &format!("'{}' object has no attribute '{}'", "SectionProxy", name),
    )
}

// ── Structured exception classes ───────────────────────────────────────────
//
// Each exception is a real `Exception` subclass so `except configparser.X`
// matches, with named attributes (section/option/source/lineno/...), an `args`
// tuple matching CPython 3.12, a `message` string, and a `__str__`.

fn exc_set(inst: MbValue, key: &str, val: MbValue) {
    set_field(inst, key, val);
}

/// Python `%r` for a plain string: single-quoted unless the string contains a
/// single quote and no double quote (then double-quoted), matching CPython.
fn pyrepr(s: &str) -> String {
    let has_single = s.contains('\'');
    let has_double = s.contains('"');
    if has_single && !has_double {
        format!("\"{}\"", s)
    } else {
        format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

fn args_tuple(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(items))
}

/// Finalize an exception instance: set `args`, `message`, and `__type__`.
fn finalize_exc(inst: MbValue, class_name: &str, args: Vec<MbValue>, message: &str) {
    exc_set(inst, "args", args_tuple(args));
    exc_set(inst, "message", new_str(message));
    exc_set(inst, "__type__", new_str(class_name));
}

unsafe extern "C" fn init_Error(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let msg = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    finalize_exc(self_v, "Error", pos.clone(), &msg);
    MbValue::none()
}

unsafe extern "C" fn init_NoSectionError(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let section = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    exc_set(self_v, "section", new_str(&section));
    let msg = format!("No section: {}", pyrepr(&section));
    finalize_exc(self_v, "NoSectionError", vec![new_str(&section)], &msg);
    MbValue::none()
}

unsafe extern "C" fn init_NoOptionError(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let option = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let section = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    exc_set(self_v, "option", new_str(&option));
    exc_set(self_v, "section", new_str(&section));
    let msg = format!(
        "No option {} in section: {}",
        pyrepr(&option),
        pyrepr(&section)
    );
    finalize_exc(
        self_v,
        "NoOptionError",
        vec![new_str(&option), new_str(&section)],
        &msg,
    );
    MbValue::none()
}

unsafe extern "C" fn init_DuplicateSectionError(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let section = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let source = pos.get(1).copied().unwrap_or_else(MbValue::none);
    let lineno = pos.get(2).copied().unwrap_or_else(MbValue::none);
    exc_set(self_v, "section", new_str(&section));
    exc_set(self_v, "source", source);
    exc_set(self_v, "lineno", lineno);
    let msg = if !source.is_none() {
        let src = extract_str(source).unwrap_or_default();
        let ln = lineno.as_int().unwrap_or(0);
        format!(
            "While reading from {} [line {:2}]: section {} already exists",
            pyrepr(&src),
            ln,
            pyrepr(&section)
        )
    } else {
        format!("Section {} already exists", pyrepr(&section))
    };
    finalize_exc(
        self_v,
        "DuplicateSectionError",
        vec![new_str(&section), source, lineno],
        &msg,
    );
    MbValue::none()
}

unsafe extern "C" fn init_DuplicateOptionError(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let section = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let option = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let source = pos.get(2).copied().unwrap_or_else(MbValue::none);
    let lineno = pos.get(3).copied().unwrap_or_else(MbValue::none);
    exc_set(self_v, "section", new_str(&section));
    exc_set(self_v, "option", new_str(&option));
    exc_set(self_v, "source", source);
    exc_set(self_v, "lineno", lineno);
    let msg = if !source.is_none() {
        let src = extract_str(source).unwrap_or_default();
        let ln = lineno.as_int().unwrap_or(0);
        format!(
            "While reading from {} [line {:2}]: option {} in section {} already exists",
            pyrepr(&src),
            ln,
            pyrepr(&option),
            pyrepr(&section)
        )
    } else {
        format!(
            "Option {} in section {} already exists",
            pyrepr(&option),
            pyrepr(&section)
        )
    };
    finalize_exc(
        self_v,
        "DuplicateOptionError",
        vec![new_str(&section), new_str(&option), source, lineno],
        &msg,
    );
    MbValue::none()
}

unsafe extern "C" fn init_InterpolationError(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let option = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let section = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let msg = pos.get(2).and_then(|v| extract_str(*v)).unwrap_or_default();
    exc_set(self_v, "option", new_str(&option));
    exc_set(self_v, "section", new_str(&section));
    finalize_exc(
        self_v,
        "InterpolationError",
        vec![new_str(&option), new_str(&section), new_str(&msg)],
        &msg,
    );
    MbValue::none()
}

unsafe extern "C" fn init_InterpolationMissingOptionError(
    self_v: MbValue,
    args: MbValue,
) -> MbValue {
    let pos = seq_items(args);
    let option = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let section = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let rawval = pos.get(2).and_then(|v| extract_str(*v)).unwrap_or_default();
    let reference = pos.get(3).and_then(|v| extract_str(*v)).unwrap_or_default();
    exc_set(self_v, "option", new_str(&option));
    exc_set(self_v, "section", new_str(&section));
    exc_set(self_v, "reference", new_str(&reference));
    let msg = format!("Bad value substitution: option {} in section {} contains an interpolation key {} which is not a valid option name. Raw value: {}",
        pyrepr(&option), pyrepr(&section), pyrepr(&reference), pyrepr(&rawval));
    finalize_exc(
        self_v,
        "InterpolationMissingOptionError",
        vec![
            new_str(&option),
            new_str(&section),
            new_str(&rawval),
            new_str(&reference),
        ],
        &msg,
    );
    MbValue::none()
}

unsafe extern "C" fn init_InterpolationDepthError(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let option = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let section = pos.get(1).and_then(|v| extract_str(*v)).unwrap_or_default();
    let rawval = pos.get(2).and_then(|v| extract_str(*v)).unwrap_or_default();
    exc_set(self_v, "option", new_str(&option));
    exc_set(self_v, "section", new_str(&section));
    let msg = format!("Recursion limit exceeded in value substitution: option {} in section {} contains an interpolation key which cannot be substituted in {} steps. Raw value: {}",
        pyrepr(&option), pyrepr(&section), 10, pyrepr(&rawval));
    finalize_exc(
        self_v,
        "InterpolationDepthError",
        vec![new_str(&option), new_str(&section), new_str(&rawval)],
        &msg,
    );
    MbValue::none()
}

unsafe extern "C" fn init_MissingSectionHeaderError(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let filename = pos
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let lineno = pos.get(1).copied().unwrap_or_else(MbValue::none);
    let line = pos.get(2).and_then(|v| extract_str(*v)).unwrap_or_default();
    exc_set(self_v, "source", new_str(&filename));
    exc_set(self_v, "lineno", lineno);
    exc_set(self_v, "line", new_str(&line));
    let ln = lineno.as_int().unwrap_or(0);
    let msg = format!(
        "File contains no section headers.\nfile: {}, line: {}\n{}",
        pyrepr(&filename),
        ln,
        pyrepr(&line)
    );
    finalize_exc(
        self_v,
        "MissingSectionHeaderError",
        vec![new_str(&filename), lineno, new_str(&line)],
        &msg,
    );
    MbValue::none()
}

unsafe extern "C" fn init_ParsingError(self_v: MbValue, args: MbValue) -> MbValue {
    let (pos, kw) = split_args(args);
    let source = kw_get(kw, "source").or_else(|| pos.first().copied());
    let filename = kw_get(kw, "filename");
    let src = source.or(filename);
    let src = match src {
        Some(s) if !s.is_none() => s,
        _ => return raise_type_error("Required argument `source' not given."),
    };
    let src_str = extract_str(src).unwrap_or_default();
    exc_set(self_v, "source", src);
    exc_set(self_v, "errors", new_list(vec![]));
    let msg = format!("Source contains parsing errors: {}", pyrepr(&src_str));
    finalize_exc(self_v, "ParsingError", vec![src], &msg);
    MbValue::none()
}

unsafe extern "C" fn exc_str(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str(
        &get_field(self_v, "message")
            .and_then(extract_str)
            .unwrap_or_default(),
    )
}

unsafe extern "C" fn pe_append(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = seq_items(args);
    let lineno = pos.first().copied().unwrap_or_else(MbValue::none);
    let line = pos.get(1).copied().unwrap_or_else(MbValue::none);
    if let Some(errors) = get_field(self_v, "errors") {
        let pair = MbValue::from_ptr(MbObject::new_tuple(vec![lineno, line]));
        super::super::class::mb_call_method(errors, new_str("append"), new_list(vec![pair]));
    }
    MbValue::none()
}

fn register_exception_classes() {
    let exc_specs: Vec<(&str, usize, &[&str])> = vec![
        ("Error", init_Error as usize, &["Exception"]),
        ("NoSectionError", init_NoSectionError as usize, &["Error"]),
        ("NoOptionError", init_NoOptionError as usize, &["Error"]),
        (
            "DuplicateSectionError",
            init_DuplicateSectionError as usize,
            &["Error"],
        ),
        (
            "DuplicateOptionError",
            init_DuplicateOptionError as usize,
            &["Error"],
        ),
        (
            "InterpolationError",
            init_InterpolationError as usize,
            &["Error"],
        ),
        (
            "InterpolationMissingOptionError",
            init_InterpolationMissingOptionError as usize,
            &["InterpolationError"],
        ),
        (
            "InterpolationDepthError",
            init_InterpolationDepthError as usize,
            &["InterpolationError"],
        ),
        (
            "MissingSectionHeaderError",
            init_MissingSectionHeaderError as usize,
            &["ParsingError"],
        ),
    ];
    for (name, init_addr, bases) in exc_specs {
        let mut map: HashMap<String, MbValue> = HashMap::new();
        map.insert("__init__".to_string(), MbValue::from_func(init_addr));
        map.insert("__str__".to_string(), MbValue::from_func(exc_str as usize));
        super::super::module::register_variadic_func(init_addr as u64);
        super::super::module::register_variadic_func(exc_str as usize as u64);
        let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
        super::super::class::mb_class_register(name, base_vec, map);
    }
    // ParsingError gets an extra `append` method.
    {
        let mut map: HashMap<String, MbValue> = HashMap::new();
        map.insert(
            "__init__".to_string(),
            MbValue::from_func(init_ParsingError as usize),
        );
        map.insert("__str__".to_string(), MbValue::from_func(exc_str as usize));
        map.insert("append".to_string(), MbValue::from_func(pe_append as usize));
        super::super::module::register_variadic_func(init_ParsingError as usize as u64);
        super::super::module::register_variadic_func(pe_append as usize as u64);
        super::super::class::mb_class_register("ParsingError", vec!["Error".to_string()], map);
    }
    // InterpolationSyntaxError is a plain InterpolationError subclass.
    {
        let mut map: HashMap<String, MbValue> = HashMap::new();
        map.insert("__str__".to_string(), MbValue::from_func(exc_str as usize));
        super::super::class::mb_class_register(
            "InterpolationSyntaxError",
            vec!["InterpolationError".to_string()],
            map,
        );
    }
}

// ── Registration ───────────────────────────────────────────────────────────

type MethodSpec = (&'static str, usize, bool);

fn register_method_class(class_name: &str, bases: &[&str], methods: &[MethodSpec]) {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (name, addr, variadic) in methods {
        map.insert(name.to_string(), MbValue::from_func(*addr));
        if *variadic {
            super::super::module::register_variadic_func(*addr as u64);
        }
    }
    let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
    super::super::class::mb_class_register(class_name, base_vec, map);
}

pub fn register() {
    let mut attrs = HashMap::new();

    // Constructors.
    let ctors: Vec<(&str, usize)> = vec![
        ("ConfigParser", dispatch_ConfigParser as usize),
        ("RawConfigParser", dispatch_RawConfigParser as usize),
        ("SafeConfigParser", dispatch_SafeConfigParser as usize),
        ("BasicInterpolation", dispatch_BasicInterpolation as usize),
        (
            "ExtendedInterpolation",
            dispatch_ExtendedInterpolation as usize,
        ),
        ("LegacyInterpolation", dispatch_LegacyInterpolation as usize),
        ("Interpolation", dispatch_interp_factory as usize),
        ("MutableMapping", dispatch_MutableMapping as usize),
    ];
    for (name, addr) in ctors {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Module constants.
    attrs.insert("DEFAULTSECT".into(), new_str("DEFAULT"));
    attrs.insert("MAX_INTERPOLATION_DEPTH".into(), MbValue::from_int(10));
    // Sentinels referenced by auto-ported fixtures.
    attrs.insert("_UNSET".into(), new_str("_UNSET"));
    attrs.insert("_default_dict".into(), new_str("dict"));

    // Exception markers (matched by type-name string).
    for err in [
        "Error",
        "NoSectionError",
        "DuplicateOptionError",
        "DuplicateSectionError",
        "NoOptionError",
        "InterpolationError",
        "InterpolationMissingOptionError",
        "InterpolationSyntaxError",
        "InterpolationDepthError",
        "MissingSectionHeaderError",
        "ParsingError",
    ] {
        attrs.insert(err.into(), new_str(err));
    }
    for marker in ["ConverterMapping", "SectionProxy"] {
        attrs.insert(marker.into(), new_str(marker));
    }

    super::register_module("configparser", attrs);

    // Bridge the parser constructor funcs -> their class name so accessing a
    // registered method on the class (`ConfigParser.read_dict`) resolves to a
    // callable unbound method via mb_getattr's func->native-class method bridge
    // (which looks the func addr up in NATIVE_TYPE_NAMES, then lookup_method in
    // the table mb_class_register populates below). Without this the methods are
    // registered but `callable(ConfigParser.read_dict)` is False.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(
            dispatch_ConfigParser as *const () as usize as u64,
            PARSER_CLASS.to_string(),
        );
        map.insert(
            dispatch_RawConfigParser as *const () as usize as u64,
            PARSER_CLASS.to_string(),
        );
        map.insert(
            dispatch_SafeConfigParser as *const () as usize as u64,
            PARSER_CLASS.to_string(),
        );
    });

    // Register parser + proxy classes.
    register_method_class(
        PARSER_CLASS,
        &[],
        &[
            ("read_string", m_read_string as usize, true),
            ("read_file", m_read_file as usize, true),
            ("read", m_read as usize, true),
            ("read_dict", m_read_dict as usize, true),
            ("sections", m_sections as usize, true),
            ("add_section", m_add_section as usize, true),
            ("has_section", m_has_section as usize, true),
            ("options", m_options as usize, true),
            ("has_option", m_has_option as usize, true),
            ("get", m_get as usize, true),
            ("getint", m_getint as usize, true),
            ("getfloat", m_getfloat as usize, true),
            ("getboolean", m_getboolean as usize, true),
            ("items", m_items as usize, true),
            ("set", m_set as usize, true),
            ("remove_option", m_remove_option as usize, true),
            ("remove_section", m_remove_section as usize, true),
            ("defaults", m_defaults as usize, true),
            ("clear", m_clear as usize, true),
            ("write", m_write as usize, true),
            ("keys", m_iter_keys as usize, true),
            ("__getitem__", m_getitem as usize, true),
            ("__setitem__", m_setitem as usize, true),
            ("__contains__", m_contains as usize, true),
            ("__iter__", m_iter as usize, true),
            ("__len__", m_len as usize, true),
            ("__getattr__", m_parser_getattr as usize, false),
        ],
    );

    register_method_class(
        PROXY_CLASS,
        &[],
        &[
            ("get", p_get as usize, true),
            ("getint", p_getint as usize, true),
            ("getfloat", p_getfloat as usize, true),
            ("getboolean", p_getboolean as usize, true),
            ("keys", p_keys as usize, true),
            ("items", p_items as usize, true),
            ("name", p_name as usize, true),
            ("__getitem__", p_getitem as usize, true),
            ("__setitem__", p_setitem as usize, true),
            ("__contains__", p_contains as usize, true),
            ("__iter__", p_iter as usize, true),
            ("__len__", p_len as usize, true),
            ("__repr__", p_repr as usize, true),
            ("__getattr__", p_getattr as usize, false),
        ],
    );

    register_method_class(
        "_ConverterAccessor",
        &[],
        &[("__call__", acc_call as usize, true)],
    );

    register_exception_classes();
}

unsafe extern "C" fn m_iter_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    let mut names = vec![new_str(&default_section_name(self_v))];
    for s in dict_str_keys(sections_dict(self_v)) {
        names.push(new_str(&s));
    }
    new_list(names)
}

// ── Back-compat free functions for in-crate Rust tests ─────────────────────
//
// The existing src/runtime/stdlib/configparser_mod.rs unit tests call these
// dict-shaped helpers directly. They are kept self-contained (independent of
// the instance-based path above) so the crate test build stays green.

fn legacy_extract_str(val: MbValue) -> Option<String> {
    extract_str(val)
}

pub fn mb_configparser_ConfigParser() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut m = lock.write().unwrap();
            m.insert("__class__".into(), new_str("ConfigParser"));
            m.insert("_data".into(), new_dict());
        }
    }
    MbValue::from_ptr(dict)
}

fn legacy_data_ptr(parser: MbValue) -> Option<MbValue> {
    parser.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get("_data").copied()
        } else {
            None
        }
    })
}

pub fn mb_configparser_read_string(parser: MbValue, text: MbValue) -> MbValue {
    let text_str = match legacy_extract_str(text) {
        Some(s) => s,
        None => return raise_type_error("read_string() argument must be str"),
    };
    let dp = match legacy_data_ptr(parser) {
        Some(p) => p,
        None => return MbValue::none(),
    };
    let mut cur_sec = String::new();
    let mut have_section = false;
    for (idx, raw) in text_str.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            cur_sec = line[1..line.len() - 1].trim().to_string();
            have_section = true;
            if !dict_contains_str(dp, &cur_sec) {
                dict_set_str(dp, &cur_sec, new_dict());
            }
        } else if !have_section {
            return raise_named(
                "MissingSectionHeaderError",
                &format!(
                    "File contains no section headers.\nfile: '<string>', line: {}\n{:?}",
                    idx + 1,
                    raw
                ),
            );
        } else {
            let pos = match (line.find('='), line.find(':')) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            if let Some(eq) = pos {
                let k = optionxform(line[..eq].trim());
                let v = line[eq + 1..].trim().to_string();
                if let Some(sv) = dict_get_str(dp, &cur_sec) {
                    dict_set_str(sv, &k, new_str(&v));
                }
            }
        }
    }
    MbValue::none()
}

fn legacy_section(parser: MbValue, sec: &str) -> Option<HashMap<String, String>> {
    let dp = legacy_data_ptr(parser)?;
    let sv = dict_get_str(dp, sec)?;
    let mut out = HashMap::new();
    for (k, v) in dict_str_pairs(sv) {
        if let Some(s) = v {
            out.insert(k, s);
        }
    }
    Some(out)
}

fn legacy_merged(parser: MbValue, sec: &str) -> HashMap<String, String> {
    let mut out = legacy_section(parser, "DEFAULT").unwrap_or_default();
    if sec != "DEFAULT" {
        if let Some(own) = legacy_section(parser, sec) {
            out.extend(own);
        }
    }
    out
}

pub fn mb_configparser_get(parser: MbValue, section: MbValue, key: MbValue) -> MbValue {
    let sec = legacy_extract_str(section).unwrap_or_default();
    let k = optionxform(&legacy_extract_str(key).unwrap_or_default());
    if sec != "DEFAULT" && legacy_section(parser, &sec).is_none() {
        return raise_named("NoSectionError", &format!("No section: {}", pyrepr(&sec)));
    }
    let opts = legacy_merged(parser, &sec);
    let raw = match opts.get(&k) {
        Some(v) => v.clone(),
        None => {
            return raise_named(
                "NoOptionError",
                &format!("No option {:?} in section: {:?}", k, sec),
            )
        }
    };
    match interpolate_basic(&raw, &opts, 1) {
        Ok(s) => new_str(&s),
        Err((exc, msg)) => raise_named(exc, &msg),
    }
}

pub fn mb_configparser_set(
    parser: MbValue,
    section: MbValue,
    key: MbValue,
    value: MbValue,
) -> MbValue {
    let sec = legacy_extract_str(section).unwrap_or_default();
    let k = optionxform(&legacy_extract_str(key).unwrap_or_default());
    if value.is_none() {
        return raise_type_error("option values must be strings");
    }
    let v = legacy_extract_str(value).unwrap_or_default();
    if let Some(dp) = legacy_data_ptr(parser) {
        if !dict_contains_str(dp, &sec) {
            dict_set_str(dp, &sec, new_dict());
        }
        if let Some(sv) = dict_get_str(dp, &sec) {
            dict_set_str(sv, &k, new_str(&v));
        }
    }
    MbValue::none()
}

pub fn mb_configparser_sections(parser: MbValue) -> MbValue {
    let mut names = Vec::new();
    if let Some(dp) = legacy_data_ptr(parser) {
        for k in dict_str_keys(dp) {
            if k == "DEFAULT" {
                continue;
            }
            names.push(new_str(&k));
        }
    }
    new_list(names)
}

pub fn mb_configparser_options(parser: MbValue, section: MbValue) -> MbValue {
    let sec = legacy_extract_str(section).unwrap_or_default();
    let keys: Vec<MbValue> = legacy_merged(parser, &sec)
        .keys()
        .map(|k| new_str(k))
        .collect();
    new_list(keys)
}

pub fn mb_configparser_ParsingError() -> MbValue {
    raise_type_error("__init__() missing 1 required positional argument: 'source'")
}

pub fn mb_configparser_interpolation_factory() -> MbValue {
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        new_str(val)
    }
    fn get_str(val: MbValue) -> String {
        extract_str(val).unwrap_or_default()
    }
    fn list_strs(val: MbValue) -> Vec<String> {
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read()
                        .unwrap()
                        .iter()
                        .filter_map(|v| extract_str(*v))
                        .collect()
                } else {
                    vec![]
                }
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_create_parser() {
        let p = mb_configparser_ConfigParser();
        assert!(p.as_ptr().is_some());
        let ptr = p.as_ptr().unwrap();
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let m = lock.read().unwrap();
                assert_eq!(
                    extract_str(*m.get("__class__").unwrap()),
                    Some("ConfigParser".to_string())
                );
            } else {
                panic!("expected dict");
            }
        }
    }

    #[test]
    fn test_read_string_basic() {
        let p = mb_configparser_ConfigParser();
        let ini = "[section1]\nkey1 = value1\nkey2 = value2\n";
        mb_configparser_read_string(p, s(ini));
        assert_eq!(
            get_str(mb_configparser_get(p, s("section1"), s("key1"))),
            "value1"
        );
        assert_eq!(
            get_str(mb_configparser_get(p, s("section1"), s("key2"))),
            "value2"
        );
    }

    #[test]
    fn test_read_string_multiple_sections() {
        let p = mb_configparser_ConfigParser();
        let ini = "[db]\nhost = localhost\nport = 5432\n\n[app]\ndebug = true\n";
        mb_configparser_read_string(p, s(ini));
        assert_eq!(
            get_str(mb_configparser_get(p, s("db"), s("host"))),
            "localhost"
        );
        assert_eq!(get_str(mb_configparser_get(p, s("db"), s("port"))), "5432");
        assert_eq!(
            get_str(mb_configparser_get(p, s("app"), s("debug"))),
            "true"
        );
    }

    #[test]
    fn test_read_string_comments_and_blank_lines() {
        let p = mb_configparser_ConfigParser();
        let ini = "# comment line\n; another comment\n\n[sec]\nk = v\n";
        mb_configparser_read_string(p, s(ini));
        assert_eq!(get_str(mb_configparser_get(p, s("sec"), s("k"))), "v");
    }

    fn pending_exc_type() -> Option<String> {
        let t = super::super::super::exception::current_exception_type();
        super::super::super::exception::mb_clear_exception();
        t
    }

    #[test]
    fn test_get_missing_section_raises_nosectionerror() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\nk = v\n"));
        let v = mb_configparser_get(p, s("nonexist"), s("k"));
        assert!(v.is_none());
        assert_eq!(pending_exc_type().as_deref(), Some("NoSectionError"));
    }

    #[test]
    fn test_get_missing_key_raises_nooptionerror() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\nk = v\n"));
        let v = mb_configparser_get(p, s("s"), s("missing"));
        assert!(v.is_none());
        assert_eq!(pending_exc_type().as_deref(), Some("NoOptionError"));
    }

    #[test]
    fn test_basic_interpolation() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\none = 1\ntwo = %(one)s and 2\n"));
        assert_eq!(get_str(mb_configparser_get(p, s("s"), s("two"))), "1 and 2");
    }

    #[test]
    fn test_interpolation_percent_literal() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\nv = 100%% done\n"));
        assert_eq!(get_str(mb_configparser_get(p, s("s"), s("v"))), "100% done");
    }

    #[test]
    fn test_interpolation_missing_referent_raises() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\nv = %(absent)s\n"));
        let _ = mb_configparser_get(p, s("s"), s("v"));
        assert_eq!(
            pending_exc_type().as_deref(),
            Some("InterpolationMissingOptionError")
        );
    }

    #[test]
    fn test_interpolation_cycle_raises_depth() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\na = %(b)s\nb = %(a)s\n"));
        let _ = mb_configparser_get(p, s("s"), s("a"));
        assert_eq!(
            pending_exc_type().as_deref(),
            Some("InterpolationDepthError")
        );
    }

    #[test]
    fn test_read_string_missing_section_header_raises() {
        let p = mb_configparser_ConfigParser();
        let _ = mb_configparser_read_string(p, s("key = value\n"));
        assert_eq!(
            pending_exc_type().as_deref(),
            Some("MissingSectionHeaderError")
        );
    }

    #[test]
    fn test_set_none_raises_typeerror() {
        let p = mb_configparser_ConfigParser();
        let _ = mb_configparser_set(p, s("s"), s("opt"), MbValue::none());
        assert_eq!(pending_exc_type().as_deref(), Some("TypeError"));
    }

    #[test]
    fn test_parsing_error_no_source_raises_typeerror() {
        let _ = mb_configparser_ParsingError();
        assert_eq!(pending_exc_type().as_deref(), Some("TypeError"));
    }

    #[test]
    fn test_set_new_section_and_key() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_set(p, s("new_sec"), s("key"), s("val"));
        assert_eq!(
            get_str(mb_configparser_get(p, s("new_sec"), s("key"))),
            "val"
        );
    }

    #[test]
    fn test_set_overwrite() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\nk = old\n"));
        mb_configparser_set(p, s("s"), s("k"), s("new"));
        assert_eq!(get_str(mb_configparser_get(p, s("s"), s("k"))), "new");
    }

    #[test]
    fn test_sections_empty() {
        let p = mb_configparser_ConfigParser();
        let secs = list_strs(mb_configparser_sections(p));
        assert!(secs.is_empty());
    }

    #[test]
    fn test_sections_after_read() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[alpha]\nx = 1\n[beta]\ny = 2\n"));
        let mut secs = list_strs(mb_configparser_sections(p));
        secs.sort();
        assert_eq!(secs, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_options() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\na = 1\nb = 2\nc = 3\n"));
        let mut opts = list_strs(mb_configparser_options(p, s("s")));
        opts.sort();
        assert_eq!(opts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_options_missing_section() {
        let p = mb_configparser_ConfigParser();
        let opts = list_strs(mb_configparser_options(p, s("nope")));
        assert!(opts.is_empty());
    }

    #[test]
    fn test_value_with_spaces_around_equals() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\n  key  =  val  \n"));
        assert_eq!(get_str(mb_configparser_get(p, s("s"), s("key"))), "val");
    }
}
