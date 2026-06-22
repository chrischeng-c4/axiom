use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// csv module for Mamba (#398).
///
/// CPython-parity csv: stateful reader / writer / DictReader / DictWriter
/// objects, the dialect registry, dialect classes (excel / excel-tab /
/// unix_dialect / Dialect), csv.Error, and a Sniffer.
///
/// Reader / writer / DictReader / DictWriter / Sniffer are registered as
/// real classes (via `mb_class_register`) carrying native methods. Their
/// instances dispatch `__iter__` / `__next__` / `writerow` / `writerows` /
/// `writeheader` / `sniff` / `has_header` through the generic instance
/// method path in `class.rs` — the same SystemV-ABI path io.StringIO's
/// `__iter__` uses — so no shared-file changes are needed.
///
/// The reader is a true stateful iterator: it parses one record per
/// `__next__`, advances `line_num`, and raises StopIteration at EOF.
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

thread_local! {
    static DIALECTS: RefCell<HashMap<String, MbValue>> = RefCell::new(HashMap::new());
    static FIELD_SIZE_LIMIT: RefCell<i64> = const { RefCell::new(131_072) };
}

// Quoting constants.
const QUOTE_MINIMAL: i64 = 0;
const QUOTE_ALL: i64 = 1;
const QUOTE_NONNUMERIC: i64 = 2;
const QUOTE_NONE: i64 = 3;
const QUOTE_STRINGS: i64 = 4;
const QUOTE_NOTNULL: i64 = 5;

// ──────────────────────────────────────────────────────────────────────
// Small value helpers
// ──────────────────────────────────────────────────────────────────────

/// Safe flat-args view: `slice::from_raw_parts` UB-panics on a null pointer
/// even with len 0 (which is how a 0-arg native call arrives). Guard it.
unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if args_ptr.is_null() || nargs == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
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

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn is_str(val: MbValue) -> bool {
    val.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
        .unwrap_or(false)
}

fn is_bytes(val: MbValue) -> bool {
    val.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
        .unwrap_or(false)
}

/// Read a named field off an `ObjData::Instance`.
fn instance_field(val: MbValue, key: &str) -> Option<MbValue> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn field_set(obj: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(key.into(), val);
            }
        }
    }
}

fn first_char(v: MbValue) -> Option<char> {
    extract_str(v).and_then(|s| s.chars().next())
}

fn raise(kind: &str, msg: impl Into<String>) -> MbValue {
    super::super::exception::mb_raise(new_str(kind), new_str(msg.into()));
    MbValue::none()
}

fn raise_type_error(msg: &str) -> MbValue {
    raise("TypeError", msg)
}
fn raise_csv_error(msg: &str) -> MbValue {
    raise("csv.Error", msg)
}

/// Python type name used in error messages.
fn type_name_of(val: MbValue) -> &'static str {
    if val.is_bool() {
        return "bool";
    }
    if val.is_int() {
        return "int";
    }
    if val.is_float() {
        return "float";
    }
    if val.is_none() {
        return "NoneType";
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match (*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                _ => "object",
            };
        }
    }
    "object"
}

fn make_instance(class_name: &str, fields: FxHashMap<String, MbValue>) -> MbValue {
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// ──────────────────────────────────────────────────────────────────────
// kwargs plumbing (trailing Dict positional)
// ──────────────────────────────────────────────────────────────────────

type KwMap = indexmap::IndexMap<super::super::dict_ops::DictKey, MbValue>;

fn trailing_kwargs(a: &[MbValue]) -> Option<KwMap> {
    a.last().and_then(|v| v.as_ptr()).and_then(|p| unsafe {
        if let ObjData::Dict(ref lock) = (*p).data {
            Some(lock.read().unwrap().clone())
        } else {
            None
        }
    })
}

fn kwarg_get(kw: &KwMap, key: &str) -> Option<MbValue> {
    for (k, v) in kw.iter() {
        if let super::super::dict_ops::DictKey::Str(ref ks) = k {
            if ks == key {
                return Some(*v);
            }
        }
    }
    None
}

// ──────────────────────────────────────────────────────────────────────
// Format parameters
// ──────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct CsvFmt {
    delimiter: char,
    quotechar: Option<char>,
    escapechar: Option<char>,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: String,
    quoting: i64,
    strict: bool,
}

impl Default for CsvFmt {
    fn default() -> Self {
        CsvFmt {
            delimiter: ',',
            quotechar: Some('"'),
            escapechar: None,
            doublequote: true,
            skipinitialspace: false,
            lineterminator: "\r\n".to_string(),
            quoting: QUOTE_MINIMAL,
            strict: false,
        }
    }
}

/// Pull a single dialect attribute off `dialect`, which may be:
/// - a registered dialect name string
/// - a dialect Instance (built-in, csv.Dialect subclass, or user instance)
/// Class attributes are resolved through the runtime getattr (so
/// `class X(csv.excel)` inherits excel's fields).
fn dialect_field(dialect: MbValue, key: &str) -> Option<MbValue> {
    if dialect.is_none() {
        return None;
    }
    // Registered dialect name.
    if let Some(name) = extract_str(dialect) {
        if let Some(found) = DIALECTS.with(|d| d.borrow().get(&name).copied()) {
            return getattr_opt(found, key);
        }
        // A class-name string that names a registered dialect class.
        return class_attr_opt(&name, key);
    }
    getattr_opt(dialect, key)
}

/// getattr that returns None if the attribute is genuinely missing,
/// without raising. Reads instance fields then class attrs through MRO.
fn getattr_opt(obj: MbValue, key: &str) -> Option<MbValue> {
    // Direct instance field.
    if let Some(v) = instance_field(obj, key) {
        return Some(v);
    }
    // Class attrs via MRO (works for user subclasses of csv.excel etc.).
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if let Some(v) = class_attr_opt(class_name, key) {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn class_attr_opt(class_name: &str, key: &str) -> Option<MbValue> {
    let v = super::super::class::lookup_method(class_name, key);
    if v.is_none() {
        None
    } else {
        Some(v)
    }
}

#[derive(PartialEq)]
enum FmtSource {
    Dialect,
    Kwarg,
}

/// Apply one set of overrides to `fmt`. Validates per CPython for the
/// kwarg source; the dialect-class source validates on instantiation
/// elsewhere. Returns false if validation raised.
fn apply_overrides(
    fmt: &mut CsvFmt,
    src: FmtSource,
    get: &dyn Fn(&str) -> Option<MbValue>,
) -> bool {
    let strict = src == FmtSource::Kwarg;

    if let Some(v) = get("delimiter") {
        if strict {
            if v.is_none() || !is_str(v) {
                raise_type_error(&format!(
                    "\"delimiter\" must be string, not {}",
                    type_name_of(v)
                ));
                return false;
            }
            let s = extract_str(v).unwrap_or_default();
            if s.chars().count() != 1 {
                raise_type_error("\"delimiter\" must be a 1-character string");
                return false;
            }
        }
        if let Some(c) = first_char(v) {
            fmt.delimiter = c;
        }
    }
    if let Some(v) = get("quotechar") {
        if v.is_none() {
            fmt.quotechar = None;
        } else {
            if strict && !is_str(v) {
                raise_type_error(&format!(
                    "\"quotechar\" must be string or None, not {}",
                    type_name_of(v)
                ));
                return false;
            }
            if strict {
                let s = extract_str(v).unwrap_or_default();
                if s.chars().count() != 1 {
                    raise_type_error("\"quotechar\" must be a 1-character string");
                    return false;
                }
            }
            fmt.quotechar = first_char(v);
        }
    }
    if let Some(v) = get("escapechar") {
        if v.is_none() {
            fmt.escapechar = None;
        } else {
            if strict && !is_str(v) {
                raise_type_error(&format!(
                    "\"escapechar\" must be string or None, not {}",
                    type_name_of(v)
                ));
                return false;
            }
            fmt.escapechar = first_char(v);
        }
    }
    if let Some(v) = get("doublequote") {
        fmt.doublequote = truthy(v);
    }
    if let Some(v) = get("skipinitialspace") {
        fmt.skipinitialspace = truthy(v);
    }
    if let Some(v) = get("lineterminator") {
        if strict && (v.is_none() || !is_str(v)) {
            raise_type_error("\"lineterminator\" must be a string");
            return false;
        }
        if let Some(s) = extract_str(v) {
            fmt.lineterminator = s;
        }
    }
    if let Some(v) = get("quoting") {
        if strict {
            match v.as_int() {
                Some(q) if (0..=5).contains(&q) && !v.is_bool() => {
                    fmt.quoting = q;
                }
                _ => {
                    raise_type_error("bad \"quoting\" value");
                    return false;
                }
            }
        } else if let Some(q) = v.as_int() {
            fmt.quoting = q;
        }
    }
    if let Some(v) = get("strict") {
        fmt.strict = truthy(v);
    }
    true
}

fn truthy(v: MbValue) -> bool {
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(i) = v.as_int() {
        return i != 0;
    }
    !v.is_none()
}

/// Resolve format from a positional dialect plus kwargs. Returns None
/// (after raising) on a validation error.
fn resolve_fmt(dialect: MbValue, kw: &Option<KwMap>) -> Option<CsvFmt> {
    let mut fmt = CsvFmt::default();

    // A `dialect=` kwarg overrides/supplies the dialect when no positional
    // dialect was given (CPython lets either form name the base dialect).
    let dialect = if dialect.is_none() {
        kw.as_ref()
            .and_then(|m| kwarg_get(m, "dialect"))
            .unwrap_or_else(MbValue::none)
    } else {
        dialect
    };

    // Bare-string dialect that isn't a registered name → csv.Error.
    if let Some(name) = extract_str(dialect) {
        let known = DIALECTS.with(|d| d.borrow().contains_key(&name))
            || super::super::class::class_is_registered(&name);
        if !known {
            raise_csv_error(&format!("unknown dialect: {name:?}"));
            return None;
        }
    }

    if !dialect.is_none() {
        if !apply_overrides(&mut fmt, FmtSource::Dialect, &|k| dialect_field(dialect, k)) {
            return None;
        }
    }
    if let Some(m) = kw {
        // Reject unknown kwargs (bad_attr) — CPython TypeErrors.
        for (k, _v) in m.iter() {
            if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                if !matches!(
                    ks.as_str(),
                    "delimiter"
                        | "quotechar"
                        | "escapechar"
                        | "doublequote"
                        | "skipinitialspace"
                        | "lineterminator"
                        | "quoting"
                        | "strict"
                        | "dialect"
                ) {
                    raise_type_error(&format!(
                        "'{ks}' is an invalid keyword argument for this function"
                    ));
                    return None;
                }
            }
        }
        if !apply_overrides(&mut fmt, FmtSource::Kwarg, &|k| kwarg_get(m, k)) {
            return None;
        }
    }

    // Cross-field consistency: a quoting mode that emits quotes (ALL /
    // NONNUMERIC / STRINGS / NOTNULL) requires a quotechar. QUOTE_MINIMAL
    // and QUOTE_NONE work with quotechar=None.
    let needs_quotechar = matches!(
        fmt.quoting,
        QUOTE_ALL | QUOTE_NONNUMERIC | QUOTE_STRINGS | QUOTE_NOTNULL
    );
    if fmt.quotechar.is_none() && needs_quotechar {
        raise_type_error("quotechar must be set if quoting enabled");
        return None;
    }
    Some(fmt)
}

/// Build a dialect Instance carrying the resolved fmt as attributes,
/// returned as `reader.dialect` / `writer.dialect`.
fn fmt_to_dialect(fmt: &CsvFmt) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("delimiter".into(), new_str(fmt.delimiter.to_string()));
    fields.insert(
        "quotechar".into(),
        match fmt.quotechar {
            Some(c) => new_str(c.to_string()),
            None => MbValue::none(),
        },
    );
    fields.insert(
        "escapechar".into(),
        match fmt.escapechar {
            Some(c) => new_str(c.to_string()),
            None => MbValue::none(),
        },
    );
    fields.insert("doublequote".into(), MbValue::from_bool(fmt.doublequote));
    fields.insert(
        "skipinitialspace".into(),
        MbValue::from_bool(fmt.skipinitialspace),
    );
    fields.insert("lineterminator".into(), new_str(fmt.lineterminator.clone()));
    fields.insert("quoting".into(), MbValue::from_int(fmt.quoting));
    fields.insert("strict".into(), MbValue::from_bool(fmt.strict));
    make_instance("csv.Dialect", fields)
}

