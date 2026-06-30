use super::super::rc::MbObject;
use super::super::value::MbValue;
/// getopt module for Mamba.
///
/// Implements Python 3.12 `getopt` stdlib: C-style command line option parsing.
/// Provides `getopt()` and `gnu_getopt()` matching CPython 3.12 signatures,
/// plus raises `GetoptError` on invalid options or missing arguments.
///
/// Short option syntax: "ho:v"
///   - 'h', 'v'  — flags with no argument
///   - 'o:'      — option requiring an argument
///
/// Long option syntax: ["help", "output=", "verbose"]
///   - "help", "verbose"  — flags with no argument
///   - "output="          — option requiring an argument
use std::collections::HashMap;

// ── Helpers ──

/// Extract a Rust String from an MbValue that holds a heap string object.
/// Returns None if the value is not a string.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Extract a Vec<String> from an MbValue that holds a List of string objects.
/// Non-string elements are skipped.
fn extract_list_of_strings(val: MbValue) -> Vec<String> {
    val.as_ptr()
        .and_then(|ptr| unsafe {
            use super::super::rc::ObjData;
            if let ObjData::List(ref rw) = (*ptr).data {
                let guard = rw.read().ok()?;
                let results: Vec<String> = guard.iter().filter_map(|v| extract_str(*v)).collect();
                Some(results)
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn extract_sequence_of_strings(val: MbValue) -> Result<Vec<String>, MbValue> {
    let Some(ptr) = val.as_ptr() else {
        return Err(raise_type_error(
            "getopt() argument 1 must be a sequence of strings",
        ));
    };
    unsafe {
        use super::super::rc::ObjData;
        match &(*ptr).data {
            ObjData::List(rw) => rw
                .read()
                .map(|guard| {
                    guard
                        .iter()
                        .map(|v| {
                            extract_str(*v).ok_or_else(|| {
                                raise_type_error(
                                    "getopt() argument 1 must be a sequence of strings",
                                )
                            })
                        })
                        .collect()
                })
                .unwrap_or_else(|_| {
                    Err(raise_type_error(
                        "getopt() argument 1 must be a sequence of strings",
                    ))
                }),
            ObjData::Tuple(items) => items
                .iter()
                .map(|v| {
                    extract_str(*v).ok_or_else(|| {
                        raise_type_error("getopt() argument 1 must be a sequence of strings")
                    })
                })
                .collect(),
            _ => Err(raise_type_error(
                "getopt() argument 1 must be a sequence of strings",
            )),
        }
    }
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise a GetoptError with the given message and return MbValue::none().
fn raise_getopt_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("GetoptError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Parse the shortopts string into a set of (char, requires_arg) entries.
/// 'o:' => ('o', true); 'v' => ('v', false)
fn parse_shortopts(shortopts: &str) -> Vec<(char, bool)> {
    let chars: Vec<char> = shortopts.chars().collect();
    let mut result = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '+' || c == '-' {
            // GNU extensions for ordering control — ignore for our purposes
            i += 1;
            continue;
        }
        let requires_arg = i + 1 < chars.len() && chars[i + 1] == ':';
        result.push((c, requires_arg));
        if requires_arg {
            i += 2;
        } else {
            i += 1;
        }
    }
    result
}

/// Parse the longopts list into a vec of (name_without_eq, requires_arg).
/// "output=" => ("output", true); "help" => ("help", false)
fn parse_longopts(longopts: &[String]) -> Vec<(String, bool)> {
    longopts
        .iter()
        .map(|s| {
            if s.ends_with('=') {
                (s[..s.len() - 1].to_string(), true)
            } else {
                (s.to_string(), false)
            }
        })
        .collect()
}

/// Core parsing logic shared by getopt and gnu_getopt.
///
/// `gnu` — if true, permute non-option args (GNU style); if false, stop at
///          first non-option arg (POSIX getopt style).
///
/// Returns Ok((opts_pairs, remaining)) on success, Err(msg) on error.
fn parse_opts(
    args: &[String],
    shortopts_table: &[(char, bool)],
    longopts_table: &[(String, bool)],
    gnu: bool,
) -> Result<(Vec<(String, String)>, Vec<String>), String> {
    let mut opts: Vec<(String, String)> = Vec::new();
    let mut remaining: Vec<String> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg == "--" {
            // End of options — everything after goes to remaining.
            remaining.extend_from_slice(&args[i + 1..]);
            break;
        }

        if arg.starts_with("--") {
            // Long option.
            let body = &arg[2..];
            // May be --name=value or --name
            let (name, inline_val) = if let Some(eq_pos) = body.find('=') {
                (&body[..eq_pos], Some(body[eq_pos + 1..].to_string()))
            } else {
                (body, None)
            };

            // Find matching long option.
            let matched = longopts_table.iter().find(|(n, _)| n == name);
            match matched {
                None => {
                    return Err(format!("option --{} not recognized", name));
                }
                Some((_, requires_arg)) => {
                    if *requires_arg {
                        let val = if let Some(v) = inline_val {
                            v
                        } else if i + 1 < args.len() {
                            i += 1;
                            args[i].clone()
                        } else {
                            return Err(format!("option --{} requires argument", name));
                        };
                        opts.push((format!("--{}", name), val));
                    } else {
                        if inline_val.is_some() {
                            return Err(format!("option --{} must not have an argument", name));
                        }
                        opts.push((format!("--{}", name), String::new()));
                    }
                }
            }
            i += 1;
        } else if arg.starts_with('-') && arg.len() > 1 {
            // Short option(s) — may be clustered like "-vho".
            let body: Vec<char> = arg[1..].chars().collect();
            let mut j = 0;
            while j < body.len() {
                let c = body[j];
                let matched = shortopts_table.iter().find(|(sc, _)| *sc == c);
                match matched {
                    None => {
                        return Err(format!("option -{} not recognized", c));
                    }
                    Some((_, requires_arg)) => {
                        if *requires_arg {
                            // Argument is either the rest of this token or the next token.
                            let val = if j + 1 < body.len() {
                                // Rest of current token is the argument.
                                let rest: String = body[j + 1..].iter().collect();
                                rest
                            } else if i + 1 < args.len() {
                                i += 1;
                                args[i].clone()
                            } else {
                                return Err(format!("option -{} requires argument", c));
                            };
                            opts.push((format!("-{}", c), val));
                            j = body.len();
                        } else {
                            opts.push((format!("-{}", c), String::new()));
                            j += 1;
                        }
                    }
                }
            }
            i += 1;
        } else {
            // Non-option argument.
            if gnu {
                // GNU mode: collect and continue processing.
                remaining.push(arg.clone());
                i += 1;
            } else {
                // POSIX mode: stop processing, treat rest as remaining.
                remaining.extend_from_slice(&args[i..]);
                break;
            }
        }
    }

    Ok((opts, remaining))
}

// ── Module registration ──

macro_rules! dispatch_ternary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
                a.get(2).copied().unwrap_or_else(|| {
                    // longopts default = []
                    MbValue::from_ptr(MbObject::new_list(vec![]))
                }),
            )
        }
    };
}

