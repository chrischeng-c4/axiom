use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// string module for Mamba (#452).
///
/// Provides string constants (ascii_lowercase, ascii_uppercase, ascii_letters,
/// digits, hexdigits, octdigits, punctuation, whitespace, printable),
/// `string.capwords`, a real subclassable `string.Formatter`, and
/// `string.Template`.
///
/// `Formatter` and `Template` are registered as real classes (CLASS_REGISTRY)
/// so `isinstance`, subclassing, and dynamic method dispatch all work. The
/// Formatter engine threads `self` through `mb_call_method` for every
/// overridable hook (`get_value`, `convert_field`, `format_field`,
/// `check_unused_args`, `parse`, `get_field`) so a subclass override is
/// honored exactly like CPython.
use std::collections::HashMap;

// ── small helpers ─────────────────────────────────────────────────────────

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn new_list(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}

fn str_of(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Str(ref s) = (*p).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn is_str_value(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
}

fn is_none_value(v: MbValue) -> bool {
    v.is_none()
}

fn raise(kind: &str, msg: impl Into<String>) -> MbValue {
    super::super::exception::mb_raise(new_str(kind), new_str(msg.into()));
    MbValue::none()
}

fn raise_value_error(msg: &str) {
    raise("ValueError", msg.to_string());
}

fn has_exc() -> bool {
    super::super::exception::mb_has_exception().as_bool() == Some(true)
}

fn instance_class_name(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*p).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

/// True when `v` is an instance of `Formatter` (or a subclass).
fn is_formatter_instance(v: MbValue) -> bool {
    match instance_class_name(v) {
        Some(cn) => {
            cn == "Formatter" || super::super::class::class_mro_any(&cn, |c| c == "Formatter")
        }
        None => false,
    }
}

/// Convert a value to its `str()` form as a Rust String.
fn pystr(v: MbValue) -> String {
    str_of(super::super::builtins::mb_str(v)).unwrap_or_default()
}

/// Call `obj.method(args...)` through the runtime so subclass overrides and
/// generators dispatch correctly. Returns the result (None if an exception is
/// pending).
fn call_method(obj: MbValue, name: &str, args: Vec<MbValue>) -> MbValue {
    super::super::class::mb_call_method(obj, new_str(name), new_list(args))
}

// ── module registration ───────────────────────────────────────────────────

unsafe extern "C" fn dispatch_capwords(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let s = a.first().copied().unwrap_or_else(MbValue::none);
    let sep = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_string_capwords(s, sep)
}

unsafe extern "C" fn dispatch_formatter(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    make_instance("Formatter", vec![])
}

unsafe extern "C" fn dispatch_template(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let template = a.first().copied().unwrap_or_else(MbValue::none);
    make_instance("Template", vec![("template", template)])
}

fn make_instance(class_name: &str, fields: Vec<(&str, MbValue)>) -> MbValue {
    let inst = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let ObjData::Instance {
            fields: ref iflds, ..
        } = (*inst).data
        {
            let mut g = iflds.write().unwrap();
            for (k, v) in fields {
                super::super::rc::retain_if_ptr(v);
                g.insert(k.to_string(), v);
            }
        }
    }
    MbValue::from_ptr(inst)
}

/// Register the string module.
pub fn register() {
    let mut attrs = HashMap::new();

    attrs.insert(
        "ascii_lowercase".to_string(),
        new_str("abcdefghijklmnopqrstuvwxyz"),
    );
    attrs.insert(
        "ascii_uppercase".to_string(),
        new_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    );
    attrs.insert(
        "ascii_letters".to_string(),
        new_str("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    );
    attrs.insert("digits".to_string(), new_str("0123456789"));
    attrs.insert("hexdigits".to_string(), new_str("0123456789abcdefABCDEF"));
    attrs.insert("octdigits".to_string(), new_str("01234567"));
    attrs.insert(
        "punctuation".to_string(),
        new_str("!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"),
    );
    attrs.insert("whitespace".to_string(), new_str(" \t\n\r\x0b\x0c"));
    // printable = digits + ascii_letters + punctuation + whitespace
    attrs.insert(
        "printable".to_string(),
        new_str(
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\
         !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~ \t\n\r\x0b\x0c",
        ),
    );

    // ── Register native classes ──
    register_formatter_class();
    register_template_class();

    // ── Constructor / function dispatchers ──
    let ctor_dispatchers: Vec<(&str, usize, &str)> = vec![
        (
            "Formatter",
            dispatch_formatter as *const () as usize,
            "Formatter",
        ),
        (
            "Template",
            dispatch_template as *const () as usize,
            "Template",
        ),
    ];
    for (name, addr, type_name) in &ctor_dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(*addr as u64, type_name.to_string());
        });
    }

    let func_dispatchers: Vec<(&str, usize)> =
        vec![("capwords", dispatch_capwords as *const () as usize)];
    for (name, addr) in &func_dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }

    super::register_module("string", attrs);
}

fn register_formatter_class() {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    let methods: Vec<(&str, *const ())> = vec![
        ("format", m_formatter_format as *const ()),
        ("vformat", m_formatter_vformat as *const ()),
        ("_vformat", m_formatter_vformat_impl as *const ()),
        ("parse", m_formatter_parse as *const ()),
        ("get_field", m_formatter_get_field as *const ()),
        ("get_value", m_formatter_get_value as *const ()),
        (
            "check_unused_args",
            m_formatter_check_unused_args as *const (),
        ),
        ("convert_field", m_formatter_convert_field as *const ()),
        ("format_field", m_formatter_format_field as *const ()),
    ];
    for (name, addr) in methods {
        let v = MbValue::from_func(addr as usize);
        // `format` and `vformat` accept (self, format_string, *args, **kwargs);
        // register variadic so the instance-dispatch path packs args into a list.
        map.insert(name.to_string(), v);
    }
    // format(self, format_string, /, *args, **kwargs)
    super::super::module::register_variadic_func(m_formatter_format as *const () as u64);
    super::super::module::register_kwargs_func(m_formatter_format as *const () as u64);
    // vformat / _vformat receive a single packed positional list (the caller
    // assembles the exact argument vector). Register variadic so the
    // instance-dispatch path packs all positionals into one list.
    super::super::module::register_variadic_func(m_formatter_vformat as *const () as u64);
    super::super::module::register_variadic_func(m_formatter_vformat_impl as *const () as u64);
    super::super::class::mb_class_register("Formatter", vec!["object".to_string()], map);
}

