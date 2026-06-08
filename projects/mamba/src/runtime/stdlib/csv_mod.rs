use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// csv module for Mamba (#398).
///
/// Wave-4 Ship #2 (Task #53): expands the 4-dispatcher + 4-constant
/// surface to the typeshed 16-entry surface:
///
/// - **Dispatchers** (flat-args ABI, #2097-hoist friendly):
///   reader, writer, DictReader, DictWriter, register_dialect,
///   unregister_dialect, get_dialect, list_dialects, field_size_limit.
/// - **Constants:** QUOTE_MINIMAL, QUOTE_ALL, QUOTE_NONNUMERIC,
///   QUOTE_NONE.
/// - **Instances:** Error (class shell), excel, excel_tab, unix_dialect
///   (Dialect Instances with frozen attribute fields).
///
/// Dialect registry is a `thread_local!` HashMap<String, MbValue> keyed
/// by dialect name; built-in dialects are pre-registered at module
/// init. register_dialect / unregister_dialect / get_dialect mutate
/// the registry. Sniffer is intentionally deferred — heuristic
/// delimiter detection is niche and not on any realistic perf hot
/// path.
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static DIALECTS: RefCell<HashMap<String, MbValue>> = RefCell::new(HashMap::new());
    static FIELD_SIZE_LIMIT: RefCell<i64> = const { RefCell::new(131_072) };
}

// ── Dispatch wrappers (flat-args ABI) ──