// ──────────────────────────────────────────────────────────────────────
// Source line splitting
// ──────────────────────────────────────────────────────────────────────

/// Split raw text into CSV records. A newline inside a quoted field stays
/// in one record. Each record string EXCLUDES the line terminator.
/// Mirrors how CPython's reader sees `next(file)` lines: an embedded
/// quoted newline keeps the record open.
fn split_records(text: &str, quotechar: Option<char>) -> Vec<String> {
    let mut records = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if Some(c) == quotechar {
            in_quotes = !in_quotes;
            current.push(c);
            continue;
        }
        match c {
            '\r' if !in_quotes => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                records.push(std::mem::take(&mut current));
            }
            '\n' if !in_quotes => {
                records.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

/// Treat one iterable element as a single source line: drop a single
/// trailing line terminator (CRLF / LF / CR). An element with an embedded
/// (unquoted) newline still produces multiple records.
fn line_to_records(s: &str, quotechar: Option<char>) -> Vec<String> {
    // Strip one trailing terminator, then split on any interior newlines.
    let trimmed = if let Some(rest) = s.strip_suffix("\r\n") {
        rest
    } else if let Some(rest) = s.strip_suffix('\n') {
        rest
    } else if let Some(rest) = s.strip_suffix('\r') {
        rest
    } else {
        s
    };
    // After stripping the trailing terminator, an empty element is one
    // empty record (CPython: csv.reader(['']) → [[]]).
    if trimmed.is_empty() {
        return vec![String::new()];
    }
    // Honor embedded unquoted newlines but DON'T drop a trailing empty.
    let mut recs = split_records(trimmed, quotechar);
    if recs.is_empty() {
        recs.push(String::new());
    }
    recs
}

/// Pull all source records out of a reader source value.
/// Accepts str / list-or-iterable of str / file-like Instance.
fn source_records(csvfile: MbValue, quotechar: Option<char>) -> Option<Vec<String>> {
    if let Some(s) = extract_str(csvfile) {
        return Some(split_records(&s, quotechar));
    }
    if let Some(ptr) = csvfile.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                let mut out = Vec::new();
                for v in items.iter() {
                    if let Some(s) = extract_str(*v) {
                        out.extend(line_to_records(&s, quotechar));
                    }
                }
                return Some(out);
            }
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let mut out = Vec::new();
                for v in items.iter() {
                    if let Some(s) = extract_str(*v) {
                        out.extend(line_to_records(&s, quotechar));
                    }
                }
                return Some(out);
            }
            if let ObjData::Instance { .. } = (*ptr).data {
                let text = read_filelike(csvfile);
                return Some(split_records(&text, quotechar));
            }
        }
    }
    // Generic iterable: drain through the iterator protocol.
    let handle = super::super::iter::mb_iter(csvfile);
    if !handle.is_none() && handle.as_int().is_some() {
        let mut out = Vec::new();
        loop {
            if super::super::iter::mb_has_next(handle).as_bool() != Some(true) {
                break;
            }
            let v = super::super::iter::mb_next(handle);
            if let Some(s) = extract_str(v) {
                out.extend(line_to_records(&s, quotechar));
            }
        }
        return Some(out);
    }
    None
}

fn read_filelike(fileobj: MbValue) -> String {
    let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
    let res = super::super::class::mb_call_method(fileobj, new_str("read"), empty);
    if let Some(s) = extract_str(res) {
        return s;
    }
    if let Some(buf) = instance_field(fileobj, "_buffer") {
        if let Some(s) = extract_str(buf) {
            return s;
        }
    }
    String::new()
}

// ──────────────────────────────────────────────────────────────────────
// Parser (one record → fields)
// ──────────────────────────────────────────────────────────────────────

/// Field value, tracking whether it was quoted (for QUOTE_NONNUMERIC).
struct ParsedField {
    text: String,
    quoted: bool,
}

/// Strict-mode parse error description (CPython csv.Error messages).
enum ParseError {
    UnexpectedAfterQuote, // '%c' expected after '%c'
    NewlineInQuoted,      // unexpected end of data
}

/// Parse one record into typed-or-string field values per fmt.
/// Returns None and raises on a QUOTE_NONNUMERIC non-quoted non-number
/// or a strict-mode malformation.
fn parse_record_values(record: &str, fmt: &CsvFmt) -> Option<Vec<MbValue>> {
    let (raw, err) = parse_record(record, fmt);
    if fmt.strict {
        if let Some(e) = err {
            match e {
                ParseError::UnexpectedAfterQuote => {
                    let qc = fmt.quotechar.unwrap_or('"');
                    raise_csv_error(&format!("'{qc}' expected after '{qc}'"));
                }
                ParseError::NewlineInQuoted => {
                    raise_csv_error("unexpected end of data");
                }
            }
            return None;
        }
    }
    // Enforce the field size limit (CPython raises csv.Error when a field
    // exceeds it).
    let limit = FIELD_SIZE_LIMIT.with(|f| *f.borrow());
    if limit >= 0 {
        for f in &raw {
            if f.text.chars().count() as i64 > limit {
                raise_csv_error(&format!("field larger than field limit ({limit})"));
                return None;
            }
        }
    }
    let mut out = Vec::with_capacity(raw.len());
    for f in raw {
        if fmt.quoting == QUOTE_NONNUMERIC && !f.quoted {
            // Unquoted field under QUOTE_NONNUMERIC → number (CPython runs
            // float() on it). float() strips surrounding whitespace.
            if f.text.is_empty() {
                // empty unquoted field: CPython yields '' for empty.
                out.push(new_str(String::new()));
                continue;
            }
            // CPython runs float() on every unquoted field under
            // QUOTE_NONNUMERIC; float() strips surrounding whitespace. The
            // result is always a float (`3` → 3.0), but `3.0 == 3` so the
            // `==`-based fixtures accept either.
            let trimmed = f.text.trim();
            match trimmed.parse::<f64>() {
                Ok(v) => out.push(MbValue::from_float(v)),
                Err(_) => {
                    // CPython raises ValueError (from float()), not csv.Error.
                    raise(
                        "ValueError",
                        format!("could not convert string to float: {:?}", f.text),
                    );
                    return None;
                }
            }
        } else {
            out.push(new_str(f.text));
        }
    }
    Some(out)
}

/// Core record→fields state machine. Honors delimiter, quotechar,
/// escapechar, doublequote, skipinitialspace, quoting. The second return
/// element is set when strict-mode would reject the record.
fn parse_record(record: &str, fmt: &CsvFmt) -> (Vec<ParsedField>, Option<ParseError>) {
    let mut fields: Vec<ParsedField> = Vec::new();
    let mut current = String::new();
    let mut field_quoted = false;
    let mut in_quotes = false;
    let mut after_quote = false; // saw a closing quote; awaiting delimiter
    let mut field_has_content = false;
    let mut at_field_start = true;
    let mut err: Option<ParseError> = None;
    let mut chars = record.chars().peekable();

    let quotechar = fmt.quotechar;
    let quoting_none = fmt.quoting == QUOTE_NONE;

    while let Some(c) = chars.next() {
        if in_quotes {
            if fmt.escapechar == Some(c) {
                if let Some(n) = chars.next() {
                    current.push(n);
                }
                continue;
            }
            if Some(c) == quotechar {
                if fmt.doublequote && chars.peek() == quotechar.as_ref() {
                    current.push(c);
                    chars.next();
                } else {
                    in_quotes = false;
                    after_quote = true;
                }
            } else {
                current.push(c);
            }
            continue;
        }

        // Not in quotes.
        // After a closing quote, any non-delimiter content (incl. the
        // escapechar) is appended literally — CPython:
        // csv.reader(['a,"b,c"\\'], escapechar='\\') → [['a', 'b,c\\']].
        // In strict mode this is an error.
        if after_quote && c != fmt.delimiter {
            if err.is_none() {
                err = Some(ParseError::UnexpectedAfterQuote);
            }
            current.push(c);
            field_has_content = true;
            continue;
        }
        if fmt.escapechar == Some(c) {
            // The escaped char is taken literally. At end-of-record the
            // escaped char is the implicit record terminator → '\n'
            // (CPython: csv.reader(['^'], escapechar='^') → [['\n']]); in
            // strict mode an escapechar at end-of-data is an error.
            match chars.next() {
                Some(n) => current.push(n),
                None => {
                    current.push('\n');
                    if err.is_none() {
                        err = Some(ParseError::NewlineInQuoted);
                    }
                }
            }
            field_has_content = true;
            at_field_start = false;
            after_quote = false;
            continue;
        }
        // skipinitialspace: a space immediately after a delimiter (i.e. at
        // field start with no content yet) is skipped — even when the
        // delimiter itself is a space (CPython collapses `a   b` →
        // ['a', 'b'] with delimiter=' ', skipinitialspace=True).
        if fmt.skipinitialspace && at_field_start && c == ' ' && !field_has_content {
            continue;
        }
        if c == fmt.delimiter {
            fields.push(ParsedField {
                text: std::mem::take(&mut current),
                quoted: field_quoted,
            });
            field_quoted = false;
            field_has_content = false;
            at_field_start = true;
            after_quote = false;
            continue;
        }
        if !quoting_none
            && Some(c) == quotechar
            && at_field_start
            && current.is_empty()
            && !after_quote
        {
            in_quotes = true;
            field_quoted = true;
            at_field_start = false;
            continue;
        }
        if after_quote {
            // Content after a closing quote (e.g. `"a"b`): CPython appends.
            after_quote = false;
        }
        current.push(c);
        field_has_content = true;
        at_field_start = false;
    }
    // Record ended inside a quoted field → unterminated quote.
    if in_quotes && err.is_none() {
        err = Some(ParseError::NewlineInQuoted);
    }
    fields.push(ParsedField {
        text: current,
        quoted: field_quoted,
    });
    (fields, err)
}