fn register_template_class() {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    let methods: Vec<(&str, *const ())> = vec![
        ("__init__", m_template_init as *const ()),
        ("substitute", m_template_substitute as *const ()),
        ("safe_substitute", m_template_safe_substitute as *const ()),
        ("get_identifiers", m_template_get_identifiers as *const ()),
        ("is_valid", m_template_is_valid as *const ()),
    ];
    for (name, addr) in methods {
        map.insert(name.to_string(), MbValue::from_func(addr as usize));
    }
    // substitute / safe_substitute accept (self, mapping=*, **kwargs)
    super::super::module::register_variadic_func(m_template_substitute as *const () as u64);
    super::super::module::register_kwargs_func(m_template_substitute as *const () as u64);
    super::super::module::register_variadic_func(m_template_safe_substitute as *const () as u64);
    super::super::module::register_kwargs_func(m_template_safe_substitute as *const () as u64);
    super::super::class::mb_class_register("Template", vec!["object".to_string()], map);
    // Default class attributes (overridable by subclasses). braceidpattern is
    // None by default (falls back to idpattern); flags=2 == re.IGNORECASE.
    let cls = new_str("Template");
    super::super::class::mb_class_set_class_attr(cls, new_str("delimiter"), new_str("$"));
    super::super::class::mb_class_set_class_attr(
        cls,
        new_str("idpattern"),
        new_str("(?a:[_a-z][_a-z0-9]*)"),
    );
    super::super::class::mb_class_set_class_attr(cls, new_str("flags"), MbValue::from_int(2));
}

// ╔══════════════════════════════════════════════════════════════════════╗
// ║ string.Formatter                                                       ║
// ╚══════════════════════════════════════════════════════════════════════╝

/// A parsed `{...}` markup field: literal prefix, field name, format spec,
/// conversion, all owned. `field_name == None` means a trailing literal only.
struct ParsedField {
    literal: String,
    field_name: Option<String>,
    format_spec: Option<String>,
    conversion: Option<char>,
}

/// CPython `str._formatter_parser`: parse a format string into a sequence of
/// (literal_text, field_name, format_spec, conversion) tuples. Returns Err with
/// a ValueError message on malformed markup.
fn formatter_parse(s: &str) -> Result<Vec<ParsedField>, String> {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;
    let mut literal = String::new();
    while i < n {
        let c = chars[i];
        if c == '{' || c == '}' {
            if i + 1 < n && chars[i + 1] == c {
                literal.push(c);
                i += 2;
                continue;
            }
            if c == '}' {
                return Err("Single '}' encountered in format string".to_string());
            }
            // c == '{' — start of a replacement field.
            i += 1;
            // Read field name + conversion + spec, balancing nested braces in spec.
            let mut field_name = String::new();
            // field name part: up to '!' (conversion) or ':' (spec) or '}'
            let mut conversion: Option<char> = None;
            let mut format_spec: Option<String> = None;
            // Read field name (no nested braces here in CPython up to ! or :)
            while i < n && chars[i] != '}' && chars[i] != '!' && chars[i] != ':' {
                field_name.push(chars[i]);
                i += 1;
            }
            if i < n && chars[i] == '!' {
                i += 1;
                if i >= n || chars[i] == '}' || chars[i] == ':' {
                    return Err("end of string while looking for conversion specifier".to_string());
                }
                conversion = Some(chars[i]);
                i += 1;
            }
            if i < n && chars[i] == ':' {
                i += 1;
                // Spec runs to the matching '}', honoring nested {} one level.
                let mut spec = String::new();
                let mut depth = 1u32;
                while i < n {
                    let cc = chars[i];
                    if cc == '{' {
                        depth += 1;
                        spec.push(cc);
                        i += 1;
                        continue;
                    }
                    if cc == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        spec.push(cc);
                        i += 1;
                        continue;
                    }
                    spec.push(cc);
                    i += 1;
                }
                format_spec = Some(spec);
            }
            if i >= n || chars[i] != '}' {
                return Err("expected '}' before end of string".to_string());
            }
            i += 1; // consume '}'
            out.push(ParsedField {
                literal: std::mem::take(&mut literal),
                field_name: Some(field_name),
                format_spec: Some(format_spec.unwrap_or_default()),
                conversion,
            });
        } else {
            literal.push(c);
            i += 1;
        }
    }
    if !literal.is_empty() {
        out.push(ParsedField {
            literal,
            field_name: None,
            format_spec: None,
            conversion: None,
        });
    }
    Ok(out)
}

/// CPython `str._formatter_field_name_split`: split a field name into the first
/// (the arg key) and an iterator of (is_attr, value) accessors. Returns
/// (first_key, [(is_attr, accessor_string)]).
fn field_name_split(field_name: &str) -> (String, Vec<(bool, String)>) {
    let chars: Vec<char> = field_name.chars().collect();
    let n = chars.len();
    let mut i = 0;
    // first: up to '.' or '['
    let mut first = String::new();
    while i < n && chars[i] != '.' && chars[i] != '[' {
        first.push(chars[i]);
        i += 1;
    }
    let mut rest = Vec::new();
    while i < n {
        if chars[i] == '.' {
            i += 1;
            let mut name = String::new();
            while i < n && chars[i] != '.' && chars[i] != '[' {
                name.push(chars[i]);
                i += 1;
            }
            rest.push((true, name));
        } else if chars[i] == '[' {
            i += 1;
            let mut idx = String::new();
            while i < n && chars[i] != ']' {
                idx.push(chars[i]);
                i += 1;
            }
            if i < n {
                i += 1;
            } // consume ']'
            rest.push((false, idx));
        } else {
            break;
        }
    }
    (first, rest)
}