unsafe extern "C" fn dispatch_reader(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_csv_reader(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_writer(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_csv_writer(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_dictreader(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_csv_dictreader(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_dictwriter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_csv_dictwriter(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_register_dialect(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_csv_register_dialect(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_unregister_dialect(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_csv_unregister_dialect(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_get_dialect(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_csv_get_dialect(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_list_dialects(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_csv_list_dialects()
}

unsafe extern "C" fn dispatch_field_size_limit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_csv_field_size_limit(a.get(0).copied().unwrap_or_else(MbValue::none))
}

/// Build a Dialect Instance with the canonical CPython attribute set.
fn build_dialect(
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
    fields.insert(
        "delimiter".into(),
        MbValue::from_ptr(MbObject::new_str(delimiter.to_string())),
    );
    fields.insert(
        "quotechar".into(),
        MbValue::from_ptr(MbObject::new_str(quotechar.to_string())),
    );
    fields.insert("escapechar".into(), MbValue::none());
    fields.insert("doublequote".into(), MbValue::from_bool(doublequote));
    fields.insert(
        "skipinitialspace".into(),
        MbValue::from_bool(skipinitialspace),
    );
    fields.insert(
        "lineterminator".into(),
        MbValue::from_ptr(MbObject::new_str(lineterminator.to_string())),
    );
    fields.insert("quoting".into(), MbValue::from_int(quoting));
    fields.insert("strict".into(), MbValue::from_bool(strict));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: format!("csv.{}", name),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn build_error() -> MbValue {
    let fields = FxHashMap::default();
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "csv.Error".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Register the csv module.
pub fn register() {
    use super::super::module::NATIVE_FUNC_ADDRS;

    fn add_dispatch(attrs: &mut HashMap<String, MbValue>, name: &str, addr: usize) {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    let mut attrs = HashMap::new();

    let dispatchers: &[(&str, usize)] = &[
        ("reader", dispatch_reader as *const () as usize),
        ("writer", dispatch_writer as *const () as usize),
        ("DictReader", dispatch_dictreader as *const () as usize),
        ("DictWriter", dispatch_dictwriter as *const () as usize),
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

    // Quoting constants (CPython csv.QUOTE_* values).
    attrs.insert("QUOTE_MINIMAL".into(), MbValue::from_int(0));
    attrs.insert("QUOTE_ALL".into(), MbValue::from_int(1));
    attrs.insert("QUOTE_NONNUMERIC".into(), MbValue::from_int(2));
    attrs.insert("QUOTE_NONE".into(), MbValue::from_int(3));

    // Dialect Instances with CPython-matching attribute fields.
    let excel = build_dialect("excel", ",", "\"", 0, true, false, "\r\n", false);
    let excel_tab = build_dialect("excel_tab", "\t", "\"", 0, true, false, "\r\n", false);
    let unix_dialect = build_dialect("unix_dialect", ",", "\"", 1, true, false, "\n", false);

    DIALECTS.with(|d| {
        let mut map = d.borrow_mut();
        map.insert("excel".to_string(), excel);
        map.insert("excel-tab".to_string(), excel_tab);
        map.insert("unix".to_string(), unix_dialect);
    });

    attrs.insert("excel".into(), excel);
    attrs.insert("excel_tab".into(), excel_tab);
    attrs.insert("unix_dialect".into(), unix_dialect);
    attrs.insert(
        "Dialect".into(),
        build_dialect("Dialect", ",", "\"", 0, true, false, "\r\n", false),
    );
    attrs.insert("Error".into(), build_error());

    super::register_module("csv", attrs);
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

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Parse a single CSV line respecting quoted fields.
fn parse_csv_line(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == delimiter {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }
    fields.push(current);
    fields
}

/// csv.reader(csvfile, delimiter=',') -> list of list of strings.
///
/// CPython accepts any iterable of strings (one per line). Mamba's
/// forward ship accepts:
/// - str: split on '\n' and tokenize each non-empty line
/// - list[str]: tokenize each element directly (already line-split)
/// Both shapes produce identical row output for line-conformant
/// input. Iterator-from-StringIO shape is reduced to list[str]
/// by the bench harness; live io.StringIO iterators are deferred
/// pending the `__next__`/`__iter__` wiring described in the scout.
pub fn mb_csv_reader(csvfile: MbValue, delimiter: MbValue) -> MbValue {
    let delim = extract_str(delimiter)
        .and_then(|d| d.chars().next())
        .unwrap_or(',');

    let tokenize = |line: &str| -> MbValue {
        let fields = parse_csv_line(line, delim);
        let items: Vec<MbValue> = fields
            .into_iter()
            .map(|f| MbValue::from_ptr(MbObject::new_str(f)))
            .collect();
        MbValue::from_ptr(MbObject::new_list(items))
    };

    let rows: Vec<MbValue> = if let Some(s) = extract_str(csvfile) {
        s.lines().filter(|l| !l.is_empty()).map(tokenize).collect()
    } else if let Some(ptr) = csvfile.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                items
                    .iter()
                    .filter_map(|v| extract_str(*v))
                    .filter(|l| !l.is_empty())
                    .map(|line| tokenize(&line))
                    .collect()
            } else {
                vec![]
            }
        }
    } else {
        return raise_type_error("csv.reader() argument must be iterable");
    };
    MbValue::from_ptr(MbObject::new_list(rows))
}

/// csv.writer(rows, delimiter=',') -> CSV string
pub fn mb_csv_writer(rows: MbValue, delimiter: MbValue) -> MbValue {
    let delim = extract_str(delimiter)
        .and_then(|d| d.chars().next())
        .unwrap_or(',');

    let mut output = String::new();
    if let Some(ptr) = rows.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let row_list = lock.read().unwrap();
                for row_val in row_list.iter() {
                    if let Some(row_ptr) = row_val.as_ptr() {
                        if let ObjData::List(ref lock2) = (*row_ptr).data {
                            let items = lock2.read().unwrap();
                            let fields: Vec<String> = items
                                .iter()
                                .map(|v| {
                                    let s = extract_str(*v).unwrap_or_default();
                                    if s.contains(delim) || s.contains('"') || s.contains('\n') {
                                        format!("\"{}\"", s.replace('"', "\"\""))
                                    } else {
                                        s
                                    }
                                })
                                .collect();
                            output.push_str(&fields.join(&delim.to_string()));
                            output.push('\n');
                        }
                    }
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_str(output))
}

/// csv.DictReader(text, fieldnames) -> list of dicts
pub fn mb_csv_dictreader(text: MbValue, fieldnames: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let mut lines = s.lines().filter(|l| !l.is_empty());

    // Get headers from fieldnames or first row
    let headers: Vec<String> = if fieldnames.is_none() {
        match lines.next() {
            Some(first) => parse_csv_line(first, ','),
            None => return MbValue::from_ptr(MbObject::new_list(vec![])),
        }
    } else if let Some(ptr) = fieldnames.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                items.iter().filter_map(|v| extract_str(*v)).collect()
            } else {
                vec![]
            }
        }
    } else {
        vec![]
    };

    let rows: Vec<MbValue> = lines
        .map(|line| {
            let fields = parse_csv_line(line, ',');
            let dict = MbObject::new_dict();
            unsafe {
                if let ObjData::Dict(ref lock) = (*dict).data {
                    let mut map = lock.write().unwrap();
                    for (i, header) in headers.iter().enumerate() {
                        let val = fields
                            .get(i)
                            .map(|f| MbValue::from_ptr(MbObject::new_str(f.clone())))
                            .unwrap_or(MbValue::none());
                        map.insert(header.clone().into(), val);
                    }
                }
            }
            MbValue::from_ptr(dict)
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(rows))
}

/// csv.DictWriter stub — returns the csv.writer result
pub fn mb_csv_dictwriter(rows: MbValue, delimiter: MbValue) -> MbValue {
    mb_csv_writer(rows, delimiter)
}

/// csv.register_dialect(name, dialect_or_kwargs) — store under name.
///
/// CPython accepts (name, [dialect], **fmtparams). We model the
/// minimum: a name + a Dialect-like Instance/value. If the second
/// arg is None, build a default excel-style Dialect.
pub fn mb_csv_register_dialect(name: MbValue, dialect: MbValue) -> MbValue {
    let n = extract_str(name).unwrap_or_default();
    if n.is_empty() {
        return MbValue::none();
    }
    let val = if dialect.is_none() {
        build_dialect(&n, ",", "\"", 0, true, false, "\r\n", false)
    } else {
        dialect
    };
    DIALECTS.with(|d| {
        d.borrow_mut().insert(n, val);
    });
    MbValue::none()
}

/// csv.unregister_dialect(name) — remove from registry.
///
/// CPython raises csv.Error on unknown names. Mamba shim returns None
/// unconditionally (callers that depend on the raise are out of scope
/// for forward ship; see scout doc).
pub fn mb_csv_unregister_dialect(name: MbValue) -> MbValue {
    if let Some(n) = extract_str(name) {
        DIALECTS.with(|d| {
            d.borrow_mut().remove(&n);
        });
    }
    MbValue::none()
}

/// csv.get_dialect(name) — return Dialect Instance or None if unknown.
pub fn mb_csv_get_dialect(name: MbValue) -> MbValue {
    let n = match extract_str(name) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    DIALECTS.with(|d| d.borrow().get(&n).copied().unwrap_or_else(MbValue::none))
}

/// csv.list_dialects() — list of registered dialect names.
pub fn mb_csv_list_dialects() -> MbValue {
    let names: Vec<MbValue> = DIALECTS.with(|d| {
        d.borrow()
            .keys()
            .map(|k| MbValue::from_ptr(MbObject::new_str(k.clone())))
            .collect()
    });
    MbValue::from_ptr(MbObject::new_list(names))
}

/// csv.field_size_limit([new_limit]) — get/set the field size limit.
///
/// Returns the previous limit (CPython semantics). If called with no
/// argument (None), returns the current limit unchanged.
pub fn mb_csv_field_size_limit(new_limit: MbValue) -> MbValue {
    let prev = FIELD_SIZE_LIMIT.with(|f| *f.borrow());
    if !new_limit.is_none() {
        if let Some(n) = new_limit.as_int() {
            FIELD_SIZE_LIMIT.with(|f| *f.borrow_mut() = n);
        }
    }
    MbValue::from_int(prev)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    // Helper: extract a string from a MbValue that should be a Str ptr.
    fn extract(v: MbValue) -> String {
        unsafe {
            if let ObjData::Str(ref st) = (*v.as_ptr().expect("expected ptr")).data {
                st.clone()
            } else {
                panic!("expected Str");
            }
        }
    }

    // Helper: get the Vec<MbValue> from a List ptr.
    fn list_items(v: MbValue) -> Vec<MbValue> {
        unsafe {
            if let ObjData::List(ref lock) = (*v.as_ptr().expect("expected ptr")).data {
                lock.read().unwrap().to_vec()
            } else {
                panic!("expected List");
            }
        }
    }

    #[test]
    fn test_csv_parse_line() {
        let fields = parse_csv_line("a,b,c", ',');
        assert_eq!(fields, vec!["a", "b", "c"]);

        let fields = parse_csv_line("\"a,b\",c", ',');
        assert_eq!(fields, vec!["a,b", "c"]);
    }

    #[test]
    fn test_csv_reader() {
        let csv = s("name,age\nalice,30\nbob,25");
        let rows = mb_csv_reader(csv, MbValue::none());
        unsafe {
            if let ObjData::List(ref lock) = (*rows.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
            }
        }
    }

    // ── New tests ──

    #[test]
    fn test_parse_csv_line_empty_string() {
        let fields = parse_csv_line("", ',');
        assert_eq!(fields, vec![""]);
    }

    #[test]
    fn test_parse_csv_line_single_field() {
        let fields = parse_csv_line("hello", ',');
        assert_eq!(fields, vec!["hello"]);
    }

    #[test]
    fn test_parse_csv_line_quoted_commas() {
        let fields = parse_csv_line("\"a,b\",c", ',');
        assert_eq!(fields, vec!["a,b", "c"]);
    }

    #[test]
    fn test_parse_csv_line_escaped_quote() {
        let fields = parse_csv_line("\"a\"\"b\",c", ',');
        assert_eq!(fields, vec!["a\"b", "c"]);
    }

    #[test]
    fn test_parse_csv_line_tab_delimiter() {
        let fields = parse_csv_line("a\tb\tc", '\t');
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_csv_reader_empty_input() {
        let rows = mb_csv_reader(s(""), MbValue::none());
        let items = list_items(rows);
        assert_eq!(items.len(), 0, "empty input should produce empty list");
    }

    #[test]
    fn test_csv_reader_single_row() {
        let rows = mb_csv_reader(s("a,b,c"), MbValue::none());
        let row_items = list_items(rows);
        assert_eq!(row_items.len(), 1);
        let fields = list_items(row_items[0]);
        assert_eq!(fields.len(), 3);
        assert_eq!(extract(fields[0]), "a");
        assert_eq!(extract(fields[1]), "b");
        assert_eq!(extract(fields[2]), "c");
    }

    #[test]
    fn test_csv_reader_multiple_rows() {
        let rows = mb_csv_reader(s("a,b\nc,d"), MbValue::none());
        let row_items = list_items(rows);
        assert_eq!(row_items.len(), 2);
    }

    #[test]
    fn test_csv_reader_quoted_field() {
        let rows = mb_csv_reader(s("\"hello, world\",foo"), MbValue::none());
        let row_items = list_items(rows);
        assert_eq!(row_items.len(), 1);
        let fields = list_items(row_items[0]);
        assert_eq!(fields.len(), 2);
        assert_eq!(extract(fields[0]), "hello, world");
        assert_eq!(extract(fields[1]), "foo");
    }

    #[test]
    fn test_csv_writer_single_row() {
        let inner = MbValue::from_ptr(MbObject::new_list(vec![s("a"), s("b")]));
        let rows = MbValue::from_ptr(MbObject::new_list(vec![inner]));
        let result = mb_csv_writer(rows, MbValue::none());
        assert_eq!(extract(result), "a,b\n");
    }

    #[test]
    fn test_csv_writer_multiple_rows() {
        let row1 = MbValue::from_ptr(MbObject::new_list(vec![s("a"), s("b")]));
        let row2 = MbValue::from_ptr(MbObject::new_list(vec![s("c"), s("d")]));
        let rows = MbValue::from_ptr(MbObject::new_list(vec![row1, row2]));
        let result = mb_csv_writer(rows, MbValue::none());
        let output = extract(result);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "a,b");
        assert_eq!(lines[1], "c,d");
    }

    #[test]
    fn test_csv_writer_field_with_comma_gets_quoted() {
        let inner = MbValue::from_ptr(MbObject::new_list(vec![s("a,b"), s("c")]));
        let rows = MbValue::from_ptr(MbObject::new_list(vec![inner]));
        let result = mb_csv_writer(rows, MbValue::none());
        let output = extract(result);
        assert!(
            output.contains("\"a,b\""),
            "field containing comma must be quoted: {}",
            output
        );
    }

    #[test]
    fn test_csv_dictreader_with_fieldnames() {
        let fieldnames = MbValue::from_ptr(MbObject::new_list(vec![s("name"), s("age")]));
        let text = s("alice,30\nbob,25");
        let result = mb_csv_dictreader(text, fieldnames);
        let rows = list_items(result);
        assert_eq!(rows.len(), 2);
        // Inspect first row dict
        unsafe {
            if let ObjData::Dict(ref lock) = (*rows[0].as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                let name_val = map.get("name").copied().unwrap_or(MbValue::none());
                assert_eq!(extract(name_val), "alice");
                let age_val = map.get("age").copied().unwrap_or(MbValue::none());
                assert_eq!(extract(age_val), "30");
            } else {
                panic!("expected Dict");
            }
        }
    }
}