// ──────────────────────────────────────────────────────────────────────
// Writer formatting
// ──────────────────────────────────────────────────────────────────────

/// Is this value numeric for QUOTE_NONNUMERIC purposes (int/float, not bool/None/str).
fn is_numeric(v: MbValue) -> bool {
    if v.is_bool() {
        return false;
    }
    v.is_int() || v.is_float()
}

/// Render one field value to its CSV string, applying quoting rules.
/// Returns None (after raising) when QUOTE_NONE needs escaping but has no escapechar.
fn format_field(v: MbValue, fmt: &CsvFmt, single_none_field: bool) -> Option<String> {
    let is_none = v.is_none();
    let numeric = is_numeric(v);

    // Stringify the value (None → "").
    let s = if is_none {
        String::new()
    } else {
        extract_str(super::super::builtins::mb_str(v)).unwrap_or_default()
    };

    // Decide whether to quote.
    let quote = match fmt.quoting {
        QUOTE_ALL => true,
        QUOTE_NONNUMERIC => !numeric,
        QUOTE_STRINGS => is_str(v),
        QUOTE_NOTNULL => !is_none,
        QUOTE_NONE => false,
        _ /* QUOTE_MINIMAL */ => {
            needs_minimal_quote(&s, fmt) || (single_none_field && is_none)
        }
    };

    let qc = fmt.quotechar;

    if fmt.quoting == QUOTE_NONE {
        // Escape special chars with escapechar; error if none available.
        let mut out = String::new();
        for ch in s.chars() {
            if ch == fmt.delimiter
                || Some(ch) == qc
                || ch == '\r'
                || ch == '\n'
                || Some(ch) == fmt.escapechar
            {
                match fmt.escapechar {
                    Some(e) => {
                        out.push(e);
                        out.push(ch);
                    }
                    None => {
                        raise_csv_error("need to escape, but no escapechar set");
                        return None;
                    }
                }
            } else {
                out.push(ch);
            }
        }
        return Some(out);
    }

    if quote {
        let qc = qc.unwrap_or('"');
        let mut out = String::new();
        out.push(qc);
        for ch in s.chars() {
            if ch == qc {
                if fmt.doublequote {
                    out.push(qc);
                    out.push(qc);
                } else if let Some(e) = fmt.escapechar {
                    out.push(e);
                    out.push(ch);
                } else {
                    out.push(ch);
                }
            } else {
                out.push(ch);
            }
        }
        out.push(qc);
        Some(out)
    } else {
        // Unquoted; if the value contains specials and we couldn't quote
        // (QUOTE_MINIMAL would have quoted), escape with escapechar.
        if needs_minimal_quote(&s, fmt) {
            if let Some(e) = fmt.escapechar {
                let mut out = String::new();
                for ch in s.chars() {
                    if ch == fmt.delimiter
                        || ch == '\r'
                        || ch == '\n'
                        || Some(ch) == qc
                        || Some(ch) == fmt.escapechar
                    {
                        out.push(e);
                    }
                    out.push(ch);
                }
                return Some(out);
            }
        }
        Some(s)
    }
}

fn needs_minimal_quote(s: &str, fmt: &CsvFmt) -> bool {
    let qc = fmt.quotechar;
    s.chars()
        .any(|ch| ch == fmt.delimiter || ch == '\r' || ch == '\n' || Some(ch) == qc)
}

/// Build the formatted record string for one row of values.
/// Returns None on a formatting error (already raised).
fn format_row(row: &[MbValue], fmt: &CsvFmt) -> Option<String> {
    let single_none = row.len() == 1;
    let mut parts: Vec<String> = Vec::with_capacity(row.len());
    for v in row {
        match format_field(*v, fmt, single_none) {
            Some(s) => parts.push(s),
            None => return None,
        }
    }
    let mut line = parts.join(&fmt.delimiter.to_string());
    line.push_str(&fmt.lineterminator);
    Some(line)
}

/// Collect a row argument (list/tuple/iterable/generator) into Vec<MbValue>.
fn collect_row(row: MbValue) -> Option<Vec<MbValue>> {
    if let Some(ptr) = row.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return Some(lock.read().unwrap().to_vec()),
                ObjData::Tuple(items) => return Some(items.clone()),
                _ => {}
            }
        }
    }
    if row.is_none() {
        return None;
    }
    // Iterable / generator.
    let handle = super::super::iter::mb_iter(row);
    if handle.is_none() || handle.as_int().is_none() {
        return None;
    }
    let mut out = Vec::new();
    loop {
        if super::super::iter::mb_has_next(handle).as_bool() != Some(true) {
            break;
        }
        out.push(super::super::iter::mb_next(handle));
    }
    Some(out)
}

// ──────────────────────────────────────────────────────────────────────
// Reader object
// ──────────────────────────────────────────────────────────────────────

fn store_fmt(obj: MbValue, fmt: &CsvFmt) {
    field_set(obj, "_delimiter", new_str(fmt.delimiter.to_string()));
    field_set(
        obj,
        "_quotechar",
        match fmt.quotechar {
            Some(c) => new_str(c.to_string()),
            None => MbValue::none(),
        },
    );
    field_set(
        obj,
        "_escapechar",
        match fmt.escapechar {
            Some(c) => new_str(c.to_string()),
            None => MbValue::none(),
        },
    );
    field_set(obj, "_doublequote", MbValue::from_bool(fmt.doublequote));
    field_set(
        obj,
        "_skipinitialspace",
        MbValue::from_bool(fmt.skipinitialspace),
    );
    field_set(obj, "_lineterminator", new_str(fmt.lineterminator.clone()));
    field_set(obj, "_quoting", MbValue::from_int(fmt.quoting));
    field_set(obj, "_strict", MbValue::from_bool(fmt.strict));
}

fn load_fmt(obj: MbValue) -> CsvFmt {
    let mut fmt = CsvFmt::default();
    if let Some(v) = instance_field(obj, "_delimiter") {
        if let Some(c) = first_char(v) {
            fmt.delimiter = c;
        }
    }
    if let Some(v) = instance_field(obj, "_quotechar") {
        fmt.quotechar = if v.is_none() { None } else { first_char(v) };
    }
    if let Some(v) = instance_field(obj, "_escapechar") {
        fmt.escapechar = if v.is_none() { None } else { first_char(v) };
    }
    if let Some(v) = instance_field(obj, "_doublequote") {
        fmt.doublequote = truthy(v);
    }
    if let Some(v) = instance_field(obj, "_skipinitialspace") {
        fmt.skipinitialspace = truthy(v);
    }
    if let Some(v) = instance_field(obj, "_lineterminator") {
        if let Some(s) = extract_str(v) {
            fmt.lineterminator = s;
        }
    }
    if let Some(v) = instance_field(obj, "_quoting") {
        if let Some(q) = v.as_int() {
            fmt.quoting = q;
        }
    }
    if let Some(v) = instance_field(obj, "_strict") {
        fmt.strict = truthy(v);
    }
    fmt
}

/// For an iterable (list/tuple/iterator) source — NOT a str/file — find the
/// record index at which CPython would raise "new-line character seen in
/// unquoted field": an element contains an unquoted \r or \n that is not its
/// sole trailing terminator. Returns the cumulative record index of the
/// offending element, if any.
fn embedded_newline_error_index(csvfile: MbValue, quotechar: Option<char>) -> Option<usize> {
    let elements: Vec<String> = if let Some(ptr) = csvfile.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock
                    .read()
                    .unwrap()
                    .iter()
                    .filter_map(|x| extract_str(*x))
                    .collect(),
                ObjData::Tuple(t) => t.iter().filter_map(|x| extract_str(*x)).collect(),
                _ => return None, // str / file: real multi-line content is fine
            }
        }
    } else {
        return None;
    };

    let mut rec_index = 0usize;
    for el in &elements {
        let recs = line_to_records(el, quotechar);
        // An element splitting into >1 record means an interior unquoted
        // newline — CPython errors on it.
        if recs.len() > 1 {
            return Some(rec_index);
        }
        rec_index += recs.len();
    }
    None
}