// Formatter.format(self, format_string, /, *args, **kwargs)
//   variadic+kwargs → (self, args_list, kwargs_dict)
extern "C" fn m_formatter_format(this: MbValue, args_list: MbValue, kwargs: MbValue) -> MbValue {
    // string.Formatter and logging.Formatter both register a class named
    // "Formatter" in CLASS_REGISTRY, so a logging.Formatter's `.format(record)`
    // resolves to this string-engine method. A logging formatter carries a
    // `_style` field — route it to the logging engine instead of treating the
    // record as a template string.
    if super::logging_mod::value_is_logging_formatter(this) {
        let items = super::super::builtins::extract_items(args_list);
        let record = items.first().copied().unwrap_or_else(MbValue::none);
        return super::logging_mod::logging_formatter_format(this, record);
    }
    let items = super::super::builtins::extract_items(args_list);
    let format_string = items.first().copied().unwrap_or_else(MbValue::none);
    if format_string.is_none() && items.is_empty() {
        return raise(
            "TypeError",
            "descriptor 'format' of 'string.Formatter' object needs an argument",
        );
    }
    let args: Vec<MbValue> = items.iter().skip(1).copied().collect();
    formatter_vformat_entry(this, format_string, args, kwargs)
}

// Formatter.vformat(self, format_string, args, kwargs)
//   variadic-only → (self, args_list) where args_list = [format_string, args, kwargs]
extern "C" fn m_formatter_vformat(this: MbValue, args_list: MbValue) -> MbValue {
    let items = super::super::builtins::extract_items(args_list);
    let format_string = items.first().copied().unwrap_or_else(MbValue::none);
    let args = items.get(1).copied().unwrap_or_else(|| new_list(vec![]));
    let kwargs = items.get(2).copied().unwrap_or_else(MbValue::none);
    let pos = super::super::builtins::extract_items(args);
    formatter_vformat_entry(this, format_string, pos, kwargs)
}

/// Shared entry for format/vformat: runs `_vformat`, then check_unused_args.
fn formatter_vformat_entry(
    this: MbValue,
    format_string: MbValue,
    args: Vec<MbValue>,
    kwargs: MbValue,
) -> MbValue {
    let kwargs = if kwargs.is_none() {
        MbValue::from_ptr(MbObject::new_dict())
    } else {
        kwargs
    };
    let args_v = new_list(args);
    let used_args = make_set();
    // _vformat(format_string, args, kwargs, used_args, recursion_depth=2)
    // returns (result_string, auto_arg_index)
    let pair = call_method(
        this,
        "_vformat",
        vec![
            format_string,
            args_v,
            kwargs,
            used_args,
            MbValue::from_int(2),
        ],
    );
    if has_exc() {
        return MbValue::none();
    }
    let parts = super::super::builtins::extract_items(pair);
    let result = parts.first().copied().unwrap_or_else(|| new_str(""));
    // check_unused_args(used_args, args, kwargs)
    call_method(this, "check_unused_args", vec![used_args, args_v, kwargs]);
    if has_exc() {
        return MbValue::none();
    }
    result
}

// Formatter._vformat(self, format_string, args, kwargs, used_args, recursion_depth, auto_arg_index=0)
//   variadic → (self, args_list); returns a (result_string, auto_arg_index) tuple.
extern "C" fn m_formatter_vformat_impl(this: MbValue, args_list: MbValue) -> MbValue {
    let a = super::super::builtins::extract_items(args_list);
    let format_string = a.first().copied().unwrap_or_else(MbValue::none);
    let args = a.get(1).copied().unwrap_or_else(|| new_list(vec![]));
    let kwargs = a
        .get(2)
        .copied()
        .unwrap_or_else(|| MbValue::from_ptr(MbObject::new_dict()));
    let used_args = a.get(3).copied().unwrap_or_else(make_set);
    let recursion_depth = a.get(4).and_then(|v| v.as_int()).unwrap_or(2);
    let mut auto_arg_index: i64 = a.get(5).and_then(|v| v.as_int()).unwrap_or(0);

    if recursion_depth < 0 {
        return raise("ValueError", "Max string recursion exceeded");
    }
    let fmt = match str_of(format_string) {
        Some(s) => s,
        None => {
            return MbValue::from_ptr(MbObject::new_tuple(vec![
                new_str(""),
                MbValue::from_int(auto_arg_index),
            ]))
        }
    };

    // parse via self.parse so overrides apply.
    let parsed = call_method(this, "parse", vec![new_str(fmt.clone())]);
    if has_exc() {
        return MbValue::none();
    }
    let parsed_list = super::super::iter::mb_list_from_iter(super::super::iter::mb_iter(parsed));
    if has_exc() {
        return MbValue::none();
    }
    let fields = super::super::builtins::extract_items(parsed_list);

    let mut result = String::new();
    for field in fields {
        // Each field is a 4-tuple (literal_text, field_name, format_spec, conversion)
        let parts = super::super::builtins::extract_items(field);
        let literal_text = parts.first().copied().unwrap_or_else(MbValue::none);
        let field_name = parts.get(1).copied().unwrap_or_else(MbValue::none);
        let format_spec = parts.get(2).copied().unwrap_or_else(MbValue::none);
        let conversion = parts.get(3).copied().unwrap_or_else(MbValue::none);

        if !is_none_value(literal_text) {
            result.push_str(&pystr(literal_text));
        }
        if !is_none_value(field_name) {
            let fname = pystr(field_name);
            let key_field: String = if fname.is_empty() {
                if auto_arg_index < 0 {
                    return raise("ValueError",
                        "cannot switch from manual field specification to automatic field numbering");
                }
                let k = format!("{}", auto_arg_index);
                auto_arg_index += 1;
                k
            } else {
                // A leading digit ⇒ manual numbering.
                let first_char = fname.chars().next().unwrap_or(' ');
                if first_char.is_ascii_digit() {
                    if auto_arg_index > 0 {
                        return raise("ValueError",
                            "cannot switch from automatic field numbering to manual field specification");
                    }
                    auto_arg_index = -1;
                }
                fname.clone()
            };
            let (obj, used_key) = get_field_dispatch(this, &key_field, args, kwargs);
            if has_exc() {
                return MbValue::none();
            }
            // Record the consumed argument key (int index or str name).
            super::super::set_ops::mb_set_add(used_args, used_key);
            let obj = apply_conversion_and_format(
                this,
                obj,
                conversion,
                format_spec,
                args,
                kwargs,
                used_args,
                recursion_depth,
                &mut auto_arg_index,
            );
            if has_exc() {
                return MbValue::none();
            }
            result.push_str(&pystr(obj));
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        new_str(result),
        MbValue::from_int(auto_arg_index),
    ]))
}