dispatch_ternary!(dispatch_getopt, mb_getopt_getopt);
dispatch_ternary!(dispatch_gnu_getopt, mb_getopt_gnu_getopt);

/// Register the getopt module in the stdlib registry.
/// `GetoptError(msg, opt='')` constructor. Models GetoptError as a real
/// exception *class* (the proven re.error pattern) instead of a sentinel Str:
/// a callable whose addr resolves to "GetoptError" via NATIVE_TYPE_NAMES, so
/// `except getopt.GetoptError` / `isinstance` keep matching the raised instance
/// (raise_getopt_error tags it with the same class name), while the registered
/// chaining slots make `hasattr(getopt.GetoptError, "__cause__")` True.
unsafe extern "C" fn dispatch_getopt_error(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    use super::super::rc::{MbObjectHeader, MbRwLock, ObjData, ObjKind};
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let msg = a.first().copied().unwrap_or_else(MbValue::none);
    let opt = a.get(1).copied().unwrap_or_else(MbValue::none);
    let mut fields = rustc_hash::FxHashMap::default();
    fields.insert("msg".to_string(), msg);
    fields.insert("message".to_string(), msg);
    fields.insert("opt".to_string(), opt);
    fields.insert(
        "args".to_string(),
        MbValue::from_ptr(MbObject::new_tuple(vec![msg])),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "GetoptError".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getopt", dispatch_getopt as usize),
        ("gnu_getopt", dispatch_gnu_getopt as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // GetoptError + its `getopt.error` alias: a real exception class (re.error
    // pattern). Both names share one constructor whose addr resolves to
    // "GetoptError" via NATIVE_TYPE_NAMES, so `except`/`isinstance` match the
    // raised instance; mb_class_register seeds the chaining slots so
    // `hasattr(getopt.GetoptError, "__cause__")` is True.
    let err_addr = dispatch_getopt_error as *const () as usize;
    attrs.insert("GetoptError".to_string(), MbValue::from_func(err_addr));
    attrs.insert("error".to_string(), MbValue::from_func(err_addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(err_addr as u64);
    });
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut()
            .insert(err_addr as u64, "GetoptError".to_string());
    });
    {
        let mut slots: HashMap<String, MbValue> = HashMap::new();
        let slot = MbValue::from_func(err_addr);
        slots.insert("__cause__".to_string(), slot);
        slots.insert("__context__".to_string(), slot);
        slots.insert("__suppress_context__".to_string(), slot);
        super::super::class::mb_class_register("GetoptError", vec!["Exception".to_string()], slots);
    }
    super::register_module("getopt", attrs);
}