fn build_reader(csvfile: MbValue, fmt: CsvFmt) -> MbValue {
    let records = source_records(csvfile, fmt.quotechar).unwrap_or_default();
    let newline_err = embedded_newline_error_index(csvfile, fmt.quotechar);

    // strict / QUOTE_NONNUMERIC error semantics: CPython raises lazily as
    // each malformed record is consumed, but the runtime iterator protocol
    // swallows a non-StopIteration exception raised from __next__ during
    // `list(reader)`. To keep parity for the common `list(csv.reader(...))`
    // form, validate eagerly here so the error surfaces at construction
    // (still inside the caller's try/except). Plain readers stay lazy.
    let limit = FIELD_SIZE_LIMIT.with(|f| *f.borrow());
    let may_exceed_limit = limit >= 0 && records.iter().any(|r| r.chars().count() as i64 > limit);
    if fmt.strict || fmt.quoting == QUOTE_NONNUMERIC || may_exceed_limit {
        for rec in &records {
            if rec.is_empty() {
                continue;
            }
            if parse_record_values(rec, &fmt).is_none() {
                // Error already raised; return None so the caller propagates it.
                return MbValue::none();
            }
        }
    }

    let rec_vals: Vec<MbValue> = records.into_iter().map(new_str).collect();
    let mut fields = FxHashMap::default();
    fields.insert(
        "_records".into(),
        MbValue::from_ptr(MbObject::new_list(rec_vals)),
    );
    fields.insert("_idx".into(), MbValue::from_int(0));
    fields.insert("line_num".into(), MbValue::from_int(0));
    fields.insert("dialect".into(), fmt_to_dialect(&fmt));
    if let Some(idx) = newline_err {
        fields.insert("_newline_err_at".into(), MbValue::from_int(idx as i64));
    }
    let obj = make_instance("_csv.reader", fields);
    store_fmt(obj, &fmt);
    obj
}

/// reader.__iter__(self) → self.
pub extern "C" fn reader_iter(slf: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(slf);
    }
    slf
}

/// reader.__next__(self) → next row (list) or StopIteration.
pub extern "C" fn reader_next(slf: MbValue) -> MbValue {
    let records = match instance_field(slf, "_records") {
        Some(v) => v,
        None => MbValue::none(),
    };
    let idx = instance_field(slf, "_idx")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    let line_num = instance_field(slf, "line_num")
        .and_then(|v| v.as_int())
        .unwrap_or(0);

    let len = records
        .as_ptr()
        .map(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                lock.read().unwrap().len() as i64
            } else {
                0
            }
        })
        .unwrap_or(0);

    if idx >= len {
        super::super::exception::mb_raise(new_str("StopIteration"), new_str(""));
        return MbValue::none();
    }

    // An iterable element carried an unquoted embedded newline at this index.
    if instance_field(slf, "_newline_err_at").and_then(|v| v.as_int()) == Some(idx) {
        field_set(slf, "_idx", MbValue::from_int(idx + 1));
        return raise_csv_error(
            "new-line character seen in unquoted field - do you need to open the file with newline=''?");
    }

    let record = records
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                lock.read().unwrap().get(idx as usize).copied()
            } else {
                None
            }
        })
        .unwrap_or_else(MbValue::none);

    field_set(slf, "_idx", MbValue::from_int(idx + 1));
    field_set(slf, "line_num", MbValue::from_int(line_num + 1));

    let rec_str = extract_str(record).unwrap_or_default();
    // An empty record (blank line) yields an empty row [].
    if rec_str.is_empty() {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }
    let fmt = load_fmt(slf);
    match parse_record_values(&rec_str, &fmt) {
        Some(vals) => MbValue::from_ptr(MbObject::new_list(vals)),
        None => MbValue::none(), // error already raised
    }
}

// ──────────────────────────────────────────────────────────────────────
// Writer object
// ──────────────────────────────────────────────────────────────────────

fn build_writer(fileobj: MbValue, fmt: CsvFmt) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(fileobj);
    }
    let mut fields = FxHashMap::default();
    fields.insert("_file".into(), fileobj);
    fields.insert("dialect".into(), fmt_to_dialect(&fmt));
    let obj = make_instance("_csv.writer", fields);
    store_fmt(obj, &fmt);
    obj
}

/// Write a formatted string to the writer's file object, returning char count.
fn writer_emit(slf: MbValue, text: &str) -> i64 {
    let file = instance_field(slf, "_file").unwrap_or_else(MbValue::none);
    let args = MbValue::from_ptr(MbObject::new_list(vec![new_str(text)]));
    super::super::class::mb_call_method(file, new_str("write"), args);
    text.chars().count() as i64
}

/// writer.writerow(self, row) → char count.
pub extern "C" fn writer_writerow(slf: MbValue, row: MbValue) -> MbValue {
    let fmt = load_fmt(slf);
    let items = match collect_row(row) {
        Some(v) => v,
        None => return raise_type_error("writerow() argument must be iterable"),
    };
    match format_row(&items, &fmt) {
        Some(line) => MbValue::from_int(writer_emit(slf, &line)),
        None => MbValue::none(),
    }
}

/// writer.writerows(self, rows) → None.
pub extern "C" fn writer_writerows(slf: MbValue, rows: MbValue) -> MbValue {
    let fmt = load_fmt(slf);
    // Iterate rows.
    let row_list = collect_row(rows);
    let row_list = match row_list {
        Some(v) => v,
        None => return raise_type_error("writerows() argument must be iterable"),
    };
    for row in row_list {
        let items = match collect_row(row) {
            Some(v) => v,
            None => return raise_type_error("writerows() argument must be iterable of iterables"),
        };
        match format_row(&items, &fmt) {
            Some(line) => {
                writer_emit(slf, &line);
            }
            None => return MbValue::none(),
        }
    }
    MbValue::none()
}

// ──────────────────────────────────────────────────────────────────────
// DictReader
// ──────────────────────────────────────────────────────────────────────

/// Materialize a fieldnames argument (list / tuple / iterator / str) to Vec<String>.
fn materialize_names(v: MbValue) -> Option<Vec<String>> {
    if v.is_none() {
        return None;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => {
                    return Some(
                        lock.read()
                            .unwrap()
                            .iter()
                            .filter_map(|x| extract_str(*x))
                            .collect(),
                    );
                }
                ObjData::Tuple(items) => {
                    return Some(items.iter().filter_map(|x| extract_str(*x)).collect());
                }
                _ => {}
            }
        }
    }
    // iterator / generator
    let handle = super::super::iter::mb_iter(v);
    if !handle.is_none() && handle.as_int().is_some() {
        let mut out = Vec::new();
        loop {
            if super::super::iter::mb_has_next(handle).as_bool() != Some(true) {
                break;
            }
            let x = super::super::iter::mb_next(handle);
            if let Some(s) = extract_str(x) {
                out.push(s);
            }
        }
        return Some(out);
    }
    None
}

fn names_to_list(names: &[String]) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(
        names.iter().map(|n| new_str(n.clone())).collect(),
    ))
}

fn build_dictreader(
    f: MbValue,
    fieldnames: MbValue,
    restkey: MbValue,
    restval: MbValue,
    dialect: MbValue,
    kw: Option<KwMap>,
) -> MbValue {
    let fmt = match resolve_fmt(dialect, &kw) {
        Some(f) => f,
        None => return MbValue::none(),
    };
    let reader = build_reader(f, fmt);

    let mut fields = FxHashMap::default();
    fields.insert("_reader".into(), reader);
    fields.insert("restkey".into(), restkey);
    fields.insert("restval".into(), restval);
    // fieldnames: materialize now if provided, else infer eagerly from the
    // first row so the `.fieldnames` attribute reads back a list.
    let names = match materialize_names(fieldnames) {
        Some(n) => n,
        None => infer_fieldnames(reader),
    };
    fields.insert("fieldnames".into(), names_to_list(&names));
    fields.insert("line_num".into(), MbValue::from_int(0));
    make_instance("csv.DictReader", fields)
}

/// Read the reader's first record as headers (consuming it).
fn infer_fieldnames(reader: MbValue) -> Vec<String> {
    let row = reader_next(reader);
    if super::super::exception::current_exception_type().as_deref() == Some("StopIteration") {
        super::super::exception::mb_clear_exception();
        return Vec::new();
    }
    row.as_ptr()
        .map(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                lock.read()
                    .unwrap()
                    .iter()
                    .filter_map(|x| extract_str(*x))
                    .collect()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default()
}

/// Resolve fieldnames (already materialized at construction).
fn dictreader_fieldnames(slf: MbValue) -> Vec<String> {
    instance_field(slf, "fieldnames")
        .and_then(materialize_names)
        .unwrap_or_default()
}

/// DictReader.__iter__(self) → self.
pub extern "C" fn dictreader_iter(slf: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(slf);
    }
    slf
}