/// Run `self.get_field(field_name, args, kwargs)`; returns (object, used_key).
fn get_field_dispatch(
    this: MbValue,
    field_name: &str,
    args: MbValue,
    kwargs: MbValue,
) -> (MbValue, MbValue) {
    let r = call_method(this, "get_field", vec![new_str(field_name), args, kwargs]);
    if has_exc() {
        return (MbValue::none(), MbValue::none());
    }
    // get_field returns (obj, used_key)
    let parts = super::super::builtins::extract_items(r);
    let obj = parts.first().copied().unwrap_or_else(MbValue::none);
    let key = parts.get(1).copied().unwrap_or_else(MbValue::none);
    (obj, key)
}

/// Apply conversion (!s/!r/!a) then recursively expand the format spec and
/// run format_field. Threads `auto_arg_index` through the nested `_vformat`.
fn apply_conversion_and_format(
    this: MbValue,
    obj: MbValue,
    conversion: MbValue,
    format_spec: MbValue,
    args: MbValue,
    kwargs: MbValue,
    used_args: MbValue,
    recursion_depth: i64,
    auto_arg_index: &mut i64,
) -> MbValue {
    let converted = call_method(this, "convert_field", vec![obj, conversion]);
    if has_exc() {
        return MbValue::none();
    }
    // Expand the format spec (it may itself contain replacement fields).
    let spec = if format_spec.is_none() {
        new_str("")
    } else {
        format_spec
    };
    let pair = call_method(
        this,
        "_vformat",
        vec![
            spec,
            args,
            kwargs,
            used_args,
            MbValue::from_int(recursion_depth - 1),
            MbValue::from_int(*auto_arg_index),
        ],
    );
    if has_exc() {
        return MbValue::none();
    }
    let parts = super::super::builtins::extract_items(pair);
    let expanded = parts.first().copied().unwrap_or_else(|| new_str(""));
    if let Some(idx) = parts.get(1).and_then(|v| v.as_int()) {
        *auto_arg_index = idx;
    }
    let r = call_method(this, "format_field", vec![converted, expanded]);
    if has_exc() {
        return MbValue::none();
    }
    r
}

// Formatter.parse(self, format_string) → list of 4-tuples
extern "C" fn m_formatter_parse(_this: MbValue, format_string: MbValue) -> MbValue {
    let s = str_of(format_string).unwrap_or_default();
    match formatter_parse(&s) {
        Ok(fields) => {
            let tuples: Vec<MbValue> = fields
                .into_iter()
                .map(|f| {
                    let conv = match f.conversion {
                        Some(c) => new_str(c.to_string()),
                        None => MbValue::none(),
                    };
                    let fname = match f.field_name {
                        Some(n) => new_str(n),
                        None => MbValue::none(),
                    };
                    let spec = match f.format_spec {
                        Some(sp) => new_str(sp),
                        None => MbValue::none(),
                    };
                    MbValue::from_ptr(MbObject::new_tuple(vec![
                        new_str(f.literal),
                        fname,
                        spec,
                        conv,
                    ]))
                })
                .collect();
            new_list(tuples)
        }
        Err(msg) => raise("ValueError", msg),
    }
}

// Formatter.get_field(self, field_name, args, kwargs) → (obj, first_key)
extern "C" fn m_formatter_get_field(
    this: MbValue,
    field_name: MbValue,
    args: MbValue,
    kwargs: MbValue,
) -> MbValue {
    let fname = str_of(field_name).unwrap_or_default();
    let (first, rest) = field_name_split(&fname);
    // key is int if it parses as an integer, else the string name.
    let key: MbValue = if let Ok(idx) = first.parse::<i64>() {
        MbValue::from_int(idx)
    } else {
        new_str(first.clone())
    };
    let mut obj = call_method(this, "get_value", vec![key, args, kwargs]);
    if has_exc() {
        return MbValue::none();
    }
    for (is_attr, accessor) in rest {
        if is_attr {
            // mb_getattr does not raise for missing attributes on built-in
            // types (it returns None); detect that and raise AttributeError so
            // `{0.attr}` against e.g. a str matches CPython.
            if !builtin_has_attr(obj, &accessor) {
                let tn = builtin_type_name(obj);
                if let Some(name) = tn {
                    return raise(
                        "AttributeError",
                        format!("'{}' object has no attribute '{}'", name, accessor),
                    );
                }
            }
            obj = super::super::class::mb_getattr(obj, new_str(accessor));
        } else {
            // index: int if numeric else string key
            if let Ok(i) = accessor.parse::<i64>() {
                if let Some(err) = index_oob_error(obj, i) {
                    return raise(err.0, err.1);
                }
                obj = super::super::class::mb_obj_getitem(obj, MbValue::from_int(i));
            } else {
                // string key (dict lookup)
                if is_dict_value(obj) && dict_get(obj, &accessor).is_none() {
                    return raise("KeyError", format!("'{}'", accessor));
                }
                obj = super::super::class::mb_obj_getitem(obj, new_str(accessor));
            }
        }
        if has_exc() {
            return MbValue::none();
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![obj, key]))
}

fn is_dict_value(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
}

/// Built-in container type name for `obj` (None for user Instances, which keep
/// their own __getattr__ semantics).
fn builtin_type_name(v: MbValue) -> Option<&'static str> {
    let p = v.as_ptr()?;
    unsafe {
        match &(*p).data {
            ObjData::Str(_) => Some("str"),
            ObjData::List(_) => Some("list"),
            ObjData::Tuple(_) => Some("tuple"),
            ObjData::Dict(_) => Some("dict"),
            ObjData::Set(_) => Some("set"),
            ObjData::Bytes(_) => Some("bytes"),
            _ => None,
        }
    }
}