// ── Public runtime functions ──

/// getopt.getopt(args, shortopts, longopts=[]) -> (opts, args)
///
/// Parse command line options POSIX-style: stops at first non-option argument.
/// Returns a tuple (opts_list, remaining_args) where opts_list is a list of
/// (option, value) tuples. Raises GetoptError on unknown options or missing args.
pub fn mb_getopt_getopt(args: MbValue, shortopts: MbValue, longopts: MbValue) -> MbValue {
    let args_vec = match extract_sequence_of_strings(args) {
        Ok(args) => args,
        Err(err) => return err,
    };
    let shortopts_str = extract_str(shortopts).unwrap_or_default();
    let longopts_vec = extract_list_of_strings(longopts);

    let shortopts_table = parse_shortopts(&shortopts_str);
    let longopts_table = parse_longopts(&longopts_vec);

    match parse_opts(&args_vec, &shortopts_table, &longopts_table, false) {
        Err(msg) => raise_getopt_error(&msg),
        Ok((pairs, remaining)) => build_result(pairs, remaining),
    }
}

/// getopt.gnu_getopt(args, shortopts, longopts=[]) -> (opts, args)
///
/// Parse command line options GNU-style: permutes non-option arguments so
/// processing continues past them. Otherwise identical to getopt().
pub fn mb_getopt_gnu_getopt(args: MbValue, shortopts: MbValue, longopts: MbValue) -> MbValue {
    let args_vec = match extract_sequence_of_strings(args) {
        Ok(args) => args,
        Err(err) => return err,
    };
    let shortopts_str = extract_str(shortopts).unwrap_or_default();
    let longopts_vec = extract_list_of_strings(longopts);

    let shortopts_table = parse_shortopts(&shortopts_str);
    let longopts_table = parse_longopts(&longopts_vec);

    match parse_opts(&args_vec, &shortopts_table, &longopts_table, true) {
        Err(msg) => raise_getopt_error(&msg),
        Ok((pairs, remaining)) => build_result(pairs, remaining),
    }
}