/// DictReader.__next__(self) → dict or StopIteration.
pub extern "C" fn dictreader_next(slf: MbValue) -> MbValue {
    let names = dictreader_fieldnames(slf);
    let reader = instance_field(slf, "_reader").unwrap_or_else(MbValue::none);

    // Skip blank rows (CPython: while row == []).
    let mut row_vals: Vec<MbValue>;
    loop {
        let row = reader_next(reader);
        if super::super::exception::current_exception_type().as_deref() == Some("StopIteration") {
            return MbValue::none();
        }
        row_vals = row
            .as_ptr()
            .map(|p| unsafe {
                if let ObjData::List(ref lock) = (*p).data {
                    lock.read().unwrap().to_vec()
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default();
        if !row_vals.is_empty() {
            break;
        }
    }

    // line_num mirrors the reader's.
    if let Some(ln) = instance_field(reader, "line_num") {
        field_set(slf, "line_num", ln);
    }

    let restval = instance_field(slf, "restval").unwrap_or_else(MbValue::none);
    let restkey = instance_field(slf, "restkey").unwrap_or_else(MbValue::none);

    // Build via mb_dict_setitem so the restkey can be the actual None object
    // (CPython's default restkey) — extras then land under the None key.
    let dict = MbValue::from_ptr(MbObject::new_dict());
    for (i, name) in names.iter().enumerate() {
        let val = row_vals.get(i).copied().unwrap_or_else(|| {
            unsafe {
                super::super::rc::retain_if_ptr(restval);
            }
            restval
        });
        super::super::dict_ops::mb_dict_setitem(dict, new_str(name.clone()), val);
    }
    // Extra values → restkey (default None).
    if row_vals.len() > names.len() {
        let extras: Vec<MbValue> = row_vals[names.len()..].to_vec();
        super::super::dict_ops::mb_dict_setitem(
            dict,
            restkey,
            MbValue::from_ptr(MbObject::new_list(extras)),
        );
    }
    dict
}

// ──────────────────────────────────────────────────────────────────────
// DictWriter
// ──────────────────────────────────────────────────────────────────────

fn build_dictwriter(
    f: MbValue,
    fieldnames: MbValue,
    restval: MbValue,
    extrasaction: MbValue,
    dialect: MbValue,
    kw: Option<KwMap>,
) -> MbValue {
    // fieldnames is required.
    let names = match materialize_names(fieldnames) {
        Some(n) => n,
        None => {
            return raise_type_error(
                "__init__() missing 1 required positional argument: 'fieldnames'",
            )
        }
    };
    let ea = if extrasaction.is_none() {
        "raise".to_string()
    } else {
        extract_str(extrasaction).unwrap_or_else(|| "raise".into())
    };
    // CPython validates case-insensitively (extrasaction.lower()).
    let ea_l = ea.to_ascii_lowercase();
    if ea_l != "raise" && ea_l != "ignore" {
        return raise(
            "ValueError",
            format!("extrasaction ({ea}) must be 'raise' or 'ignore'"),
        );
    }
    let fmt = match resolve_fmt(dialect, &kw) {
        Some(f) => f,
        None => return MbValue::none(),
    };
    let writer = build_writer(f, fmt);

    let mut fields = FxHashMap::default();
    fields.insert("_writer".into(), writer);
    fields.insert("fieldnames".into(), names_to_list(&names));
    fields.insert(
        "restval".into(),
        if restval.is_none() {
            new_str(String::new())
        } else {
            restval
        },
    );
    fields.insert("extrasaction".into(), new_str(ea));
    make_instance("csv.DictWriter", fields)
}

fn dictwriter_names(slf: MbValue) -> Vec<String> {
    instance_field(slf, "fieldnames")
        .and_then(materialize_names)
        .unwrap_or_default()
}

/// DictWriter.writeheader(self) → char count.
pub extern "C" fn dictwriter_writeheader(slf: MbValue) -> MbValue {
    let names = dictwriter_names(slf);
    let writer = instance_field(slf, "_writer").unwrap_or_else(MbValue::none);
    let row = MbValue::from_ptr(MbObject::new_list(
        names.iter().map(|n| new_str(n.clone())).collect(),
    ));
    writer_writerow(writer, row)
}

/// Map a dict row to the ordered field values per the writer's fieldnames.
fn dict_to_row(slf: MbValue, rowdict: MbValue) -> Option<Vec<MbValue>> {
    let names = dictwriter_names(slf);
    let restval = instance_field(slf, "restval").unwrap_or_else(|| new_str(String::new()));
    let extrasaction = instance_field(slf, "extrasaction")
        .and_then(extract_str)
        .unwrap_or_else(|| "raise".into());

    if extrasaction.eq_ignore_ascii_case("raise") {
        // Collect ALL dict keys (any type) and report those not in fieldnames,
        // preserving insertion order and Python repr (str → 'x', int → 1).
        let extras: Vec<String> = rowdict
            .as_ptr()
            .map(|p| unsafe {
                if let ObjData::Dict(ref lock) = (*p).data {
                    lock.read()
                        .unwrap()
                        .keys()
                        .filter_map(|k| match k {
                            super::super::dict_ops::DictKey::Str(s) if !names.contains(s) => {
                                Some(format!("'{s}'"))
                            }
                            super::super::dict_ops::DictKey::Int(i) => Some(format!("{i}")),
                            _ => None,
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default();
        if !extras.is_empty() {
            let joined = extras.join(", ");
            raise(
                "ValueError",
                format!("dict contains fields not in fieldnames: {joined}"),
            );
            return None;
        }
    }

    let mut row = Vec::with_capacity(names.len());
    for name in &names {
        let v = rowdict.as_ptr().and_then(|p| unsafe {
            if let ObjData::Dict(ref lock) = (*p).data {
                lock.read()
                    .unwrap()
                    .get(&super::super::dict_ops::DictKey::Str(name.clone()))
                    .copied()
            } else {
                None
            }
        });
        match v {
            Some(val) => row.push(val),
            None => {
                unsafe {
                    super::super::rc::retain_if_ptr(restval);
                }
                row.push(restval);
            }
        }
    }
    Some(row)
}

/// DictWriter.writerow(self, rowdict) → char count.
pub extern "C" fn dictwriter_writerow(slf: MbValue, rowdict: MbValue) -> MbValue {
    let row = match dict_to_row(slf, rowdict) {
        Some(r) => r,
        None => return MbValue::none(),
    };
    let writer = instance_field(slf, "_writer").unwrap_or_else(MbValue::none);
    let row_val = MbValue::from_ptr(MbObject::new_list(row));
    writer_writerow(writer, row_val)
}

/// DictWriter.writerows(self, rowdicts) → None.
pub extern "C" fn dictwriter_writerows(slf: MbValue, rowdicts: MbValue) -> MbValue {
    let list = match collect_row(rowdicts) {
        Some(v) => v,
        None => return raise_type_error("writerows() argument must be iterable"),
    };
    for rowdict in list {
        let r = dictwriter_writerow(slf, rowdict);
        if r.is_none() && super::super::exception::current_exception_type().is_some() {
            return MbValue::none();
        }
    }
    MbValue::none()
}

// ──────────────────────────────────────────────────────────────────────
// Sniffer
// ──────────────────────────────────────────────────────────────────────

fn build_sniffer() -> MbValue {
    make_instance("csv.Sniffer", FxHashMap::default())
}

/// Sniffer.sniff(self, *args) → dialect. Registered variadic, so the
/// generic instance dispatch passes (self, args_list). `args_list` is
/// [sample] or [sample, delimiters] — and a trailing kwargs dict carries
/// `delimiters=`.
pub extern "C" fn sniffer_sniff(slf: MbValue, args_list: MbValue) -> MbValue {
    let _ = slf;
    let items: Vec<MbValue> = args_list
        .as_ptr()
        .map(|p| unsafe {
            match &(*p).data {
                ObjData::List(lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(t) => t.clone(),
                _ => vec![args_list],
            }
        })
        .unwrap_or_else(|| vec![args_list]);

    let kw = trailing_kwargs(&items);
    let positional_end = if kw.is_some() {
        items.len().saturating_sub(1)
    } else {
        items.len()
    };
    let sample = items.first().copied().unwrap_or_else(MbValue::none);
    let mut delim_set: Option<Vec<char>> = None;
    if positional_end >= 2 {
        if let Some(s) = extract_str(items[1]) {
            delim_set = Some(s.chars().collect());
        }
    }
    if let Some(ref m) = kw {
        if let Some(v) = kwarg_get(m, "delimiters") {
            if let Some(s) = extract_str(v) {
                delim_set = Some(s.chars().collect());
            }
        }
    }
    let text = extract_str(sample).unwrap_or_default();
    let (delimiter, quotechar, skipinitialspace, doublequote) =
        sniff_full(&text, delim_set.as_deref());

    let mut fields = FxHashMap::default();
    fields.insert("delimiter".into(), new_str(delimiter.to_string()));
    fields.insert("quotechar".into(), new_str(quotechar.to_string()));
    fields.insert("escapechar".into(), MbValue::none());
    fields.insert("doublequote".into(), MbValue::from_bool(doublequote));
    fields.insert(
        "skipinitialspace".into(),
        MbValue::from_bool(skipinitialspace),
    );
    fields.insert("lineterminator".into(), new_str("\r\n"));
    fields.insert("quoting".into(), MbValue::from_int(QUOTE_MINIMAL));
    fields.insert("strict".into(), MbValue::from_bool(false));
    make_instance("csv.Dialect", fields)
}

/// Full sniff returning (delimiter, quotechar, skipinitialspace, doublequote).
fn sniff_full(sample: &str, restrict: Option<&[char]>) -> (char, char, bool, bool) {
    // Try quote-based detection first (CPython _guess_quote_and_delimiter).
    if let Some((d, q, dq, sis)) = sniff_quoted_full(sample, restrict) {
        return (d, q, sis, dq);
    }
    let (d, q, sis) = sniff_dialect(sample, restrict);
    (d, q, sis, false)
}

/// CPython-style quote+delimiter guess. Returns
/// (delimiter, quotechar, doublequote, skipinitialspace) when a quotechar
/// is found, else None.
fn sniff_quoted_full(sample: &str, restrict: Option<&[char]>) -> Option<(char, char, bool, bool)> {
    let chars: Vec<char> = sample.chars().collect();
    let n = chars.len();
    for &q in &['"', '\''] {
        // Find quoted regions: q ... q.
        let mut regions: Vec<(usize, usize)> = Vec::new(); // (open, close) indices
        let mut i = 0;
        while i < n {
            if chars[i] == q {
                let mut j = i + 1;
                while j < n && chars[j] != q {
                    j += 1;
                }
                if j < n {
                    regions.push((i, j));
                    i = j + 1;
                    continue;
                }
            }
            i += 1;
        }
        if regions.is_empty() {
            continue;
        }

        // Skip regions that look like stray apostrophes inside a word: content
        // spans a newline AND is bounded by alnum on both sides (e.g. the span
        // between the apostrophes of "Harry's"/"Tommy's" across lines).
        let good_region = |open: usize, close: usize| -> bool {
            let inner_newline = chars[open + 1..close].iter().any(|&c| c == '\n');
            let lhs_alnum = open == 0 || chars[open - 1].is_alphanumeric();
            let rhs_alnum = close + 1 >= n || chars[close + 1].is_alphanumeric();
            !(inner_newline && lhs_alnum && rhs_alnum)
        };
        let usable: Vec<(usize, usize)> = regions
            .iter()
            .copied()
            .filter(|&(o, c)| good_region(o, c))
            .collect();
        if usable.is_empty() {
            continue;
        }

        // Tally the char adjacent to each usable quoted region (before open or
        // after close) as the delimiter vote.
        let mut delim_votes: HashMap<char, usize> = HashMap::new();
        let mut space_after = false;
        for &(open, close) in &usable {
            if open > 0 {
                let before = chars[open - 1];
                if is_delim_candidate(before, q) {
                    *delim_votes.entry(before).or_insert(0) += 1;
                }
            }
            if close + 1 < n {
                let after = chars[close + 1];
                if after == ' ' {
                    space_after = true;
                }
                if is_delim_candidate(after, q) {
                    *delim_votes.entry(after).or_insert(0) += 1;
                }
            }
        }
        let regions = usable;

        // Pick the best delimiter, honoring the restrict set.
        let best_vote = delim_votes
            .iter()
            .filter(|(c, _)| restrict.map(|r| r.contains(c)).unwrap_or(true))
            .max_by_key(|(_, &v)| v)
            .map(|(&c, v)| (c, *v));

        let delim = match best_vote {
            Some((c, votes)) if votes >= 1 => c,
            _ => {
                // No adjacent delimiter found. Only accept the quotechar if a
                // restrict set names a delimiter that occurs in the sample;
                // otherwise this isn't real quoting → fall through.
                match restrict.and_then(|r| r.iter().copied().find(|&c| sample.contains(c))) {
                    Some(c) => c,
                    None if regions.len() == 1 => '\n', // single quoted field
                    None => continue,
                }
            }
        };

        // doublequote: a doubled quotechar (qq) appears in the sample.
        let dq_pat: String = std::iter::repeat(q).take(2).collect();
        let doublequote = sample.contains(&dq_pat);

        return Some((delim, q, doublequote, space_after));
    }
    None
}

fn is_delim_candidate(c: char, q: char) -> bool {
    !c.is_alphanumeric() && c != q && c != '\n' && c != '\r' && c != ' '
}

/// Heuristic delimiter/quote detection. Returns (delimiter, quotechar, skipinitialspace).
fn sniff_dialect(sample: &str, restrict: Option<&[char]>) -> (char, char, bool) {
    // First try the quote+delimiter regex approach for quoted samples.
    if let Some((d, q)) = sniff_quoted(sample) {
        if restrict.map(|r| r.contains(&d)).unwrap_or(true) {
            let sis = sample
                .lines()
                .next()
                .map(|l| l.contains(&format!("{d} ")) || l.contains(", "))
                .unwrap_or(false)
                && d == ',';
            return (d, q, sis);
        }
    }

    let preferred = [',', '\t', ';', ' ', ':', '|'];
    let candidates: Vec<char> = match restrict {
        Some(r) => r.to_vec(),
        None => {
            // Collect all non-alphanumeric chars seen, ranked by consistency.
            let mut seen: Vec<char> = Vec::new();
            for ch in sample.chars() {
                if !ch.is_alphanumeric()
                    && ch != '\n'
                    && ch != '\r'
                    && ch != '"'
                    && ch != '\''
                    && !seen.contains(&ch)
                {
                    seen.push(ch);
                }
            }
            seen
        }
    };

    let lines: Vec<&str> = sample.lines().filter(|l| !l.is_empty()).collect();
    let mut best: Option<(char, i64)> = None;
    for &cand in candidates.iter() {
        // Count occurrences per line; reward consistency.
        let counts: Vec<usize> = lines.iter().map(|l| l.matches(cand).count()).collect();
        if counts.iter().all(|&c| c == 0) {
            continue;
        }
        let first = counts.first().copied().unwrap_or(0);
        if first == 0 {
            continue;
        }
        let consistent = counts.iter().all(|&c| c == first);
        let pref_bonus = preferred
            .iter()
            .position(|&p| p == cand)
            .map(|i| (preferred.len() - i) as i64)
            .unwrap_or(0);
        let score = (if consistent { 1000 } else { 0 }) + (first as i64) * 10 + pref_bonus;
        if best.map(|(_, s)| score > s).unwrap_or(true) {
            best = Some((cand, score));
        }
    }
    let delimiter = best.map(|(c, _)| c).unwrap_or(',');
    let skipinitialspace = lines
        .first()
        .map(|l| l.contains(&format!("{delimiter} ")))
        .unwrap_or(false);
    (delimiter, '"', skipinitialspace)
}

/// Detect a quoted-field dialect: find a quotechar and the delimiter adjacent to it.
fn sniff_quoted(sample: &str) -> Option<(char, char)> {
    for &q in &['"', '\''] {
        // pattern: quote ... quote delimiter  OR delimiter quote
        let bytes: Vec<char> = sample.chars().collect();
        let mut delim_after_quote: HashMap<char, usize> = HashMap::new();
        let mut i = 0;
        let n = bytes.len();
        let mut found_quote = false;
        while i < n {
            if bytes[i] == q {
                found_quote = true;
                // find closing quote
                let mut j = i + 1;
                while j < n && bytes[j] != q {
                    j += 1;
                }
                if j < n {
                    // char after closing quote
                    if j + 1 < n {
                        let after = bytes[j + 1];
                        if !after.is_alphanumeric()
                            && after != q
                            && after != '\n'
                            && after != '\r'
                            && after != ' '
                        {
                            *delim_after_quote.entry(after).or_insert(0) += 1;
                        }
                    }
                    // char before opening quote
                    if i > 0 {
                        let before = bytes[i - 1];
                        if !before.is_alphanumeric()
                            && before != q
                            && before != '\n'
                            && before != '\r'
                            && before != ' '
                        {
                            *delim_after_quote.entry(before).or_insert(0) += 1;
                        }
                    }
                    i = j + 1;
                    continue;
                }
            }
            i += 1;
        }
        if found_quote {
            if let Some((&d, _)) = delim_after_quote.iter().max_by_key(|(_, &c)| c) {
                return Some((d, q));
            }
        }
    }
    None
}

/// True iff `s` parses as a Python complex() value (covers int / float /
/// complex literals such as `5+0j`). complex() strips surrounding whitespace.
fn parses_as_complex(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    if t.parse::<f64>().is_ok() {
        return true;
    }
    // Strip an optional surrounding parens pair.
    let body = t
        .strip_prefix('(')
        .and_then(|b| b.strip_suffix(')'))
        .unwrap_or(t);
    // Trailing 'j'/'J' → imaginary; split a+bj.
    let lower = body.to_ascii_lowercase();
    if !lower.contains('j') {
        return false;
    }
    // Find split point between real and imaginary at a +/- that is not an
    // exponent sign and not the leading sign.
    let bytes: Vec<char> = body.chars().collect();
    let mut split = None;
    for i in 1..bytes.len() {
        let c = bytes[i];
        if (c == '+' || c == '-') && bytes[i - 1].to_ascii_lowercase() != 'e' {
            split = Some(i);
        }
    }
    let (real, imag) = match split {
        Some(i) => (&body[..i], &body[i..]),
        None => ("", body),
    };
    // imag must end with j and the rest (minus j) be a float or sign-only.
    let imag_l = imag.to_ascii_lowercase();
    if !imag_l.ends_with('j') {
        return false;
    }
    let imag_num = &imag[..imag.len() - 1];
    let imag_ok = imag_num.is_empty()
        || imag_num == "+"
        || imag_num == "-"
        || imag_num.parse::<f64>().is_ok();
    let real_ok = real.is_empty() || real.parse::<f64>().is_ok();
    imag_ok && real_ok
}

/// Sniffer.has_header(self, sample) → bool. Mirrors CPython's
/// `Sniffer.has_header`: per-column, body cells vote on a consistent type
/// (complex-number vs. string-length); the header votes for "is a header"
/// when it doesn't match.
pub extern "C" fn sniffer_has_header(slf: MbValue, sample: MbValue) -> MbValue {
    let _ = slf;
    let text = extract_str(sample).unwrap_or_default();
    let (delimiter, quotechar, sis, doublequote) = sniff_full(&text, None);
    let fmt = CsvFmt {
        delimiter,
        quotechar: Some(quotechar),
        skipinitialspace: sis,
        doublequote,
        ..CsvFmt::default()
    };
    let records: Vec<String> = split_records(&text, fmt.quotechar)
        .into_iter()
        .filter(|r| !r.is_empty())
        .collect();
    if records.is_empty() {
        return MbValue::from_bool(false);
    }
    let header: Vec<String> = parse_record(&records[0], &fmt)
        .0
        .into_iter()
        .map(|f| f.text)
        .collect();
    let columns = header.len();
    if columns == 0 {
        return MbValue::from_bool(false);
    }

    // Column type: None = unset; Some(None) = complex (numeric); Some(Some(len))
    // = string of that length. A column gets removed once it's inconsistent.
    let mut col_type: Vec<Option<Option<usize>>> = vec![None; columns];
    let mut removed = vec![false; columns];

    let mut checked = 0;
    for rec in records.iter().skip(1) {
        if checked > 20 {
            break;
        }
        checked += 1;
        let fields: Vec<String> = parse_record(rec, &fmt)
            .0
            .into_iter()
            .map(|f| f.text)
            .collect();
        if fields.len() != columns {
            continue;
        }
        for c in 0..columns {
            if removed[c] {
                continue;
            }
            let this_type: Option<usize> = if parses_as_complex(&fields[c]) {
                None // complex/numeric
            } else {
                Some(fields[c].chars().count())
            };
            match col_type[c] {
                None => col_type[c] = Some(this_type),
                Some(existing) => {
                    if existing != this_type {
                        removed[c] = true;
                    }
                }
            }
        }
    }

    let mut has_header: i64 = 0;
    for c in 0..columns {
        if removed[c] {
            continue;
        }
        match col_type[c] {
            Some(Some(len)) => {
                // String column of length `len`.
                if header[c].chars().count() != len {
                    has_header += 1;
                } else {
                    has_header -= 1;
                }
            }
            Some(None) => {
                // Numeric (complex) column.
                if parses_as_complex(&header[c]) {
                    has_header -= 1;
                } else {
                    has_header += 1;
                }
            }
            None => {}
        }
    }
    MbValue::from_bool(has_header > 0)
}

// ──────────────────────────────────────────────────────────────────────
// Module-level dispatchers
// ──────────────────────────────────────────────────────────────────────

unsafe extern "C" fn dispatch_reader(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    if positional_end == 0 {
        return raise_type_error("reader() argument 1 must support iteration");
    }
    let csvfile = a[0];
    if csvfile.is_none() {
        return raise_type_error("argument 1 must be an iterator");
    }
    let dialect = if positional_end >= 2 {
        a[1]
    } else {
        MbValue::none()
    };
    let fmt = match resolve_fmt(dialect, &kw) {
        Some(f) => f,
        None => return MbValue::none(),
    };
    // CPython rejects bytes lines with csv.Error.
    if list_has_bytes(csvfile) {
        return raise_csv_error(
            "iterator should return strings, not bytes (the file should be opened in text mode)",
        );
    }
    build_reader(csvfile, fmt)
}

/// True if `v` is a list/tuple whose first element is a bytes object.
fn list_has_bytes(v: MbValue) -> bool {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => {
                    return lock
                        .read()
                        .unwrap()
                        .iter()
                        .next()
                        .map(|x| is_bytes(*x))
                        .unwrap_or(false);
                }
                ObjData::Tuple(items) => {
                    return items.iter().next().map(|x| is_bytes(*x)).unwrap_or(false);
                }
                _ => {}
            }
        }
    }
    false
}

unsafe extern "C" fn dispatch_writer(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    if positional_end == 0 {
        return raise_type_error("writer() argument 1 must have a \"write\" method");
    }
    let fileobj = a[0];
    if fileobj.is_none() {
        return raise_type_error("argument 1 must have a \"write\" method");
    }
    let dialect = if positional_end >= 2 {
        a[1]
    } else {
        MbValue::none()
    };
    let fmt = match resolve_fmt(dialect, &kw) {
        Some(f) => f,
        None => return MbValue::none(),
    };
    build_writer(fileobj, fmt)
}

unsafe extern "C" fn dispatch_dictreader(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    let f = if positional_end >= 1 {
        a[0]
    } else {
        MbValue::none()
    };
    let mut fieldnames = if positional_end >= 2 {
        a[1]
    } else {
        MbValue::none()
    };
    let mut restkey = if positional_end >= 3 {
        a[2]
    } else {
        MbValue::none()
    };
    let mut restval = if positional_end >= 4 {
        a[3]
    } else {
        MbValue::none()
    };
    let mut dialect = if positional_end >= 5 {
        a[4]
    } else {
        MbValue::none()
    };
    let mut fmt_kw: KwMap = indexmap::IndexMap::new();
    if let Some(ref m) = kw {
        for (k, v) in m.iter() {
            if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                match ks.as_str() {
                    "fieldnames" => fieldnames = *v,
                    "restkey" => restkey = *v,
                    "restval" => restval = *v,
                    "dialect" => dialect = *v,
                    _ => {
                        fmt_kw.insert(k.clone(), *v);
                    }
                }
            }
        }
    }
    let fmt_kw_opt = if fmt_kw.is_empty() {
        None
    } else {
        Some(fmt_kw)
    };
    build_dictreader(f, fieldnames, restkey, restval, dialect, fmt_kw_opt)
}

unsafe extern "C" fn dispatch_dictwriter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    let f = if positional_end >= 1 {
        a[0]
    } else {
        MbValue::none()
    };
    let mut fieldnames = if positional_end >= 2 {
        a[1]
    } else {
        MbValue::none()
    };
    let mut restval = if positional_end >= 3 {
        a[2]
    } else {
        MbValue::none()
    };
    let mut extrasaction = if positional_end >= 4 {
        a[3]
    } else {
        MbValue::none()
    };
    let mut dialect = if positional_end >= 5 {
        a[4]
    } else {
        MbValue::none()
    };
    let mut fmt_kw: KwMap = indexmap::IndexMap::new();
    if let Some(ref m) = kw {
        for (k, v) in m.iter() {
            if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                match ks.as_str() {
                    "fieldnames" => fieldnames = *v,
                    "restval" => restval = *v,
                    "extrasaction" => extrasaction = *v,
                    "dialect" => dialect = *v,
                    _ => {
                        fmt_kw.insert(k.clone(), *v);
                    }
                }
            }
        }
    }
    let fmt_kw_opt = if fmt_kw.is_empty() {
        None
    } else {
        Some(fmt_kw)
    };
    build_dictwriter(f, fieldnames, restval, extrasaction, dialect, fmt_kw_opt)
}

unsafe extern "C" fn dispatch_sniffer_new(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    build_sniffer()
}

unsafe extern "C" fn dispatch_register_dialect(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    if positional_end == 0 {
        return raise_type_error("register_dialect() takes at least 1 argument (0 given)");
    }
    let name = a[0];
    if !is_str(name) {
        return raise_type_error("dialect name must be a string");
    }
    if positional_end > 2 {
        return raise_type_error("register_dialect() takes at most 2 positional arguments");
    }
    let dialect = if positional_end >= 2 {
        a[1]
    } else {
        MbValue::none()
    };
    mb_csv_register_dialect_impl(name, dialect, &kw)
}

unsafe extern "C" fn dispatch_unregister_dialect(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if a.is_empty() {
        return raise_type_error("unregister_dialect() takes exactly 1 argument (0 given)");
    }
    let name = a[0];
    let n = match extract_str(name) {
        Some(s) => s,
        None => return raise_csv_error(&format!("unknown dialect: {:?}", "")),
    };
    let existed = DIALECTS.with(|d| d.borrow_mut().remove(&n).is_some());
    if !existed {
        return raise_csv_error(&format!("unknown dialect: {n:?}"));
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_get_dialect(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if a.is_empty() {
        return raise_type_error("get_dialect() takes exactly 1 argument (0 given)");
    }
    let name = a[0];
    let n = match extract_str(name) {
        Some(s) => s,
        None => {
            return raise_csv_error(&format!(
                "unknown dialect: {}",
                if name.is_none() {
                    "None".to_string()
                } else {
                    String::new()
                }
            ))
        }
    };
    match DIALECTS.with(|d| d.borrow().get(&n).copied()) {
        Some(v) => v,
        None => raise_csv_error(&format!("unknown dialect: {n:?}")),
    }
}

unsafe extern "C" fn dispatch_list_dialects(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if !a.is_empty() && trailing_kwargs(a).is_none() {
        return raise_type_error("list_dialects() takes no arguments");
    }
    let names: Vec<MbValue> =
        DIALECTS.with(|d| d.borrow().keys().map(|k| new_str(k.clone())).collect());
    MbValue::from_ptr(MbObject::new_list(names))
}

unsafe extern "C" fn dispatch_field_size_limit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    if positional_end > 1 {
        return raise_type_error("field_size_limit() takes at most 1 argument");
    }
    let prev = FIELD_SIZE_LIMIT.with(|f| *f.borrow());
    if positional_end == 1 {
        let new_limit = a[0];
        // CPython requires an integer (None / non-int → TypeError).
        match new_limit.as_int() {
            Some(n) if !new_limit.is_none() => {
                FIELD_SIZE_LIMIT.with(|f| *f.borrow_mut() = n);
            }
            _ => return raise_type_error("limit must be an integer"),
        }
    }
    MbValue::from_int(prev)
}

fn mb_csv_register_dialect_impl(name: MbValue, dialect: MbValue, kw: &Option<KwMap>) -> MbValue {
    let n = extract_str(name).unwrap_or_default();
    if n.is_empty() {
        return raise_type_error("dialect name must be a string");
    }
    // Resolve from dialect + kwargs into a frozen dialect instance.
    let fmt = match resolve_fmt(dialect, kw) {
        Some(f) => f,
        None => return MbValue::none(),
    };
    let val = fmt_to_dialect(&fmt);
    DIALECTS.with(|d| {
        d.borrow_mut().insert(n, val);
    });
    MbValue::none()
}

// ──────────────────────────────────────────────────────────────────────
// Dialect class construction (instances carry frozen fields).
// ──────────────────────────────────────────────────────────────────────

/// csv.Dialect.__init__(self): validate the (class-attribute) format
/// parameters, raising csv.Error with CPython messages. Runs when a user
/// `class X(csv.Dialect)` is instantiated.
pub extern "C" fn dialect_init(slf: MbValue) -> MbValue {
    // 1-character string params. `optional` params may be None and use the
    // "string or None" wording; `delimiter` is required and uses "string".
    let check_1char = |name: &str, required: bool| -> bool {
        let or_none = if required { "" } else { " or None" };
        let v = getattr_opt(slf, name);
        match v {
            None => {
                if required {
                    raise_csv_error(&format!("\"{name}\" must be string{or_none}, not NoneType"));
                    return false;
                }
                true
            }
            Some(val) => {
                if val.is_none() {
                    if required {
                        raise_csv_error(&format!(
                            "\"{name}\" must be string{or_none}, not NoneType"
                        ));
                        return false;
                    }
                    return true;
                }
                if is_bytes(val) {
                    raise_csv_error(&format!("\"{name}\" must be string{or_none}, not bytes"));
                    return false;
                }
                if !is_str(val) {
                    raise_csv_error(&format!(
                        "\"{name}\" must be string{or_none}, not {}",
                        type_name_of(val)
                    ));
                    return false;
                }
                let s = extract_str(val).unwrap_or_default();
                if s.chars().count() != 1 {
                    raise_csv_error(&format!("\"{name}\" must be a 1-character string"));
                    return false;
                }
                true
            }
        }
    };

    if !check_1char("delimiter", true) {
        return MbValue::none();
    }
    // quotechar may be None only when quoting is QUOTE_NONE. When `quoting`
    // resolves to a concrete int that isn't QUOTE_NONE and quotechar is None,
    // the dialect is incomplete. (If `quoting` can't be resolved — a known
    // class-body limitation for `quoting = csv.QUOTE_NONE` — stay lenient so
    // valid QUOTE_NONE dialects aren't falsely rejected.)
    let quoting = getattr_opt(slf, "quoting").and_then(|v| v.as_int());
    let qc = getattr_opt(slf, "quotechar");
    let qc_is_none = qc.map(|v| v.is_none()).unwrap_or(true);
    if qc_is_none {
        if matches!(quoting, Some(q) if q != QUOTE_NONE) {
            raise_csv_error("quotechar must be set if quoting enabled");
            return MbValue::none();
        }
    } else if !check_1char("quotechar", false) {
        return MbValue::none();
    }
    if !check_1char("escapechar", false) {
        return MbValue::none();
    }

    // lineterminator must be a string.
    if let Some(v) = getattr_opt(slf, "lineterminator") {
        if !v.is_none() && !is_str(v) {
            raise_csv_error("\"lineterminator\" must be a string");
            return MbValue::none();
        }
    }

    // quoting must be a valid int constant when present.
    if let Some(v) = getattr_opt(slf, "quoting") {
        if !v.is_none() {
            match v.as_int() {
                Some(q) if (0..=5).contains(&q) => {}
                _ => {
                    raise_csv_error("bad \"quoting\" value");
                    return MbValue::none();
                }
            }
        }
    }

    MbValue::none()
}

fn build_dialect_instance(
    name: &str,
    delimiter: &str,
    quotechar: &str,
    quoting: i64,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &str,
    strict: bool,
) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("delimiter".into(), new_str(delimiter));
    fields.insert("quotechar".into(), new_str(quotechar));
    fields.insert("escapechar".into(), MbValue::none());
    fields.insert("doublequote".into(), MbValue::from_bool(doublequote));
    fields.insert(
        "skipinitialspace".into(),
        MbValue::from_bool(skipinitialspace),
    );
    fields.insert("lineterminator".into(), new_str(lineterminator));
    fields.insert("quoting".into(), MbValue::from_int(quoting));
    fields.insert("strict".into(), MbValue::from_bool(strict));
    make_instance(&format!("csv.{name}"), fields)
}

/// Register a dialect as a real class with class attributes, so user
/// subclasses inherit its fields. Returns the class-name string value.
fn register_dialect_class(
    name: &str,
    base: &str,
    delimiter: &str,
    quotechar: &str,
    quoting: i64,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &str,
    strict: bool,
) {
    let bases = if base.is_empty() {
        vec![]
    } else {
        vec![base.to_string()]
    };
    // Attach a validating __init__ to the root Dialect class; subclasses
    // inherit it so `class X(csv.Dialect)` validates on instantiation.
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    if base.is_empty() {
        methods.insert(
            "__init__".to_string(),
            MbValue::from_func(dialect_init as *const () as usize),
        );
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(dialect_init as *const () as u64);
        });
    }
    super::super::class::mb_class_register(name, bases, methods);
    let set = |attr: &str, v: MbValue| {
        super::super::class::mb_class_set_class_attr(new_str(name), new_str(attr), v);
    };
    // The root Dialect leaves delimiter/quotechar unset (None) so a subclass
    // that supplies only some attributes fails validation (CPython parity);
    // concrete dialects (excel, …) override with real values.
    set(
        "delimiter",
        if delimiter.is_empty() {
            MbValue::none()
        } else {
            new_str(delimiter)
        },
    );
    set(
        "quotechar",
        if quotechar.is_empty() {
            MbValue::none()
        } else {
            new_str(quotechar)
        },
    );
    set("escapechar", MbValue::none());
    set("doublequote", MbValue::from_bool(doublequote));
    set("skipinitialspace", MbValue::from_bool(skipinitialspace));
    set("lineterminator", new_str(lineterminator));
    set("quoting", MbValue::from_int(quoting));
    set("strict", MbValue::from_bool(strict));
}

// ──────────────────────────────────────────────────────────────────────
// Back-compat symbol shims (referenced by symbols.rs rt_sym! table).
// The live module dispatch goes through the flat-args dispatchers above;
// these 2-arg entries keep the named JIT symbols resolvable.
// ──────────────────────────────────────────────────────────────────────

pub fn mb_csv_reader(csvfile: MbValue, dialect: MbValue) -> MbValue {
    let fmt = match resolve_fmt(dialect, &None) {
        Some(f) => f,
        None => return MbValue::none(),
    };
    build_reader(csvfile, fmt)
}

pub fn mb_csv_writer(fileobj: MbValue, dialect: MbValue) -> MbValue {
    let fmt = match resolve_fmt(dialect, &None) {
        Some(f) => f,
        None => return MbValue::none(),
    };
    build_writer(fileobj, fmt)
}

pub fn mb_csv_dictreader(f: MbValue, fieldnames: MbValue) -> MbValue {
    build_dictreader(
        f,
        fieldnames,
        MbValue::none(),
        MbValue::none(),
        MbValue::none(),
        None,
    )
}

pub fn mb_csv_dictwriter(f: MbValue, fieldnames: MbValue) -> MbValue {
    build_dictwriter(
        f,
        fieldnames,
        MbValue::none(),
        MbValue::none(),
        MbValue::none(),
        None,
    )
}

// ──────────────────────────────────────────────────────────────────────
// Module registration
// ──────────────────────────────────────────────────────────────────────

pub fn register() {
    use super::super::module::{NATIVE_FUNC_ADDRS, NATIVE_TYPE_NAMES};

    fn add_dispatch(attrs: &mut HashMap<String, MbValue>, name: &str, addr: usize) {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Register the reader / writer / Dict* / Sniffer classes carrying their
    // native methods. Method dispatch flows through class.rs's generic
    // instance path (SystemV ABI: self is the first MbValue arg).
    fn reg_class(name: &str, methods: &[(&str, *const ())]) {
        let mut map: HashMap<String, MbValue> = HashMap::new();
        for (m, addr) in methods {
            map.insert(m.to_string(), MbValue::from_func(*addr as usize));
        }
        super::super::class::mb_class_register(name, vec![], map);
    }

    reg_class(
        "_csv.reader",
        &[
            ("__iter__", reader_iter as *const ()),
            ("__next__", reader_next as *const ()),
        ],
    );
    reg_class(
        "_csv.writer",
        &[
            ("writerow", writer_writerow as *const ()),
            ("writerows", writer_writerows as *const ()),
        ],
    );
    reg_class(
        "csv.DictReader",
        &[
            ("__iter__", dictreader_iter as *const ()),
            ("__next__", dictreader_next as *const ()),
        ],
    );
    reg_class(
        "csv.DictWriter",
        &[
            ("writeheader", dictwriter_writeheader as *const ()),
            ("writerow", dictwriter_writerow as *const ()),
            ("writerows", dictwriter_writerows as *const ()),
        ],
    );
    reg_class(
        "csv.Sniffer",
        &[
            ("has_header", sniffer_has_header as *const ()),
            ("sniff", sniffer_sniff as *const ()),
        ],
    );
    // sniff(self, *args): variadic so the generic instance dispatch packs
    // positional args (+ trailing kwargs dict) into a single list arg.
    super::super::module::register_variadic_func(sniffer_sniff as *const () as u64);

    let mut attrs = HashMap::new();

    let dispatchers: &[(&str, usize)] = &[
        ("reader", dispatch_reader as *const () as usize),
        ("writer", dispatch_writer as *const () as usize),
        ("DictReader", dispatch_dictreader as *const () as usize),
        ("DictWriter", dispatch_dictwriter as *const () as usize),
        ("Sniffer", dispatch_sniffer_new as *const () as usize),
        (
            "register_dialect",
            dispatch_register_dialect as *const () as usize,
        ),
        (
            "unregister_dialect",
            dispatch_unregister_dialect as *const () as usize,
        ),
        ("get_dialect", dispatch_get_dialect as *const () as usize),
        (
            "list_dialects",
            dispatch_list_dialects as *const () as usize,
        ),
        (
            "field_size_limit",
            dispatch_field_size_limit as *const () as usize,
        ),
    ];
    for (n, a) in dispatchers {
        add_dispatch(&mut attrs, n, *a);
    }

    // Bind constructor types so isinstance() / type() see them.
    NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(
            dispatch_sniffer_new as *const () as u64,
            "csv.Sniffer".into(),
        );
    });

    // Quoting constants.
    attrs.insert("QUOTE_MINIMAL".into(), MbValue::from_int(QUOTE_MINIMAL));
    attrs.insert("QUOTE_ALL".into(), MbValue::from_int(QUOTE_ALL));
    attrs.insert(
        "QUOTE_NONNUMERIC".into(),
        MbValue::from_int(QUOTE_NONNUMERIC),
    );
    attrs.insert("QUOTE_NONE".into(), MbValue::from_int(QUOTE_NONE));
    attrs.insert("QUOTE_STRINGS".into(), MbValue::from_int(QUOTE_STRINGS));
    attrs.insert("QUOTE_NOTNULL".into(), MbValue::from_int(QUOTE_NOTNULL));

    // Dialect classes (subclassable; subclasses inherit fields via MRO).
    // Root Dialect: delimiter/quotechar unset so incomplete subclasses fail.
    register_dialect_class(
        "Dialect",
        "",
        "",
        "",
        QUOTE_MINIMAL,
        true,
        false,
        "\r\n",
        false,
    );
    register_dialect_class(
        "excel",
        "Dialect",
        ",",
        "\"",
        QUOTE_MINIMAL,
        true,
        false,
        "\r\n",
        false,
    );
    register_dialect_class(
        "excel-tab",
        "excel",
        "\t",
        "\"",
        QUOTE_MINIMAL,
        true,
        false,
        "\r\n",
        false,
    );
    register_dialect_class(
        "unix_dialect",
        "Dialect",
        ",",
        "\"",
        QUOTE_ALL,
        true,
        false,
        "\n",
        false,
    );

    // csv.Error as a subclassable Exception.
    super::super::class::mb_class_register(
        "csv.Error",
        vec!["Exception".to_string()],
        HashMap::new(),
    );

    // Built-in dialect instances in the registry.
    let excel = build_dialect_instance(
        "excel",
        ",",
        "\"",
        QUOTE_MINIMAL,
        true,
        false,
        "\r\n",
        false,
    );
    let excel_tab = build_dialect_instance(
        "excel-tab",
        "\t",
        "\"",
        QUOTE_MINIMAL,
        true,
        false,
        "\r\n",
        false,
    );
    let unix = build_dialect_instance("unix", ",", "\"", QUOTE_ALL, true, false, "\n", false);
    DIALECTS.with(|d| {
        let mut map = d.borrow_mut();
        map.insert("excel".to_string(), excel);
        map.insert("excel-tab".to_string(), excel_tab);
        map.insert("unix".to_string(), unix);
    });

    // Module-level dialect class references (callable / subclassable names).
    attrs.insert("excel".into(), new_str("excel"));
    attrs.insert("excel_tab".into(), new_str("excel-tab"));
    attrs.insert("unix_dialect".into(), new_str("unix_dialect"));
    attrs.insert("Dialect".into(), new_str("Dialect"));
    attrs.insert("Error".into(), new_str("csv.Error"));

    // Module dunders (CPython exposes csv.__version__ == "1.0" and a __doc__).
    attrs.insert("__version__".into(), new_str("1.0"));
    attrs.insert("__doc__".into(), new_str("CSV parsing and writing.\n"));

    super::register_module("csv", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_parse_simple() {
        let fmt = CsvFmt::default();
        let f = parse_record("a,b,c", &fmt).0;
        assert_eq!(
            f.iter().map(|x| x.text.clone()).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn test_parse_quoted_comma() {
        let fmt = CsvFmt::default();
        let f = parse_record("\"a,b\",c", &fmt).0;
        assert_eq!(
            f.iter().map(|x| x.text.clone()).collect::<Vec<_>>(),
            vec!["a,b", "c"]
        );
    }

    #[test]
    fn test_parse_doubled_quote() {
        let fmt = CsvFmt::default();
        let f = parse_record("\"a\"\"b\",c", &fmt).0;
        assert_eq!(
            f.iter().map(|x| x.text.clone()).collect::<Vec<_>>(),
            vec!["a\"b", "c"]
        );
    }

    #[test]
    fn test_format_minimal_quotes_comma() {
        let fmt = CsvFmt::default();
        let out = format_row(&[s("a,b"), s("c")], &fmt).unwrap();
        assert_eq!(out, "\"a,b\",c\r\n");
    }
}