/// True when `obj` is NOT a built-in container (so attr access defers to the
/// runtime, e.g. user Instances with __getattr__), or when the built-in
/// genuinely exposes that attribute. Built-in containers expose no data
/// attributes via `{0.attr}`, so this returns false for any name on them.
fn builtin_has_attr(v: MbValue, _attr: &str) -> bool {
    builtin_type_name(v).is_none()
}

/// If `obj` is a built-in sequence and `idx` is out of range, return the
/// (exc_type, message) to raise; otherwise None.
fn index_oob_error(v: MbValue, idx: i64) -> Option<(&'static str, String)> {
    let p = v.as_ptr()?;
    unsafe {
        let len = match &(*p).data {
            ObjData::List(ref lock) => lock.read().unwrap().len() as i64,
            ObjData::Tuple(ref items) => items.len() as i64,
            ObjData::Str(ref s) => s.chars().count() as i64,
            ObjData::Bytes(ref b) => b.len() as i64,
            // dict: an integer key may legitimately be present.
            _ => return None,
        };
        let actual = if idx < 0 { idx + len } else { idx };
        if actual < 0 || actual >= len {
            let kind = match &(*p).data {
                ObjData::Str(_) => "string index out of range",
                ObjData::Tuple(_) => "tuple index out of range",
                _ => "list index out of range",
            };
            return Some(("IndexError", kind.to_string()));
        }
    }
    None
}

// Formatter.get_value(self, key, args, kwargs)
extern "C" fn m_formatter_get_value(
    _this: MbValue,
    key: MbValue,
    args: MbValue,
    kwargs: MbValue,
) -> MbValue {
    if let Some(idx) = key.as_int() {
        let items = super::super::builtins::extract_items(args);
        if idx < 0 || idx as usize >= items.len() {
            return raise(
                "IndexError",
                "Replacement index out of range for positional args tuple",
            );
        }
        return items[idx as usize];
    }
    // string key → kwargs[key]
    super::super::class::mb_obj_getitem(kwargs, key)
}

// Formatter.check_unused_args(self, used_args, args, kwargs) — default no-op
extern "C" fn m_formatter_check_unused_args(
    _this: MbValue,
    _used: MbValue,
    _args: MbValue,
    _kwargs: MbValue,
) -> MbValue {
    MbValue::none()
}

// Formatter.convert_field(self, value, conversion)
extern "C" fn m_formatter_convert_field(
    _this: MbValue,
    value: MbValue,
    conversion: MbValue,
) -> MbValue {
    if conversion.is_none() {
        return value;
    }
    let c = str_of(conversion).unwrap_or_default();
    match c.as_str() {
        "s" => super::super::builtins::mb_str(value),
        "r" => super::super::builtins::mb_repr(value),
        "a" => super::super::builtins::mb_ascii(value),
        other => raise(
            "ValueError",
            format!("Unknown conversion specifier {}", other),
        ),
    }
}

// Formatter.format_field(self, value, format_spec)
extern "C" fn m_formatter_format_field(
    _this: MbValue,
    value: MbValue,
    format_spec: MbValue,
) -> MbValue {
    let spec = if format_spec.is_none() {
        new_str("")
    } else {
        format_spec
    };
    super::super::builtins::mb_format(value, spec)
}

fn make_set() -> MbValue {
    super::super::set_ops::mb_set_new()
}

/// Entry point used by `mb_str_format_kwargs` when its receiver is a Formatter
/// instance (the `.format(..., kw=...)` lowering path). Delegates to the real
/// `format` method so subclass overrides + kwargs both work.
pub fn formatter_format_from_kwargs(this: MbValue, pos_args: MbValue, kwargs: MbValue) -> MbValue {
    let items = super::super::builtins::extract_items(pos_args);
    // `format_string` is positional-only — passing it as a keyword is a
    // TypeError (CPython: "format() got some positional-only arguments passed
    // as keyword arguments: 'format_string'").
    if items.is_empty() && dict_get(kwargs, "format_string").is_some() {
        return raise("TypeError",
            "format() got some positional-only arguments passed as keyword arguments: 'format_string'");
    }
    if items.is_empty() {
        return raise(
            "TypeError",
            "format() missing 1 required positional argument: 'format_string'",
        );
    }
    let format_string = items.first().copied().unwrap_or_else(MbValue::none);
    let rest: Vec<MbValue> = items.iter().skip(1).copied().collect();
    formatter_vformat_entry(this, format_string, rest, kwargs)
}

/// True when `v` is a Formatter-family instance (used by string_ops bridge).
pub fn value_is_formatter(v: MbValue) -> bool {
    is_formatter_instance(v)
}

// ╔══════════════════════════════════════════════════════════════════════╗
// ║ string.Template                                                        ║
// ╚══════════════════════════════════════════════════════════════════════╝

/// Resolve the effective grammar config for a Template instance, honoring
/// subclass overrides of `delimiter`, `idpattern`, `braceidpattern`, `flags`,
/// and a fully custom `pattern`.
struct TemplateConfig {
    delimiter: String,
    idpattern: String,
    braceidpattern: Option<String>,
    ignorecase: bool,
    custom_pattern: Option<String>,
}

fn template_config(this: MbValue) -> TemplateConfig {
    let getattr_str = |name: &str| -> Option<String> {
        let v = super::super::class::mb_getattr_default(this, new_str(name), MbValue::none());
        if v.is_none() {
            None
        } else {
            str_of(v)
        }
    };
    let delimiter = getattr_str("delimiter").unwrap_or_else(|| "$".to_string());
    let idpattern = getattr_str("idpattern").unwrap_or_else(|| "(?a:[_a-z][_a-z0-9]*)".to_string());
    let braceidpattern = getattr_str("braceidpattern");
    // flags: re.IGNORECASE == 2. Default is IGNORECASE on.
    let flags_v =
        super::super::class::mb_getattr_default(this, new_str("flags"), MbValue::from_int(2));
    let flags = flags_v.as_int().unwrap_or(2);
    let ignorecase = (flags & 2) != 0;
    let custom_pattern = getattr_str("pattern");
    TemplateConfig {
        delimiter,
        idpattern,
        braceidpattern,
        ignorecase,
        custom_pattern,
    }
}