/// Build the (opts_list, remaining) tuple result from parsed pairs.
fn build_result(pairs: Vec<(String, String)>, remaining: Vec<String>) -> MbValue {
    let opts_list: Vec<MbValue> = pairs
        .into_iter()
        .map(|(k, v)| {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str(k)),
                MbValue::from_ptr(MbObject::new_str(v)),
            ]))
        })
        .collect();

    let remaining_list: Vec<MbValue> = remaining
        .into_iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str(s)))
        .collect();

    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_list(opts_list)),
        MbValue::from_ptr(MbObject::new_list(remaining_list)),
    ]))
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::super::super::rc::ObjData;
    use super::*;

    fn make_str(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    fn make_list(items: Vec<&str>) -> MbValue {
        let vals: Vec<MbValue> = items.iter().map(|s| make_str(s)).collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    /// Unwrap the outer tuple (opts_list, remaining) from a getopt result.
    /// Returns (opts_count, remaining_count) for simple length checks.
    fn unwrap_result_counts(result: MbValue) -> (usize, usize) {
        let ptr = result.as_ptr().expect("result must be non-null");
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 2, "outer tuple must have 2 elements");
                let opts_count = items[0]
                    .as_ptr()
                    .map(|p| {
                        if let ObjData::List(ref rw) = (*p).data {
                            rw.read().map(|g| g.len()).unwrap_or(0)
                        } else {
                            0
                        }
                    })
                    .unwrap_or(0);
                let rem_count = items[1]
                    .as_ptr()
                    .map(|p| {
                        if let ObjData::List(ref rw) = (*p).data {
                            rw.read().map(|g| g.len()).unwrap_or(0)
                        } else {
                            0
                        }
                    })
                    .unwrap_or(0);
                (opts_count, rem_count)
            } else {
                panic!("expected outer Tuple");
            }
        }
    }

    /// Get the string value of opts[index].key from a getopt result.
    fn get_opt_key(result: MbValue, index: usize) -> String {
        let ptr = result.as_ptr().unwrap();
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let opts_ptr = items[0].as_ptr().unwrap();
                if let ObjData::List(ref rw) = (*opts_ptr).data {
                    let guard = rw.read().unwrap();
                    let pair_ptr = guard[index].as_ptr().unwrap();
                    if let ObjData::Tuple(ref pair) = (*pair_ptr).data {
                        return extract_str(pair[0]).unwrap_or_default();
                    }
                }
            }
            panic!("could not extract opt key at index {}", index);
        }
    }

    /// Get the string value of opts[index].value from a getopt result.
    fn get_opt_val(result: MbValue, index: usize) -> String {
        let ptr = result.as_ptr().unwrap();
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let opts_ptr = items[0].as_ptr().unwrap();
                if let ObjData::List(ref rw) = (*opts_ptr).data {
                    let guard = rw.read().unwrap();
                    let pair_ptr = guard[index].as_ptr().unwrap();
                    if let ObjData::Tuple(ref pair) = (*pair_ptr).data {
                        return extract_str(pair[1]).unwrap_or_default();
                    }
                }
            }
            panic!("could not extract opt val at index {}", index);
        }
    }

    /// Get remaining[index] string from a getopt result.
    fn get_remaining(result: MbValue, index: usize) -> String {
        let ptr = result.as_ptr().unwrap();
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let rem_ptr = items[1].as_ptr().unwrap();
                if let ObjData::List(ref rw) = (*rem_ptr).data {
                    let guard = rw.read().unwrap();
                    return extract_str(guard[index]).unwrap_or_default();
                }
            }
            panic!("could not extract remaining at index {}", index);
        }
    }

    // REQ: R2
    #[test]
    fn test_getopt_short_no_arg() {
        // parse ["-v"] with shortopts "v" => opts=[("-v","")], remaining=[]
        let args = make_list(vec!["-v"]);
        let shortopts = make_str("v");
        let longopts = make_list(vec![]);
        let result = mb_getopt_getopt(args, shortopts, longopts);
        let (opts_count, rem_count) = unwrap_result_counts(result);
        assert_eq!(opts_count, 1, "expected 1 option parsed");
        assert_eq!(rem_count, 0, "expected 0 remaining args");
        // re-run to check key/value (result was moved)
        let args2 = make_list(vec!["-v"]);
        let shortopts2 = make_str("v");
        let longopts2 = make_list(vec![]);
        let result2 = mb_getopt_getopt(args2, shortopts2, longopts2);
        assert_eq!(get_opt_key(result2, 0), "-v");
        let result3 = mb_getopt_getopt(make_list(vec!["-v"]), make_str("v"), make_list(vec![]));
        assert_eq!(get_opt_val(result3, 0), "");
    }

    // REQ: R2
    #[test]
    fn test_getopt_short_with_arg() {
        // parse ["-o", "foo"] with shortopts "o:" => opts=[("-o","foo")], remaining=[]
        let args = make_list(vec!["-o", "foo"]);
        let shortopts = make_str("o:");
        let longopts = make_list(vec![]);
        let result = mb_getopt_getopt(args, shortopts, longopts);
        let (opts_count, rem_count) = unwrap_result_counts(result);
        assert_eq!(opts_count, 1, "expected 1 option");
        assert_eq!(rem_count, 0, "expected 0 remaining");

        let result2 = mb_getopt_getopt(
            make_list(vec!["-o", "foo"]),
            make_str("o:"),
            make_list(vec![]),
        );
        assert_eq!(get_opt_key(result2, 0), "-o");
        let result3 = mb_getopt_getopt(
            make_list(vec!["-o", "foo"]),
            make_str("o:"),
            make_list(vec![]),
        );
        assert_eq!(get_opt_val(result3, 0), "foo");
    }

    // REQ: R2
    #[test]
    fn test_getopt_long() {
        // parse ["--help"] with longopts ["help"] => opts=[("--help","")], remaining=[]
        let args = make_list(vec!["--help"]);
        let shortopts = make_str("");
        let longopts = make_list(vec!["help"]);
        let result = mb_getopt_getopt(args, shortopts, longopts);
        let (opts_count, rem_count) = unwrap_result_counts(result);
        assert_eq!(opts_count, 1, "expected 1 option");
        assert_eq!(rem_count, 0, "expected 0 remaining");

        let result2 = mb_getopt_getopt(
            make_list(vec!["--help"]),
            make_str(""),
            make_list(vec!["help"]),
        );
        assert_eq!(get_opt_key(result2, 0), "--help");
    }

    // REQ: R2
    #[test]
    fn test_getopt_long_with_arg() {
        // parse ["--output=foo"] with longopts ["output="] => opts=[("--output","foo")], remaining=[]
        let args = make_list(vec!["--output=foo"]);
        let shortopts = make_str("");
        let longopts = make_list(vec!["output="]);
        let result = mb_getopt_getopt(args, shortopts, longopts);
        let (opts_count, rem_count) = unwrap_result_counts(result);
        assert_eq!(opts_count, 1, "expected 1 option");
        assert_eq!(rem_count, 0, "expected 0 remaining");

        let result2 = mb_getopt_getopt(
            make_list(vec!["--output=foo"]),
            make_str(""),
            make_list(vec!["output="]),
        );
        assert_eq!(get_opt_key(result2, 0), "--output");
        let result3 = mb_getopt_getopt(
            make_list(vec!["--output=foo"]),
            make_str(""),
            make_list(vec!["output="]),
        );
        assert_eq!(get_opt_val(result3, 0), "foo");
    }

    // REQ: R2
    #[test]
    fn test_getopt_stops_at_nonoption() {
        // parse ["-v", "foo", "-h"] with shortopts "vh"
        // POSIX: stops at "foo", opts=[("-v","")], remaining=["foo","-h"]
        let args = make_list(vec!["-v", "foo", "-h"]);
        let shortopts = make_str("vh");
        let longopts = make_list(vec![]);
        let result = mb_getopt_getopt(args, shortopts, longopts);
        let (opts_count, rem_count) = unwrap_result_counts(result);
        assert_eq!(
            opts_count, 1,
            "POSIX getopt: only -v before non-option 'foo'"
        );
        assert_eq!(
            rem_count, 2,
            "POSIX getopt: remaining must contain 'foo' and '-h'"
        );

        let result2 = mb_getopt_getopt(
            make_list(vec!["-v", "foo", "-h"]),
            make_str("vh"),
            make_list(vec![]),
        );
        assert_eq!(get_remaining(result2, 0), "foo");
        let result3 = mb_getopt_getopt(
            make_list(vec!["-v", "foo", "-h"]),
            make_str("vh"),
            make_list(vec![]),
        );
        assert_eq!(get_remaining(result3, 1), "-h");
    }

    // REQ: R2
    #[test]
    fn test_gnu_getopt_permutes() {
        // parse ["-v", "foo", "-h"] with shortopts "vh"
        // GNU: permutes — processes -v and -h, remaining=["foo"]
        let args = make_list(vec!["-v", "foo", "-h"]);
        let shortopts = make_str("vh");
        let longopts = make_list(vec![]);
        let result = mb_getopt_gnu_getopt(args, shortopts, longopts);
        let (opts_count, rem_count) = unwrap_result_counts(result);
        assert_eq!(opts_count, 2, "GNU getopt: both -v and -h must be parsed");
        assert_eq!(rem_count, 1, "GNU getopt: only 'foo' remains");

        let result2 = mb_getopt_gnu_getopt(
            make_list(vec!["-v", "foo", "-h"]),
            make_str("vh"),
            make_list(vec![]),
        );
        assert_eq!(get_remaining(result2, 0), "foo");
    }

    // REQ: R2
    #[test]
    fn test_getopt_double_dash_terminates() {
        // parse ["-v", "--", "-h"] with shortopts "vh"
        // "--" terminates option parsing; "-h" goes into remaining
        let args = make_list(vec!["-v", "--", "-h"]);
        let shortopts = make_str("vh");
        let longopts = make_list(vec![]);
        let result = mb_getopt_getopt(args, shortopts, longopts);
        let (opts_count, rem_count) = unwrap_result_counts(result);
        assert_eq!(opts_count, 1, "only -v parsed before --");
        assert_eq!(rem_count, 1, "-h after -- goes into remaining");

        let result2 = mb_getopt_getopt(
            make_list(vec!["-v", "--", "-h"]),
            make_str("vh"),
            make_list(vec![]),
        );
        assert_eq!(get_remaining(result2, 0), "-h");
    }
}