/// Convert a CPython idpattern fragment to a Rust-regex compatible one.
/// CPython idpattern default is `(?a:[_a-z][_a-z0-9]*)`; the `(?a:...)` ASCII
/// flag group is dropped (Rust regex is byte/Unicode but the char class is
/// ASCII anyway).
fn normalize_idpattern(p: &str) -> String {
    let t = p.trim();
    // Strip a leading `(?a:` ... `)` wrapper if present.
    if let Some(inner) = t.strip_prefix("(?a:").and_then(|s| s.strip_suffix(')')) {
        return inner.to_string();
    }
    t.to_string()
}

/// A single placeholder match in the template.
enum Token {
    Literal(String),
    Escaped,        // delimiter delimiter -> literal delimiter
    Named(String),  // $name
    Braced(String), // ${name}
    Invalid(usize), // index into the string where the bad delimiter is
}

/// Tokenize the template using the (possibly subclass-overridden) grammar.
/// Returns Err(byte_index_of_invalid) when a custom pattern path needs error
/// reporting handled by the regex engine; here we hand-roll the default and
/// split-id grammars and use a regex fallback for custom patterns.
fn tokenize_default(template: &str, cfg: &TemplateConfig) -> Result<Vec<Token>, usize> {
    let delim: Vec<char> = cfg.delimiter.chars().collect();
    let chars: Vec<char> = template.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;
    let mut lit = String::new();

    let id_re = build_id_regex(&normalize_idpattern(&cfg.idpattern), cfg.ignorecase);
    let brace_src = cfg
        .braceidpattern
        .clone()
        .map(|b| normalize_idpattern(&b))
        .unwrap_or_else(|| normalize_idpattern(&cfg.idpattern));
    let brace_re = build_id_regex(&brace_src, cfg.ignorecase);

    let matches_delim = |pos: usize| -> bool {
        if delim.is_empty() {
            return false;
        }
        if pos + delim.len() > n {
            return false;
        }
        chars[pos..pos + delim.len()] == delim[..]
    };

    while i < n {
        if matches_delim(i) {
            let dstart = i;
            let after = i + delim.len();
            // escaped: delimiter delimiter
            if matches_delim(after) {
                if !lit.is_empty() {
                    out.push(Token::Literal(std::mem::take(&mut lit)));
                }
                out.push(Token::Escaped);
                i = after + delim.len();
                continue;
            }
            // braced: delimiter { name }
            if after < n && chars[after] == '{' {
                // find closing '}'
                let mut j = after + 1;
                let start = j;
                while j < n && chars[j] != '}' {
                    j += 1;
                }
                if j < n {
                    let name: String = chars[start..j].iter().collect();
                    if regex_full_match(&brace_re, &name) {
                        if !lit.is_empty() {
                            out.push(Token::Literal(std::mem::take(&mut lit)));
                        }
                        out.push(Token::Braced(name));
                        i = j + 1;
                        continue;
                    }
                }
                // invalid braced placeholder: the delimiter matches `invalid`
                // (empty), consuming just the delimiter. Emit Invalid and keep
                // scanning from immediately after the delimiter.
                if !lit.is_empty() {
                    out.push(Token::Literal(std::mem::take(&mut lit)));
                }
                out.push(Token::Invalid(dstart));
                i = after;
                continue;
            }
            // named: delimiter name
            if after < n {
                // greedily match the id pattern from `after`
                let tail: String = chars[after..].iter().collect();
                if let Some(m) = regex_prefix_match(&id_re, &tail) {
                    if !lit.is_empty() {
                        out.push(Token::Literal(std::mem::take(&mut lit)));
                    }
                    out.push(Token::Named(m.clone()));
                    i = after + m.chars().count();
                    continue;
                }
            }
            // invalid lone/trailing delimiter — consume just the delimiter.
            if !lit.is_empty() {
                out.push(Token::Literal(std::mem::take(&mut lit)));
            }
            out.push(Token::Invalid(dstart));
            i = after;
            continue;
        }
        lit.push(chars[i]);
        i += 1;
    }
    if !lit.is_empty() {
        out.push(Token::Literal(lit));
    }
    Ok(out)
}

fn build_id_regex(body: &str, ignorecase: bool) -> regex::Regex {
    let prefix = if ignorecase { "(?i)" } else { "" };
    let pat = format!("{}^(?:{})", prefix, body);
    regex::Regex::new(&pat).unwrap_or_else(|_| regex::Regex::new("^$").unwrap())
}

/// True if the whole string matches the id regex (anchored both ends).
fn regex_full_match(re: &regex::Regex, s: &str) -> bool {
    if let Some(m) = re.find(s) {
        m.start() == 0 && m.end() == s.len()
    } else {
        false
    }
}

/// Match the id regex at the start of `s`, returning the matched prefix.
fn regex_prefix_match(re: &regex::Regex, s: &str) -> Option<String> {
    re.find(s)
        .filter(|m| m.start() == 0)
        .map(|m| m.as_str().to_string())
}

/// Compute (line, col) of a byte/char offset for error messages.
fn line_col(template: &str, char_idx: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, c) in template.chars().enumerate() {
        if i == char_idx {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Look up a name in (kwargs first, then mapping). Returns Some(value) when
/// found, None when absent. A `__getitem__` that raises (KeyError) is treated
/// as "absent" (the exception is cleared); the caller re-raises KeyError or
/// keeps the placeholder for safe_substitute.
fn template_lookup(name: &str, mapping: MbValue, kwargs: MbValue) -> Option<MbValue> {
    // kwargs first
    if let Some(v) = dict_get(kwargs, name) {
        return Some(v);
    }
    if mapping.is_none() {
        return None;
    }
    // A plain dict: probe directly so a genuine None value is still "found".
    if mapping
        .as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
    {
        return dict_get(mapping, name);
    }
    // Otherwise an object supporting __getitem__ (e.g. a custom Mapping).
    let v = super::super::class::mb_obj_getitem(mapping, new_str(name));
    if has_exc() {
        super::super::exception::mb_clear_exception();
        return None;
    }
    Some(v)
}

fn dict_get(d: MbValue, key: &str) -> Option<MbValue> {
    d.as_ptr().and_then(|p| unsafe {
        if let ObjData::Dict(ref lock) = (*p).data {
            lock.read()
                .unwrap()
                .get(&super::super::dict_ops::DictKey::Str(key.to_string()))
                .copied()
        } else {
            None
        }
    })
}

fn template_string(this: MbValue) -> String {
    let v = super::super::class::mb_getattr_default(this, new_str("template"), MbValue::none());
    str_of(v).unwrap_or_default()
}

// Template.__init__(self, template) — store the raw template string. Runs for
// both `string.Template(...)` and subclass instantiation `Sub(...)`.
extern "C" fn m_template_init(this: MbValue, template: MbValue) -> MbValue {
    super::super::class::mb_setattr(this, new_str("template"), template);
    MbValue::none()
}

// Template.substitute(self, mapping={}, /, **kwargs)  variadic+kwargs
extern "C" fn m_template_substitute(this: MbValue, args_list: MbValue, kwargs: MbValue) -> MbValue {
    let items = super::super::builtins::extract_items(args_list);
    let mapping = items.first().copied().unwrap_or_else(MbValue::none);
    template_do_substitute(this, mapping, kwargs, false)
}

// Template.safe_substitute(self, mapping={}, /, **kwargs)
extern "C" fn m_template_safe_substitute(
    this: MbValue,
    args_list: MbValue,
    kwargs: MbValue,
) -> MbValue {
    let items = super::super::builtins::extract_items(args_list);
    let mapping = items.first().copied().unwrap_or_else(MbValue::none);
    template_do_substitute(this, mapping, kwargs, true)
}

fn template_do_substitute(this: MbValue, mapping: MbValue, kwargs: MbValue, safe: bool) -> MbValue {
    let cfg = template_config(this);
    let template = template_string(this);

    // Custom pattern: fall back to a regex-driven scan.
    if let Some(pat) = &cfg.custom_pattern {
        return template_substitute_custom(&template, pat, &cfg, mapping, kwargs, safe);
    }

    let tokens = match tokenize_default(&template, &cfg) {
        Ok(t) => t,
        Err(idx) => {
            let (l, c) = line_col(&template, idx);
            return raise(
                "ValueError",
                format!("Invalid placeholder in string: line {}, col {}", l, c),
            );
        }
    };
    let mut out = String::new();
    for tok in tokens {
        match tok {
            Token::Literal(s) => out.push_str(&s),
            Token::Escaped => out.push_str(&cfg.delimiter),
            Token::Named(name) => match template_lookup(&name, mapping, kwargs) {
                Some(v) => out.push_str(&pystr(v)),
                None => {
                    if safe {
                        out.push_str(&format!("{}{}", cfg.delimiter, name));
                    } else {
                        return raise_keyerror(&name);
                    }
                }
            },
            Token::Braced(name) => match template_lookup(&name, mapping, kwargs) {
                Some(v) => out.push_str(&pystr(v)),
                None => {
                    if safe {
                        out.push_str(&format!("{}{{{}}}", cfg.delimiter, name));
                    } else {
                        return raise_keyerror(&name);
                    }
                }
            },
            Token::Invalid(idx) => {
                if safe {
                    out.push_str(&cfg.delimiter);
                } else {
                    let (l, c) = line_col(&template, idx);
                    return raise(
                        "ValueError",
                        format!("Invalid placeholder in string: line {}, col {}", l, c),
                    );
                }
            }
        }
    }
    new_str(out)
}

fn raise_keyerror(name: &str) -> MbValue {
    super::super::exception::mb_raise(new_str("KeyError"), new_str(name.to_string()));
    MbValue::none()
}

/// Translate a CPython regex into a Rust-regex-compatible one for the narrow
/// shapes used by `string.Template` patterns: escape a `{` that does not begin
/// a valid `{m}` / `{m,}` / `{m,n}` / `{,n}` quantifier, and escape any `}`
/// that does not close one. Honors `\`-escapes and `[...]` character classes
/// (braces inside a class are literal already).
fn cpython_re_to_rust(pat: &str) -> String {
    let chars: Vec<char> = pat.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(n + 8);
    let mut i = 0;
    let mut in_class = false;
    while i < n {
        let c = chars[i];
        match c {
            '\\' => {
                out.push(c);
                if i + 1 < n {
                    out.push(chars[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            '[' if !in_class => {
                in_class = true;
                out.push(c);
                i += 1;
                continue;
            }
            ']' if in_class => {
                in_class = false;
                out.push(c);
                i += 1;
                continue;
            }
            '{' if !in_class => {
                // Look ahead for a valid quantifier body: digits, optional comma,
                // digits, then '}'.
                let mut j = i + 1;
                let mut saw_digit = false;
                while j < n && chars[j].is_ascii_digit() {
                    j += 1;
                    saw_digit = true;
                }
                if j < n && chars[j] == ',' {
                    j += 1;
                    while j < n && chars[j].is_ascii_digit() {
                        j += 1;
                        saw_digit = true;
                    }
                }
                if saw_digit && j < n && chars[j] == '}' {
                    // valid quantifier; copy verbatim
                    for &ch in &chars[i..=j] {
                        out.push(ch);
                    }
                    i = j + 1;
                } else {
                    out.push_str("\\{");
                    i += 1;
                }
                continue;
            }
            '}' if !in_class => {
                // A bare `}` not closing a quantifier — escape it. (Quantifier
                // closers were already consumed above.)
                out.push_str("\\}");
                i += 1;
                continue;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Custom-pattern substitution path: compile the user `pattern` (re.VERBOSE)
/// with named groups escaped/named/braced/invalid and drive substitution.
fn template_substitute_custom(
    template: &str,
    pattern: &str,
    cfg: &TemplateConfig,
    mapping: MbValue,
    kwargs: MbValue,
    safe: bool,
) -> MbValue {
    // CPython compiles `pattern` with re.IGNORECASE|re.VERBOSE (or just VERBOSE
    // when flags overridden). Build a Rust regex with (?x) verbose and
    // optionally (?i). CPython's `re` accepts a bare `{`/`}` as a literal when
    // it is not a valid `{m,n}` quantifier; Rust's regex is stricter, so
    // pre-escape those braces.
    let mut prefix = String::from("(?x)");
    if cfg.ignorecase {
        prefix.push_str("(?i)");
    }
    let translated = cpython_re_to_rust(pattern);
    let full = format!("{}{}", prefix, translated);
    let re = match regex::Regex::new(&full) {
        Ok(r) => r,
        Err(_) => {
            // Can't compile under Rust regex; surface as a ValueError so the
            // caller does not silently mis-substitute.
            return raise("ValueError", "invalid Template pattern");
        }
    };
    let mut out = String::new();
    let mut last = 0usize;
    for caps in re.captures_iter(template) {
        let m = caps.get(0).unwrap();
        out.push_str(&template[last..m.start()]);
        last = m.end();
        let named = caps.name("named").map(|x| x.as_str().to_string());
        let braced = caps.name("braced").map(|x| x.as_str().to_string());
        let escaped = caps.name("escaped");
        let invalid = caps.name("invalid");
        if escaped.is_some() {
            out.push_str(&cfg.delimiter);
            continue;
        }
        if let Some(name) = named.or(braced) {
            match template_lookup(&name, mapping, kwargs) {
                Some(v) => out.push_str(&pystr(v)),
                None => {
                    if safe {
                        out.push_str(m.as_str());
                    } else {
                        return raise_keyerror(&name);
                    }
                }
            }
            continue;
        }
        if invalid.is_some() {
            if safe {
                // safe_substitute keeps the matched (invalid) text verbatim.
                out.push_str(m.as_str());
            } else {
                let off = m.start();
                let cidx = template[..off].chars().count();
                let (l, c) = line_col(template, cidx);
                return raise(
                    "ValueError",
                    format!("Invalid placeholder in string: line {}, col {}", l, c),
                );
            }
            continue;
        }
        // No recognized group (escaped/named/braced/invalid) participated — the
        // pattern has an unexpected capture (e.g. a stray `badname`). CPython
        // raises ValueError in both substitute and safe_substitute.
        return raise("ValueError", "Unrecognized named group in pattern");
    }
    out.push_str(&template[last..]);
    new_str(out)
}

// Template.get_identifiers(self) → list of names
extern "C" fn m_template_get_identifiers(this: MbValue) -> MbValue {
    let cfg = template_config(this);
    let template = template_string(this);
    let mut ids: Vec<MbValue> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    if cfg.custom_pattern.is_none() {
        if let Ok(tokens) = tokenize_default(&template, &cfg) {
            for tok in tokens {
                if let Token::Named(name) | Token::Braced(name) = tok {
                    if seen.insert(name.clone()) {
                        ids.push(new_str(name));
                    }
                }
            }
        }
    }
    new_list(ids)
}

// Template.is_valid(self) → bool
extern "C" fn m_template_is_valid(this: MbValue) -> MbValue {
    let cfg = template_config(this);
    let template = template_string(this);
    if cfg.custom_pattern.is_none() {
        if let Ok(tokens) = tokenize_default(&template, &cfg) {
            for tok in tokens {
                if let Token::Invalid(_) = tok {
                    return MbValue::from_bool(false);
                }
            }
            return MbValue::from_bool(true);
        }
    }
    MbValue::from_bool(true)
}

// ╔══════════════════════════════════════════════════════════════════════╗
// ║ string.capwords                                                        ║
// ╚══════════════════════════════════════════════════════════════════════╝

/// string.capwords(s, sep=None):
///   (sep or ' ').join(x.capitalize() for x in s.split(sep))
pub fn mb_string_capwords(val: MbValue, sep: MbValue) -> MbValue {
    let s = match str_of(val) {
        Some(s) => s,
        // capwords calls `val.split(...)`; a non-string raises AttributeError
        // ("'int' object has no attribute 'split'"), matching CPython, not a
        // silent None.
        None => {
            return raise(
                "AttributeError",
                &format!(
                    "'{}' object has no attribute 'split'",
                    super::super::builtins::value_type_name(val)
                ),
            );
        }
    };
    let sep_str = if sep.is_none() { None } else { str_of(sep) };

    let words: Vec<String> = match &sep_str {
        None => s.split_whitespace().map(|w| w.to_string()).collect(),
        Some(sp) if sp.is_empty() => {
            // s.split('') raises ValueError in Python.
            return raise("ValueError", "empty separator");
        }
        Some(sp) => s.split(sp.as_str()).map(|w| w.to_string()).collect(),
    };
    let capitalized: Vec<String> = words.iter().map(|w| capitalize(w)).collect();
    let joiner = sep_str.clone().unwrap_or_else(|| " ".to_string());
    new_str(capitalized.join(&joiner))
}

/// Python str.capitalize(): first char upper, rest lower.
fn capitalize(w: &str) -> String {
    let mut chars = w.chars();
    match chars.next() {
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            format!("{}{}", upper, chars.as_str().to_lowercase())
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capwords_default() {
        let s = new_str("hello world foo");
        let result = mb_string_capwords(s, MbValue::none());
        assert_eq!(str_of(result).as_deref(), Some("Hello World Foo"));
    }

    #[test]
    fn test_capwords_sep() {
        let s = new_str("ABC-DEF-GHI");
        let result = mb_string_capwords(s, new_str("-"));
        assert_eq!(str_of(result).as_deref(), Some("Abc-Def-Ghi"));
    }

    #[test]
    fn test_formatter_parse_basic() {
        // CPython 3.12: list(Formatter().parse('foo{0}{1}-{1}')) yields
        // [('foo','0','',None), ('','1','',None), ('-','1','',None)] — 3 tuples.
        let parsed = formatter_parse("foo{0}{1}-{1}").unwrap();
        assert_eq!(parsed.len(), 3);
    }
}
