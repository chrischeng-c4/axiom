use super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::value::MbValue;
/// Exception handling runtime (#283).
///
/// Implements Python-compatible exception objects and the try/except/finally
/// runtime support using setjmp/longjmp-style unwinding.
use rustc_hash::FxHashMap;

/// Exception object stored on the heap.
#[derive(Clone)]
pub struct MbException {
    /// Exception type name (e.g. "ValueError", "TypeError")
    pub exc_type: String,
    /// Exception message
    pub message: String,
    /// Chained exception (__cause__ from `raise X from Y`)
    pub cause: Option<Box<MbException>>,
    /// Implicit chaining (__context__ — active exception when this was raised)
    pub context: Option<Box<MbException>>,
    /// Whether __context__ should be suppressed (set by `from None`)
    pub suppress_context: bool,
    /// Traceback entries: (filename, line, function)
    pub traceback: Vec<(String, u32, String)>,
}

impl MbException {
    pub fn new(exc_type: &str, message: &str) -> Self {
        Self {
            exc_type: exc_type.to_string(),
            message: message.to_string(),
            cause: None,
            context: None,
            suppress_context: false,
            traceback: Vec::new(),
        }
    }

    pub fn with_cause(mut self, cause: MbException) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    pub fn with_context(mut self, context: MbException) -> Self {
        self.context = Some(Box::new(context));
        self
    }
}

// Thread-local exception state — current unhandled exception.
thread_local! {
    static CURRENT_EXCEPTION: std::cell::RefCell<Option<MbException>> = std::cell::RefCell::new(None);
    static EXCEPTION_HANDLERS: std::cell::RefCell<Vec<ExceptionHandler>> = std::cell::RefCell::new(Vec::new());
    /// Most-recently caught exception. Mirrors CPython's "currently handled
    /// exception" reachable via sys.exc_info() / traceback.format_exc().
    /// Populated by mb_catch_exception; never cleared automatically — we keep
    /// it around so `format_exc()` can stringify the latest handled exception
    /// even after the except-block has exited.
    static LAST_HANDLED_EXCEPTION: std::cell::RefCell<Option<MbException>> = std::cell::RefCell::new(None);
    /// Save/restore stack for the "currently handled exception" (CPython
    /// clears it when an except-handler region unwinds). Each entry snapshots
    /// BOTH `LAST_HANDLED_EXCEPTION` (here) and `class.rs`'s `LAST_CAUGHT_VALUE`
    /// (read via the bits getter). `mb_save_handled_exc` pushes a snapshot at
    /// try-entry and returns its index; `mb_restore_handled_exc` pops back to
    /// that index and reinstates the snapshot when the region exits.
    static HANDLED_EXC_SAVE_STACK: std::cell::RefCell<Vec<(Option<MbException>, u64)>> =
        std::cell::RefCell::new(Vec::new());
}

/// Snapshot the currently-handled-exception state (both thread-locals) at
/// try-entry, push it on the save stack, and return a token (the slot index).
/// The snapshot is taken BEFORE any `except` handler runs `mb_catch_exception*`,
/// so it captures the *enclosing* frame's handled exception (which CPython
/// restores once this handler region unwinds).
pub fn mb_save_handled_exc() -> i64 {
    let last_handled = LAST_HANDLED_EXCEPTION.with(|h| h.borrow().clone());
    let last_caught_bits = super::class::last_caught_value_bits();
    // Park an owned reference to the caught value while it sits on the save
    // stack so it can't be freed before we restore it (LAST_CAUGHT_VALUE itself
    // is a borrowed slot — its owner is the live except handler's binding, which
    // may be dropped before the region we are nested in exits).
    unsafe {
        super::rc::retain_if_ptr(MbValue::from_bits(last_caught_bits));
    }
    HANDLED_EXC_SAVE_STACK.with(|s| {
        let mut stack = s.borrow_mut();
        let token = stack.len() as i64;
        stack.push((last_handled, last_caught_bits));
        token
    })
}

/// Restore the handled-exception state captured by `mb_save_handled_exc` for
/// `token`, then drop that slot (and any deeper ones). Restoring at the
/// region-exit edges (handler fall-through / return-from-handler) reinstates
/// the enclosing frame's handled exception, so `sys.exception()` /
/// `sys.exc_info()` / `traceback.format_exc()` no longer leak the just-handled
/// exception. Idempotent: a token at/above the current depth is a no-op.
pub fn mb_restore_handled_exc(token: i64) {
    if token < 0 {
        return;
    }
    let token = token as usize;
    // Pop this slot and every deeper one. The slot at `token` is reinstated;
    // the deeper ones (whose own regions were exited abnormally without a
    // matching restore — e.g. a raise that propagated past them) are discarded.
    // Each carried a parked retain (see mb_save_handled_exc) that must be
    // released exactly once now.
    let popped = HANDLED_EXC_SAVE_STACK.with(|s| {
        let mut stack = s.borrow_mut();
        if token >= stack.len() {
            return Vec::new();
        }
        stack.split_off(token)
    });
    if let Some((last_handled, last_caught_bits)) = popped.first().cloned() {
        LAST_HANDLED_EXCEPTION.with(|h| {
            *h.borrow_mut() = last_handled;
        });
        super::class::set_last_caught_value_bits(last_caught_bits);
    }
    // Release every parked retain we just removed from the stack.
    for (_, bits) in &popped {
        unsafe {
            super::rc::release_if_ptr(MbValue::from_bits(*bits));
        }
    }
}

/// Public read accessor for the most-recently caught exception.
/// Returns a clone — caller does not own the cell.
pub fn last_handled_exception() -> Option<(String, String)> {
    LAST_HANDLED_EXCEPTION.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|e| (e.exc_type.clone(), e.message.clone()))
    })
}

/// An exception handler frame (registered by try blocks).
#[allow(dead_code)]
struct ExceptionHandler {
    /// Which exception types this handler catches (empty = catch all)
    catch_types: Vec<String>,
    /// Whether this handler has a finally block
    has_finally: bool,
}

// ── Exception Creation ──

/// Create a new exception of the given type.
pub fn mb_exception_new(exc_type: MbValue, message: MbValue) -> MbValue {
    let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
    let msg = message_display(message);
    let exc = MbException::new(&type_name, &msg);
    store_exception_as_value(exc)
}

fn new_str_value(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn list_items(args_list: MbValue) -> Vec<MbValue> {
    let mut arg_items: Vec<MbValue> = Vec::new();
    if let Some(ptr) = args_list.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                arg_items = lock.read().unwrap().to_vec();
            }
        }
    }
    arg_items
}

fn tuple_items(value: MbValue) -> Option<Vec<MbValue>> {
    value.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Tuple(ref items) = (*ptr).data {
            Some(items.clone())
        } else {
            None
        }
    })
}

fn value_as_i64(value: MbValue) -> Option<i64> {
    value.as_int_pyint()
}

fn value_as_bytes(value: MbValue) -> Option<Vec<u8>> {
    value.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Bytes(data) => Some(data.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

fn insert_borrowed_field(fields: &mut FxHashMap<String, MbValue>, key: &str, value: MbValue) {
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    fields.insert(key.to_string(), value);
}

fn raise_type_error_message(message: impl Into<String>) {
    mb_raise(new_str_value("TypeError"), new_str_value(message.into()));
}

fn populate_import_error_fields(fields: &mut FxHashMap<String, MbValue>, arg_items: &[MbValue]) {
    let msg = arg_items.first().copied().unwrap_or_else(MbValue::none);
    insert_borrowed_field(fields, "msg", msg);
    fields.insert("name".to_string(), MbValue::none());
    fields.insert("path".to_string(), MbValue::none());
}

fn populate_syntax_error_fields(fields: &mut FxHashMap<String, MbValue>, arg_items: &[MbValue]) -> bool {
    let msg = arg_items.first().copied().unwrap_or_else(MbValue::none);
    insert_borrowed_field(fields, "msg", msg);
    fields.insert("filename".to_string(), MbValue::none());
    fields.insert("lineno".to_string(), MbValue::none());
    fields.insert("offset".to_string(), MbValue::none());
    fields.insert("text".to_string(), MbValue::none());
    fields.insert("end_lineno".to_string(), MbValue::none());
    fields.insert("end_offset".to_string(), MbValue::none());

    let Some(details) = arg_items.get(1).copied() else {
        return true;
    };
    let Some(items) = tuple_items(details) else {
        raise_type_error_message("SyntaxError() argument 2 must be a tuple");
        return false;
    };
    if items.len() != 4 && items.len() != 6 {
        raise_type_error_message("SyntaxError() argument 2 must be a 4-item or 6-item tuple");
        return false;
    }
    insert_borrowed_field(fields, "filename", items[0]);
    insert_borrowed_field(fields, "lineno", items[1]);
    insert_borrowed_field(fields, "offset", items[2]);
    insert_borrowed_field(fields, "text", items[3]);
    if items.len() == 6 {
        insert_borrowed_field(fields, "end_lineno", items[4]);
        insert_borrowed_field(fields, "end_offset", items[5]);
    }
    true
}

fn populate_unicode_error_fields(
    fields: &mut FxHashMap<String, MbValue>,
    type_name: &str,
    arg_items: &[MbValue],
) {
    let slots: &[&str] = match type_name {
        "UnicodeEncodeError" | "UnicodeDecodeError" => &["encoding", "object", "start", "end", "reason"],
        "UnicodeTranslateError" => &["object", "start", "end", "reason"],
        _ => return,
    };
    for (idx, key) in slots.iter().enumerate() {
        let value = arg_items.get(idx).copied().unwrap_or_else(MbValue::none);
        insert_borrowed_field(fields, key, value);
    }
}

fn populate_exception_fields(
    fields: &mut FxHashMap<String, MbValue>,
    type_name: &str,
    arg_items: &[MbValue],
    include_chain_fields: bool,
) -> bool {
    let msg = if let Some(first) = arg_items.first() {
        extract_str(*first).unwrap_or_default()
    } else {
        String::new()
    };
    fields.insert("message".to_string(), new_str_value(msg));
    fields.insert("__type__".to_string(), new_str_value(type_name.to_string()));
    if include_chain_fields {
        fields.insert("__cause__".to_string(), MbValue::none());
        fields.insert("__context__".to_string(), MbValue::none());
        fields.insert("__suppress_context__".to_string(), MbValue::from_bool(false));
    }
    if type_name == "StopIteration" {
        let value_val = arg_items.first().copied().unwrap_or_else(MbValue::none);
        insert_borrowed_field(fields, "value", value_val);
    }
    if type_name == "AttributeError" {
        fields.insert("name".to_string(), MbValue::none());
        fields.insert("obj".to_string(), MbValue::none());
    } else if type_name == "NameError" {
        fields.insert("name".to_string(), MbValue::none());
    } else if type_name == "ImportError" || type_name == "ModuleNotFoundError" {
        populate_import_error_fields(fields, arg_items);
    } else if type_name == "SyntaxError" || type_name == "IndentationError" || type_name == "TabError" {
        if !populate_syntax_error_fields(fields, arg_items) {
            return false;
        }
    } else if matches!(type_name, "UnicodeEncodeError" | "UnicodeDecodeError" | "UnicodeTranslateError") {
        populate_unicode_error_fields(fields, type_name, arg_items);
    }
    let args_tuple = MbValue::from_ptr(MbObject::new_tuple_borrowed(arg_items.to_vec()));
    fields.insert("args".to_string(), args_tuple);
    true
}

/// Create a new exception instance preserving all constructor arguments in the `args` tuple.
/// Used for `ExcType(arg1, arg2, ...)` expressions so `e.args` returns all arguments.
pub fn mb_exception_new_with_args(exc_type: MbValue, args_list: MbValue) -> MbValue {
    let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
    let arg_items = list_items(args_list);
    // Build instance fields directly (avoids circular dep with class.rs)
    let mut fields = FxHashMap::default();
    if !populate_exception_fields(&mut fields, &type_name, &arg_items, true) {
        return MbValue::none();
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: type_name,
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

pub fn mb_exception_init_instance(instance: MbValue, args_list: MbValue) -> MbValue {
    let type_name = instance
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Exception".to_string());
    let arg_items = list_items(args_list);
    let mut new_fields = FxHashMap::default();
    if !populate_exception_fields(&mut new_fields, &type_name, &arg_items, false) {
        return MbValue::none();
    }
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut guard = fields.write().unwrap();
                for (key, value) in new_fields {
                    let old = guard.insert(key, value);
                    if let Some(prev) = old {
                        super::rc::release_if_ptr(prev);
                    }
                }
            }
        }
    }
    MbValue::none()
}

fn dict_get_str(dict: MbValue, key: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn dict_string_keys(dict: MbValue) -> Vec<String> {
    dict.as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                Some(
                    lock.read()
                        .unwrap()
                        .keys()
                        .filter_map(|key| match key {
                            super::dict_ops::DictKey::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .collect(),
                )
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn set_exception_field(instance: MbValue, key: &str, value: MbValue) {
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::rc::retain_if_ptr(value);
                let old = fields.write().unwrap().insert(key.to_string(), value);
                if let Some(prev) = old {
                    super::rc::release_if_ptr(prev);
                }
            }
        }
    }
}

fn instance_field(instance: MbValue, key: &str) -> Option<MbValue> {
    instance.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn stringish_value(value: MbValue) -> String {
    extract_str(value).unwrap_or_else(|| super::string_ops::value_to_string(value))
}

fn unicode_char_escape(ch: char) -> String {
    let cp = ch as u32;
    if cp <= 0xff {
        format!("\\x{cp:02x}")
    } else if cp <= 0xffff {
        format!("\\u{cp:04x}")
    } else {
        format!("\\U{cp:08x}")
    }
}

fn unicode_char_at(value: MbValue, index: i64) -> Option<char> {
    if index < 0 {
        return None;
    }
    extract_str(value)?.chars().nth(index as usize)
}

fn unicode_range_fields(instance: MbValue) -> Option<(i64, i64, String)> {
    let start = value_as_i64(instance_field(instance, "start")?)?;
    let end = value_as_i64(instance_field(instance, "end")?)?;
    let reason = stringish_value(instance_field(instance, "reason").unwrap_or_else(MbValue::none));
    Some((start, end, reason))
}

pub fn unicode_error_str(class_name: &str, instance: MbValue) -> Option<String> {
    if !matches!(class_name, "UnicodeEncodeError" | "UnicodeDecodeError" | "UnicodeTranslateError") {
        return None;
    }
    let object = instance_field(instance, "object").unwrap_or_else(MbValue::none);
    if object.is_none() {
        return Some(String::new());
    }
    let (start, end, reason) = match unicode_range_fields(instance) {
        Some(parts) => parts,
        None => return Some(String::new()),
    };
    let range_end = end.saturating_sub(1);
    match class_name {
        "UnicodeEncodeError" => {
            let encoding = stringish_value(instance_field(instance, "encoding").unwrap_or_else(MbValue::none));
            if end == start + 1 {
                let ch = unicode_char_at(object, start).map(unicode_char_escape).unwrap_or_default();
                Some(format!("'{encoding}' codec can't encode character '{ch}' in position {start}: {reason}"))
            } else {
                Some(format!("'{encoding}' codec can't encode characters in position {start}-{range_end}: {reason}"))
            }
        }
        "UnicodeDecodeError" => {
            let encoding = stringish_value(instance_field(instance, "encoding").unwrap_or_else(MbValue::none));
            let bytes = match value_as_bytes(object) {
                Some(bytes) => bytes,
                None => return Some(String::new()),
            };
            if end == start + 1 {
                let byte = bytes.get(start as usize).copied().unwrap_or_default();
                Some(format!("'{encoding}' codec can't decode byte 0x{byte:02x} in position {start}: {reason}"))
            } else {
                Some(format!("'{encoding}' codec can't decode bytes in position {start}-{range_end}: {reason}"))
            }
        }
        "UnicodeTranslateError" => {
            if end == start + 1 {
                let ch = unicode_char_at(object, start).map(unicode_char_escape).unwrap_or_default();
                Some(format!("can't translate character '{ch}' in position {start}: {reason}"))
            } else {
                Some(format!("can't translate characters in position {start}-{range_end}: {reason}"))
            }
        }
        _ => None,
    }
}

/// Create an exception instance from positional args plus keyword metadata.
/// Consumes CPython's small set of built-in exception keyword-only attrs.
pub fn mb_exception_new_with_args_and_kwargs(
    exc_type: MbValue,
    args_list: MbValue,
    kwargs_dict: MbValue,
) -> MbValue {
    let type_name = super::class::resolve_class_name(exc_type)
        .or_else(|| extract_str(exc_type))
        .unwrap_or_else(|| "Exception".to_string());
    let allowed: &[&str] = match type_name.as_str() {
        "AttributeError" => &["name", "obj"],
        "NameError" | "UnboundLocalError" => &["name"],
        "ImportError" | "ModuleNotFoundError" => &["name", "path"],
        _ => &[],
    };
    for key in dict_string_keys(kwargs_dict) {
        if !allowed.contains(&key.as_str()) {
            raise_type_error_message(format!("'{key}' is an invalid keyword argument for {type_name}()"));
            return MbValue::none();
        }
    }
    let instance = mb_exception_new_with_args(
        MbValue::from_ptr(MbObject::new_str(type_name.clone())),
        args_list,
    );
    if instance.is_none() {
        return instance;
    }
    if type_name == "AttributeError" {
        if let Some(name) = dict_get_str(kwargs_dict, "name") {
            set_exception_field(instance, "name", name);
        }
        if let Some(obj) = dict_get_str(kwargs_dict, "obj") {
            set_exception_field(instance, "obj", obj);
        }
    } else if type_name == "NameError" || type_name == "UnboundLocalError" {
        if let Some(name) = dict_get_str(kwargs_dict, "name") {
            set_exception_field(instance, "name", name);
        }
    } else if type_name == "ImportError" || type_name == "ModuleNotFoundError" {
        if let Some(name) = dict_get_str(kwargs_dict, "name") {
            set_exception_field(instance, "name", name);
        }
        if let Some(path) = dict_get_str(kwargs_dict, "path") {
            set_exception_field(instance, "path", path);
        }
    }
    instance
}

pub fn mb_attribute_error_with_name_obj(msg: &str, name: &str, obj: MbValue) -> MbValue {
    let exc_type = MbValue::from_ptr(MbObject::new_str("AttributeError".to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
        MbObject::new_str(msg.to_string()),
    )]));
    let instance = mb_exception_new_with_args(exc_type, args);
    set_exception_field(
        instance,
        "name",
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    set_exception_field(instance, "obj", obj);
    instance
}

pub fn mb_name_error_with_name(name: MbValue) -> MbValue {
    let name_s = extract_str(name).unwrap_or_default();
    let exc_type = MbValue::from_ptr(MbObject::new_str("NameError".to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
        MbObject::new_str(format!("name '{name_s}' is not defined")),
    )]));
    let instance = mb_exception_new_with_args(exc_type, args);
    set_exception_field(
        instance,
        "name",
        MbValue::from_ptr(MbObject::new_str(name_s)),
    );
    instance
}

pub fn mb_unbound_local_error_value(name: MbValue) -> MbValue {
    let name_s = extract_str(name).unwrap_or_default();
    mb_raise(
        MbValue::from_ptr(MbObject::new_str("UnboundLocalError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "cannot access local variable '{name_s}' where it is not associated with a value"
        ))),
    );
    MbValue::none()
}

/// Convert a MbException to a MbValue (stored as an Instance object).
fn store_exception_as_value(exc: MbException) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "message".to_string(),
        MbValue::from_ptr(MbObject::new_str(exc.message.clone())),
    );
    fields.insert(
        "__type__".to_string(),
        MbValue::from_ptr(MbObject::new_str(exc.exc_type.clone())),
    );
    // Exception chaining fields
    fields.insert(
        "__cause__".to_string(),
        match exc.cause {
            Some(cause) => store_exception_as_value(*cause),
            None => MbValue::none(),
        },
    );
    fields.insert(
        "__context__".to_string(),
        match exc.context {
            Some(ctx) => store_exception_as_value(*ctx),
            None => MbValue::none(),
        },
    );
    fields.insert(
        "__suppress_context__".to_string(),
        MbValue::from_bool(exc.suppress_context),
    );
    // args = tuple of constructor arguments (CPython: e.args)
    // When created from MbException we only have the single message string.
    let args_items: Vec<MbValue> = if exc.message.is_empty() {
        Vec::new()
    } else {
        vec![MbValue::from_ptr(MbObject::new_str(exc.message.clone()))]
    };
    fields.insert(
        "args".to_string(),
        MbValue::from_ptr(MbObject::new_tuple(args_items)),
    );
    if exc.exc_type == "AttributeError" {
        fields.insert("name".to_string(), MbValue::none());
        fields.insert("obj".to_string(), MbValue::none());
    } else if exc.exc_type == "NameError" {
        fields.insert("name".to_string(), MbValue::none());
    }
    // StopIteration.value: generator return value takes priority; for
    // explicit `raise StopIteration(x)` the constructor argument is kept
    // in `exc.message` and surfaced here as a string fallback.
    if exc.exc_type == "StopIteration" {
        let stop_val = super::generator::mb_generator_stop_value();
        let value_val = if stop_val.is_none() && !exc.message.is_empty() {
            MbValue::from_ptr(MbObject::new_str(exc.message.clone()))
        } else {
            stop_val
        };
        fields.insert("value".to_string(), value_val);
    }
    if !exc.traceback.is_empty() {
        fields.insert(
            "__traceback__".to_string(),
            super::stdlib::traceback_mod::make_tb_from_traceback_entries(&exc.traceback),
        );
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: exc.exc_type,
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Extract string from MbValue.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Display text for a raise-site message operand: CPython stringifies a
/// non-str single arg (`str(ValueError(3)) == "3"`; `SystemExit(3)`
/// carries its exit status here), so int/float operands must not vanish.
fn message_display(message: MbValue) -> String {
    extract_str(message)
        .or_else(|| message.as_int().map(|i| i.to_string()))
        .or_else(|| message.as_float().map(|f| f.to_string()))
        .unwrap_or_default()
}

// ── Raise / Catch ──

/// Raise an exception. Sets the thread-local exception state.
pub fn mb_raise(exc_type: MbValue, message: MbValue) {
    let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
    let msg = message_display(message);
    super::class::clear_last_raised_instance();
    // Also signal StopIteration via the iterator flag for user-defined __next__
    if type_name == "StopIteration" {
        super::iter::signal_stop_iteration();
    }
    let exc = MbException::new(&type_name, &msg);
    CURRENT_EXCEPTION.with(|cell| {
        *cell.borrow_mut() = Some(exc);
    });
}

/// Raise with chaining: `raise X from Y`.
/// Always sets __suppress_context__ = True (per Python semantics).
/// If cause is None, __cause__ remains None.
/// Convert an exception MbValue (an Instance carrying message/__cause__/
/// __context__ fields) into an owned MbException, preserving its full
/// cause/context chain. `raise X from Y` must keep Y's *own* __cause__ so a
/// deep chain (`KeyError`←`LookupError`←`ValueError`) walks all the way down;
/// rebuilding the cause from just its type+message dropped the inner links.
/// Bounded depth guards a cyclic chain.
fn mbvalue_to_mbexception(exc: MbValue, depth: u32) -> Option<MbException> {
    if depth > 64 || exc.is_none() {
        return None;
    }
    let ty = get_exception_type(exc)?;
    let msg = get_exception_message(exc).unwrap_or_default();
    let mut out = MbException::new(&ty, &msg);
    if let Some(ptr) = exc.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let (cause_v, ctx_v, suppress) = {
                    let f = fields.read().unwrap();
                    (
                        f.get("__cause__").copied().unwrap_or_else(MbValue::none),
                        f.get("__context__").copied().unwrap_or_else(MbValue::none),
                        f.get("__suppress_context__")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    )
                };
                out.suppress_context = suppress;
                out.cause = mbvalue_to_mbexception(cause_v, depth + 1).map(Box::new);
                out.context = mbvalue_to_mbexception(ctx_v, depth + 1).map(Box::new);
            }
        }
    }
    Some(out)
}

pub fn mb_raise_from(exc_type: MbValue, message: MbValue, cause: MbValue) {
    let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
    let msg = message_display(message);
    super::class::clear_last_raised_instance();
    let mut exc = MbException::new(&type_name, &msg);
    // `raise X from Y` always sets suppress_context = True
    exc.suppress_context = true;
    if !cause.is_none() {
        // Preserve the cause's own chain (its __cause__/__context__) so deep
        // `raise ... from ...` ladders walk correctly.
        exc.cause = mbvalue_to_mbexception(cause, 0).map(Box::new);
    }
    CURRENT_EXCEPTION.with(|cell| {
        *cell.borrow_mut() = Some(exc);
    });
}

/// Raise with implicit chaining: sets __context__ to the active exception.
/// Called when a raise occurs inside an except handler body.
pub fn mb_raise_with_context(exc_type: MbValue, message: MbValue, context: MbValue) {
    let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
    let msg = message_display(message);
    super::class::clear_last_raised_instance();
    let mut exc = MbException::new(&type_name, &msg);
    if !context.is_none() {
        exc.context = mbvalue_to_mbexception(context, 0).map(Box::new);
    }
    CURRENT_EXCEPTION.with(|cell| {
        *cell.borrow_mut() = Some(exc);
    });
}

/// Raise with both explicit cause and implicit context.
/// Always sets __suppress_context__ = True (per Python `raise from` semantics).
pub fn mb_raise_from_with_context(
    exc_type: MbValue,
    message: MbValue,
    cause: MbValue,
    context: MbValue,
) {
    let type_name = extract_str(exc_type).unwrap_or_else(|| "Exception".to_string());
    let msg = message_display(message);
    super::class::clear_last_raised_instance();
    let mut exc = MbException::new(&type_name, &msg);
    // `raise X from Y` always sets suppress_context = True
    exc.suppress_context = true;
    if !cause.is_none() {
        exc.cause = mbvalue_to_mbexception(cause, 0).map(Box::new);
    }
    if !context.is_none() {
        exc.context = mbvalue_to_mbexception(context, 0).map(Box::new);
    }
    CURRENT_EXCEPTION.with(|cell| {
        *cell.borrow_mut() = Some(exc);
    });
}

/// Re-raise an exception value (put it back into thread-local state).
/// Used when no except handler matched, and for except* rest-group propagation.
/// Delegates to mb_raise_instance so the full instance (with custom fields like
/// ExceptionGroup.exceptions) is preserved for the next mb_catch_exception_instance.
pub fn mb_reraise(exc: MbValue) {
    super::class::mb_raise_instance(exc);
}

/// Re-raise the currently handled exception for a bare `raise` that executes
/// outside the lexical except body that originally caught it. Generator bodies
/// can resume while the caller is inside an except handler, so their bare
/// `raise` must consult the runtime handled-exception slot rather than trap.
pub fn mb_reraise_handled() {
    let caught = super::class::last_caught_exception_value();
    if !caught.is_none() {
        super::class::mb_raise_instance(caught);
        unsafe {
            super::rc::release_if_ptr(caught);
        }
        return;
    }

    if let Some(exc) = LAST_HANDLED_EXCEPTION.with(|cell| cell.borrow().clone()) {
        set_current_exception(exc);
        return;
    }

    mb_raise(
        MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "No active exception to reraise".to_string(),
        )),
    );
}

/// Check if there's a pending exception.
pub fn mb_has_exception() -> MbValue {
    CURRENT_EXCEPTION.with(|cell| MbValue::from_bool(cell.borrow().is_some()))
}

/// Take the current exception (clearing the pending state) and return a
/// CPython-style `Traceback ... ExcType: message` string. Returns None if
/// no exception is pending. Used by the driver to report uncaught
/// exceptions at the end of module execution.
pub fn mb_take_uncaught_traceback() -> Option<String> {
    CURRENT_EXCEPTION.with(|cell| {
        cell.borrow_mut().take().map(|exc| {
            let mut out = String::from("Traceback (most recent call last):\n");
            out.push_str("  File \"<module>\"\n");
            if exc.message.is_empty() {
                out.push_str(&format!("{}", exc.exc_type));
            } else {
                out.push_str(&format!("{}: {}", exc.exc_type, exc.message));
            }
            out
        })
    })
}

/// Get the current exception as a MbValue, clearing the pending state.
/// Also records the exception in LAST_HANDLED_EXCEPTION so that
/// `traceback.format_exc()` and `sys.exc_info()` can report it after the
/// except handler has consumed the pending slot.
pub fn mb_catch_exception() -> MbValue {
    CURRENT_EXCEPTION.with(|cell| match cell.borrow_mut().take() {
        Some(exc) => {
            LAST_HANDLED_EXCEPTION.with(|h| {
                *h.borrow_mut() = Some(exc.clone());
            });
            let val = store_exception_as_value(exc);
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            val
        }
        None => MbValue::none(),
    })
}

/// Check if the current exception matches a given type.
pub fn mb_exception_matches(exc: MbValue, exc_type: MbValue) -> MbValue {
    let actual_type = get_exception_type(exc).unwrap_or_default();

    let mut targets = Vec::new();
    if collect_matcher_targets(exc_type, &mut targets).is_err() {
        mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "catching classes that do not inherit from BaseException is not allowed"
                    .to_string(),
            )),
        );
        return MbValue::from_bool(false);
    }

    let matches = targets
        .iter()
        .any(|target| actual_type == *target || is_subclass_of(&actual_type, target));
    MbValue::from_bool(matches)
}

fn collect_matcher_targets(exc_type: MbValue, out: &mut Vec<String>) -> Result<(), ()> {
    if let Some(ptr) = exc_type.as_ptr() {
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                for item in items {
                    collect_matcher_targets(*item, out)?;
                }
                return Ok(());
            }
        }
    }
    let Some(target) = super::class::resolve_class_name(exc_type) else {
        return Err(());
    };
    if !is_subclass_of(&target, "BaseException") {
        return Err(());
    }
    out.push(target);
    Ok(())
}

/// Clear the current exception (used after successful except handling).
pub fn mb_clear_exception() {
    super::class::clear_last_raised_instance();
    CURRENT_EXCEPTION.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Set the current exception directly (for use by class.rs raise_instance).
pub fn set_current_exception(exc: MbException) {
    super::class::clear_last_raised_instance();
    CURRENT_EXCEPTION.with(|cell| {
        *cell.borrow_mut() = Some(exc);
    });
}

pub fn set_current_traceback(entries: Vec<(String, u32, String)>) {
    CURRENT_EXCEPTION.with(|cell| {
        if let Some(exc) = cell.borrow_mut().as_mut() {
            exc.traceback = entries;
        }
    });
}

/// Clear the current exception state (for use by class.rs catch_exception_instance).
pub fn clear_current_exception() {
    super::class::clear_last_raised_instance();
    CURRENT_EXCEPTION.with(|cell| {
        if cell.borrow().is_some() {
            *cell.borrow_mut() = None;
        }
    });
}

/// Peek the pending exception's type name without clearing it. Returns `None`
/// when no exception is pending. Used by eager iterable drains (e.g.
/// `itertools.cycle`) to tell a real raise apart from a benign StopIteration.
pub fn current_exception_type() -> Option<String> {
    CURRENT_EXCEPTION.with(|cell| cell.borrow().as_ref().map(|e| e.exc_type.clone()))
}

// ── Exception Introspection ──

/// Get the type name of an exception value.
fn get_exception_type(exc: MbValue) -> Option<String> {
    exc.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            // `raise SomeType` where SomeType is a type OBJECT (Instance
            // class_name "type" carrying __name__): the exception's type is
            // the named class, not the literal string "type".
            if class_name == "type" {
                if let Some(n) = fields
                    .read()
                    .unwrap()
                    .get("__name__")
                    .and_then(|v| extract_str(*v))
                {
                    return Some(n);
                }
            }
            Some(class_name.clone())
        } else {
            None
        }
    })
}

/// Public version for use by class.rs.
pub fn get_exception_type_pub(exc: MbValue) -> Option<String> {
    get_exception_type(exc)
}

/// Get the message of an exception value.
fn get_exception_message(exc: MbValue) -> Option<String> {
    exc.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let fields = fields.read().unwrap();
            fields.get("message").and_then(|v| {
                // CPython str(ValueError(3)) == "3": a non-str single arg
                // stringifies rather than vanishing (SystemExit(3) carries
                // its exit status here).
                extract_str(*v)
                    .or_else(|| v.as_int().map(|i| i.to_string()))
                    .or_else(|| v.as_float().map(|f| f.to_string()))
            })
        } else {
            None
        }
    })
}

/// Public version for use by class.rs.
pub fn get_exception_message_pub(exc: MbValue) -> Option<String> {
    get_exception_message(exc)
}

/// Names of built-in exception classes (subclasses of BaseException). Used by
/// `is_subclass_of` to answer "is `child` a built-in exception?" without an
/// unconditional yes for parent=Exception/BaseException, which previously
/// caused class.rs to treat every no-`__init__` user class as an exception
/// and inject a bogus `args` field. (#1551)
pub(crate) fn is_builtin_exception_name(name: &str) -> bool {
    matches!(
        name,
        "BaseException" | "Exception"
        | "ArithmeticError" | "ZeroDivisionError" | "OverflowError" | "FloatingPointError"
        | "LookupError" | "IndexError" | "KeyError"
        | "UnicodeError" | "UnicodeDecodeError" | "UnicodeEncodeError" | "UnicodeTranslateError"
        | "ValueError" | "JSONDecodeError"
        | "OSError" | "IOError"
        | "FileNotFoundError" | "PermissionError" | "IsADirectoryError"
        | "NotADirectoryError" | "FileExistsError" | "ConnectionError"
        | "TimeoutError" | "BrokenPipeError" | "ConnectionAbortedError"
        | "ConnectionRefusedError" | "ConnectionResetError"
        | "BlockingIOError" | "ChildProcessError"
        | "InterruptedError" | "ProcessLookupError"
        | "RuntimeError" | "NotImplementedError" | "RecursionError"
        | "NameError" | "UnboundLocalError"
        | "ImportError" | "ModuleNotFoundError"
        | "SyntaxError" | "IndentationError" | "TabError"
        | "AttributeError" | "TypeError" | "AssertionError"
        | "StopIteration" | "StopAsyncIteration" | "GeneratorExit"
        | "SystemExit" | "KeyboardInterrupt"
        | "MemoryError" | "BufferError" | "EOFError"
        | "ReferenceError" | "SystemError"
        | "ExceptionGroup" | "BaseExceptionGroup"
        | "Warning" | "DeprecationWarning" | "RuntimeWarning" | "UserWarning"
        | "SyntaxWarning" | "FutureWarning" | "PendingDeprecationWarning"
        | "UnicodeWarning" | "BytesWarning" | "ResourceWarning"
        | "ImportWarning" | "EncodingWarning" | "InvalidTZPathWarning"
        // subprocess exception tree (subprocess_mod.rs raises these). All
        // ultimately derive from Exception.
        | "SubprocessError" | "CalledProcessError" | "TimeoutExpired"
        // configparser exception tree (configparser_mod.rs raises these). The
        // base `Error` derives from Exception; the rest derive from it.
        | "NoSectionError" | "NoOptionError"
        | "DuplicateSectionError" | "DuplicateOptionError"
        | "InterpolationError" | "InterpolationMissingOptionError"
        | "InterpolationSyntaxError" | "InterpolationDepthError"
        | "ParsingError" | "MissingSectionHeaderError"
        // binascii exception tree (binascii_mod.rs raises these). `Error`
        // derives from ValueError; `Incomplete` derives from Exception.
        | "binascii.Error" | "binascii.Incomplete"
        // statistics.StatisticsError (statistics_mod.rs raises this). It
        // derives from ValueError (see the ValueError arm of is_subclass_of),
        // so `except ValueError` / `except Exception` must both catch it.
        | "StatisticsError"
        // calendar.IllegalMonthError / IllegalWeekdayError (calendar_mod.rs
        // raises these). Both derive from ValueError (CPython 3.12).
        | "IllegalMonthError" | "IllegalWeekdayError"
        // dataclasses.FrozenInstanceError (class.rs mb_setattr raises this on
        // frozen-dataclass assignment). Derives from AttributeError.
        | "FrozenInstanceError"
    )
}

fn builtin_exception_is_exception_subclass(name: &str) -> bool {
    if matches!(
        name,
        "BaseException"
            | "SystemExit"
            | "KeyboardInterrupt"
            | "GeneratorExit"
            | "BaseExceptionGroup"
    ) {
        return false;
    }
    is_builtin_exception_name(name)
}

/// Simplified exception hierarchy check.
pub fn is_subclass_of(child: &str, parent: &str) -> bool {
    if parent == "Exception" {
        if builtin_exception_is_exception_subclass(child) {
            return true;
        }
        if child == "Error" || is_configparser_error_subclass(child) {
            return true;
        }
        if super::class::class_mro_any(child, builtin_exception_is_exception_subclass) {
            return true;
        }
        return super::class::check_class_hierarchy(child, parent);
    }
    if parent == "BaseException" {
        if is_builtin_exception_name(child) {
            return true;
        }
        // configparser base `Error` (and its subclasses) derive from Exception.
        // `Error` is intentionally not listed in is_builtin_exception_name
        // because the bare name is generic, but the whole configparser tree is
        // a subclass of Exception/BaseException.
        if child == "Error" || is_configparser_error_subclass(child) {
            return true;
        }
        // User-class case: walk MRO; any built-in exception ancestor counts.
        // (`check_class_hierarchy` only matches when the literal `parent`
        // string is in MRO — but built-in exceptions like `ValueError`
        // never carry "Exception" in their MRO, so we need this scan.)
        if super::class::class_mro_any(child, |c| is_builtin_exception_name(c)) {
            return true;
        }
        return super::class::check_class_hierarchy(child, parent);
    }
    // Check user-defined class hierarchy via CLASS_REGISTRY
    let found_in_registry = super::class::check_class_hierarchy(child, parent);
    if found_in_registry {
        return true;
    }
    // Built-in exception hierarchy
    match parent {
        "ArithmeticError" => matches!(
            child,
            "ZeroDivisionError" | "OverflowError" | "FloatingPointError"
        ),
        "LookupError" => matches!(child, "IndexError" | "KeyError" | "ZoneInfoNotFoundError"),
        // zoneinfo.ZoneInfoNotFoundError derives from KeyError.
        "KeyError" => matches!(child, "ZoneInfoNotFoundError"),
        "UnicodeError" => matches!(
            child,
            "UnicodeDecodeError" | "UnicodeEncodeError" | "UnicodeTranslateError"
        ),
        "ValueError" => matches!(
            child,
            "UnicodeDecodeError" | "UnicodeEncodeError" | "UnicodeTranslateError"
            | "UnicodeError" | "JSONDecodeError"
            // binascii.Error subclasses ValueError (CPython 3.12).
            | "binascii.Error"
            // statistics.StatisticsError subclasses ValueError (CPython 3.12).
            | "StatisticsError"
            // calendar.IllegalMonthError / IllegalWeekdayError subclass
            // ValueError (CPython 3.12).
            | "IllegalMonthError" | "IllegalWeekdayError"
        ),
        "OSError" => matches!(
            child,
            "FileNotFoundError"
                | "PermissionError"
                | "IsADirectoryError"
                | "NotADirectoryError"
                | "FileExistsError"
                | "ConnectionError"
                | "TimeoutError"
                | "BrokenPipeError"
                | "ConnectionAbortedError"
                | "ConnectionRefusedError"
                | "ConnectionResetError"
                | "BlockingIOError"
                | "ChildProcessError"
                | "InterruptedError"
                | "ProcessLookupError"
        ),
        "ConnectionError" => matches!(
            child,
            "BrokenPipeError"
                | "ConnectionAbortedError"
                | "ConnectionRefusedError"
                | "ConnectionResetError"
        ),
        "RuntimeError" => matches!(child, "NotImplementedError" | "RecursionError"),
        "NameError" => matches!(child, "UnboundLocalError"),
        "ImportError" => matches!(child, "ModuleNotFoundError"),
        // dataclasses.FrozenInstanceError subclasses AttributeError (PEP 557).
        "AttributeError" => matches!(child, "FrozenInstanceError"),
        // PEP 654: ExceptionGroup derives from both BaseExceptionGroup and Exception.
        "BaseExceptionGroup" => matches!(child, "ExceptionGroup"),
        "SyntaxError" => matches!(child, "IndentationError" | "TabError"),
        "IndentationError" => matches!(child, "TabError"),
        "Warning" => matches!(
            child,
            "DeprecationWarning"
                | "RuntimeWarning"
                | "UserWarning"
                | "SyntaxWarning"
                | "FutureWarning"
                | "PendingDeprecationWarning"
                | "UnicodeWarning"
                | "BytesWarning"
                | "ResourceWarning"
                | "ImportWarning"
                | "EncodingWarning"
                | "InvalidTZPathWarning"
        ),
        // zoneinfo.InvalidTZPathWarning derives from RuntimeWarning.
        "RuntimeWarning" => matches!(child, "InvalidTZPathWarning"),
        // subprocess: SubprocessError ⊂ Exception; CalledProcessError and
        // TimeoutExpired ⊂ SubprocessError (Python 3.12).
        "SubprocessError" => matches!(child, "CalledProcessError" | "TimeoutExpired"),
        // configparser: every leaf derives (directly or transitively) from the
        // package base `Error`. `except configparser.Error` must catch them all.
        "Error" => is_configparser_error_subclass(child),
        // configparser interpolation subtree:
        // InterpolationMissingOptionError / InterpolationSyntaxError /
        // InterpolationDepthError ⊂ InterpolationError ⊂ Error.
        "InterpolationError" => matches!(
            child,
            "InterpolationMissingOptionError"
                | "InterpolationSyntaxError"
                | "InterpolationDepthError"
        ),
        // configparser: MissingSectionHeaderError ⊂ ParsingError ⊂ Error.
        "ParsingError" => matches!(child, "MissingSectionHeaderError"),
        _ => false,
    }
}

/// True if `child` is a configparser exception class that derives, directly or
/// transitively, from configparser's base `Error`. Mirrors the Python 3.12
/// configparser hierarchy:
///
/// ```text
/// Error
/// ├── NoSectionError
/// ├── NoOptionError
/// ├── DuplicateSectionError
/// ├── DuplicateOptionError
/// ├── InterpolationError
/// │   ├── InterpolationMissingOptionError
/// │   ├── InterpolationSyntaxError
/// │   └── InterpolationDepthError
/// └── ParsingError
///     └── MissingSectionHeaderError
/// ```
fn is_configparser_error_subclass(child: &str) -> bool {
    matches!(
        child,
        "NoSectionError"
            | "NoOptionError"
            | "DuplicateSectionError"
            | "DuplicateOptionError"
            | "InterpolationError"
            | "InterpolationMissingOptionError"
            | "InterpolationSyntaxError"
            | "InterpolationDepthError"
            | "ParsingError"
            | "MissingSectionHeaderError"
    )
}

// ── Try/Except Frame Management ──

/// Push an exception handler frame (called at try block entry).
pub fn mb_push_handler(catch_all: bool) {
    EXCEPTION_HANDLERS.with(|cell| {
        cell.borrow_mut().push(ExceptionHandler {
            catch_types: if catch_all { Vec::new() } else { Vec::new() },
            has_finally: false,
        });
    });
}

/// Pop an exception handler frame (called at try block exit).
pub fn mb_pop_handler() {
    EXCEPTION_HANDLERS.with(|cell| {
        cell.borrow_mut().pop();
    });
}

// ── Built-in Exception Constructors ──

pub fn mb_type_error(msg: &str) -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    )
}

/// Raise `TypeError(msg)` for a statically-detected call-binding violation
/// (too many positional args / duplicate argument / missing required
/// keyword-only argument) and return None. Emitted directly at the call site
/// by the ast_to_hir arg-binding validator, replacing the would-be call so the
/// enclosing `try` sees a genuine runtime exception.
pub fn mb_arg_bind_error(msg: MbValue) -> MbValue {
    mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        msg,
    );
    MbValue::none()
}

pub fn mb_value_error(msg: &str) -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    )
}

pub fn mb_index_error(msg: &str) -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    )
}

pub fn mb_key_error(msg: &str) -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    )
}

pub fn mb_attribute_error(msg: &str) -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    )
}

pub fn mb_stop_iteration() -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("StopIteration".to_string())),
        MbValue::from_ptr(MbObject::new_str(String::new())),
    )
}

pub fn mb_runtime_error(msg: &str) -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    )
}

pub fn mb_zero_division_error() -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
        MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
    )
}

pub fn mb_import_error(msg: &str) -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("ImportError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    )
}

// ── ExceptionGroup (#410) ──

/// Create an ExceptionGroup: ExceptionGroup(message, [exc1, exc2, ...])
/// The `exceptions` field is always stored as a tuple (matching CPython).
pub fn mb_exception_group_new(message: MbValue, exceptions: MbValue) -> MbValue {
    mb_exception_group_new_as(message, exceptions, "ExceptionGroup")
}

/// Like `mb_exception_group_new` but stamps a specific class name (so a
/// `BaseExceptionGroup` / subclass keeps its identity in isinstance and repr).
pub fn mb_exception_group_new_as(
    message: MbValue,
    exceptions: MbValue,
    class_name: &str,
) -> MbValue {
    let msg = message_display(message);
    // Convert exceptions list to tuple (CPython stores as tuple)
    let exc_tuple = if let Some(ptr) = exceptions.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    MbValue::from_ptr(MbObject::new_tuple_borrowed(items.to_vec()))
                }
                ObjData::Tuple(_) => exceptions, // already a tuple
                _ => exceptions,                 // fallback
            }
        }
    } else {
        exceptions
    };
    let mut fields = FxHashMap::default();
    fields.insert(
        "message".to_string(),
        MbValue::from_ptr(MbObject::new_str(msg.clone())),
    );
    fields.insert(
        "__type__".to_string(),
        MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
    );
    fields.insert("exceptions".to_string(), exc_tuple);
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// PEP 654 constructor narrowing: `BaseExceptionGroup(msg, excs)` returns an
/// `ExceptionGroup` when every member is an `Exception` (not a bare
/// `BaseException` such as KeyboardInterrupt); otherwise it stays a
/// `BaseExceptionGroup`. `ExceptionGroup` and user subclasses keep their name.
/// True if `m` is a bare `BaseException` member (NOT an `Exception` subclass),
/// which cannot be nested in an `ExceptionGroup`. `is_subclass_of(_, "Exception")`
/// over-reports (early-return on any builtin exc name), so test the known
/// BaseException-only roots; `BaseExceptionGroup` itself derives from
/// BaseException, not Exception.
pub fn eg_member_is_bare_base(m: MbValue) -> bool {
    m.as_ptr()
        .map(|p| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*p).data {
                matches!(
                    class_name.as_str(),
                    "BaseException" | "KeyboardInterrupt" | "SystemExit"
                        | "GeneratorExit" | "BaseExceptionGroup"
                )
            } else {
                false
            }
        })
        .unwrap_or(false)
}

fn narrow_eg_class_name(cn: &str, members: &[MbValue]) -> String {
    if cn != "BaseExceptionGroup" {
        return cn.to_string();
    }
    if members.iter().all(|m| !eg_member_is_bare_base(*m)) {
        "ExceptionGroup".to_string()
    } else {
        "BaseExceptionGroup".to_string()
    }
}

/// True if `v` is an exception *instance* (an Instance whose class is in the
/// exception tree) — not an exception *type* (a name-string) nor a plain value.
fn is_exception_instance(v: MbValue) -> bool {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return is_builtin_exception_name(class_name)
                    || super::class::class_mro_any(class_name, is_builtin_exception_name)
                    || is_subclass_of(class_name, "BaseException");
            }
        }
    }
    false
}

/// `(Base)ExceptionGroup(message, exceptions)` constructor with full CPython
/// argument validation. Returns the group, or None with a pending TypeError/
/// ValueError on a bad argument. `args_list` is the list of positional args;
/// `class_name` is "ExceptionGroup" / "BaseExceptionGroup".
pub fn mb_exception_group_construct(args_list: MbValue, class_name: MbValue) -> MbValue {
    let cn = extract_str(class_name).unwrap_or_else(|| "ExceptionGroup".to_string());
    let items = super::builtins::extract_items(args_list);
    // Exactly two positional arguments.
    if items.len() != 2 {
        mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "{cn}() takes exactly 2 arguments ({} given)",
                items.len()
            ))),
        );
        return MbValue::none();
    }
    let message = items[0];
    let exceptions = items[1];
    // 1. message must be a str.
    let msg_is_str = message
        .as_ptr()
        .map(|p| matches!(unsafe { &(*p).data }, ObjData::Str(_)))
        .unwrap_or(false);
    if !msg_is_str {
        mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "argument 1 must be str, not {}",
                super::builtins::value_type_name(message)
            ))),
        );
        return MbValue::none();
    }
    // 2. exceptions must be a sequence (list or tuple).
    let seq: Option<Vec<MbValue>> = exceptions.as_ptr().and_then(|p| unsafe {
        match &(*p).data {
            ObjData::List(ref lock) => Some(lock.read().unwrap().to_vec()),
            ObjData::Tuple(ref items) => Some(items.clone()),
            _ => None,
        }
    });
    let Some(seq) = seq else {
        mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "second argument (exceptions) must be a sequence".to_string(),
            )),
        );
        return MbValue::none();
    };
    // 3. non-empty.
    if seq.is_empty() {
        mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "second argument (exceptions) must be a non-empty sequence".to_string(),
            )),
        );
        return MbValue::none();
    }
    // 4. every element must be an exception instance.
    for (i, it) in seq.iter().enumerate() {
        if !is_exception_instance(*it) {
            mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "Item {i} of second argument (exceptions) is not an exception"
                ))),
            );
            return MbValue::none();
        }
    }
    // PEP 654: only BaseExceptionGroup may hold a bare BaseException; an
    // ExceptionGroup (or a subclass of it) raises TypeError.
    if cn != "BaseExceptionGroup" && seq.iter().any(|m| eg_member_is_bare_base(*m)) {
        mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(eg_nest_error_message(&cn))),
        );
        return MbValue::none();
    }
    let actual = narrow_eg_class_name(&cn, &seq);
    mb_exception_group_new_as(message, exceptions, &actual)
}

/// The CPython TypeError text for nesting a BaseException in a non-base group:
/// the plain `ExceptionGroup` reads "an ExceptionGroup", a subclass `'Name'`.
pub fn eg_nest_error_message(cn: &str) -> String {
    if cn == "ExceptionGroup" {
        "Cannot nest BaseExceptions in an ExceptionGroup".to_string()
    } else {
        format!("Cannot nest BaseExceptions in '{cn}'")
    }
}

/// `raise (Base)ExceptionGroup(...)` path: validate + build via
/// mb_exception_group_construct, then raise the group. A validation failure
/// leaves its TypeError/ValueError pending and is not overwritten.
pub fn mb_exception_group_construct_and_raise(args_list: MbValue, class_name: MbValue) -> MbValue {
    let group = mb_exception_group_construct(args_list, class_name);
    if !group.is_none() {
        super::class::mb_raise_instance(group);
    }
    MbValue::none()
}

pub fn mb_exception_group_construct_and_raise_with_context(
    args_list: MbValue,
    class_name: MbValue,
    context: MbValue,
) -> MbValue {
    let group = mb_exception_group_construct(args_list, class_name);
    if !group.is_none() {
        super::class::mb_raise_instance_with_context(group, context);
    }
    MbValue::none()
}

pub fn mb_exception_group_construct_and_raise_from(
    args_list: MbValue,
    class_name: MbValue,
    cause: MbValue,
) -> MbValue {
    let group = mb_exception_group_construct(args_list, class_name);
    if !group.is_none() {
        super::class::mb_raise_instance_from(group, cause);
    }
    MbValue::none()
}

pub fn mb_exception_group_construct_and_raise_from_with_context(
    args_list: MbValue,
    class_name: MbValue,
    cause: MbValue,
    context: MbValue,
) -> MbValue {
    let group = mb_exception_group_construct(args_list, class_name);
    if !group.is_none() {
        super::class::mb_raise_instance_from_with_context(group, cause, context);
    }
    MbValue::none()
}

/// except* handler: match exceptions in a group by type.
/// Returns (matched, rest) tuple — matched is an ExceptionGroup of matching
/// exceptions, rest is an ExceptionGroup of non-matching (or None if all matched).
pub fn mb_except_star(group: MbValue, exc_type: MbValue) -> MbValue {
    let cond = match parse_eg_condition(exc_type) {
        Ok(c) => c,
        Err(()) => return MbValue::none(),
    };
    let (matched, rest) = eg_split_rec(group, &cond);
    MbValue::from_ptr(MbObject::new_tuple(vec![matched, rest]))
}

// ── R1: Built-in Exception Class Registration ──

/// Register all built-in exception classes in the class registry with correct
/// MRO inheritance.  Called once during runtime init so that `check_class_hierarchy`
/// resolves exception subclass relationships without the hard-coded match table.
///
/// Hierarchy (Python 3.12):
/// ```text
/// BaseException
/// ├── Exception
/// │   ├── ArithmeticError
/// │   │   ├── ZeroDivisionError
/// │   │   ├── OverflowError
/// │   │   └── FloatingPointError
/// │   ├── LookupError
/// │   │   ├── IndexError
/// │   │   └── KeyError
/// │   ├── ValueError
/// │   │   └── UnicodeError
/// │   │       ├── UnicodeDecodeError
/// │   │       ├── UnicodeEncodeError
/// │   │       └── UnicodeTranslateError
/// │   ├── OSError
/// │   │   ├── FileNotFoundError
/// │   │   ├── PermissionError
/// │   │   ├── IsADirectoryError
/// │   │   ├── FileExistsError
/// │   │   ├── TimeoutError
/// │   │   └── ConnectionError
/// │   │       ├── BrokenPipeError
/// │   │       ├── ConnectionAbortedError
/// │   │       ├── ConnectionRefusedError
/// │   │       └── ConnectionResetError
/// │   ├── TypeError
/// │   ├── AttributeError
/// │   ├── NameError
/// │   │   └── UnboundLocalError
/// │   ├── StopIteration
/// │   ├── StopAsyncIteration
/// │   ├── RuntimeError
/// │   │   ├── NotImplementedError
/// │   │   └── RecursionError
/// │   ├── ImportError
/// │   │   └── ModuleNotFoundError
/// │   ├── SyntaxError
/// │   │   ├── IndentationError
/// │   │   │   └── TabError
/// │   ├── Warning
/// │   │   ├── DeprecationWarning
/// │   │   ├── RuntimeWarning
/// │   │   ├── UserWarning
/// │   │   ├── SyntaxWarning
/// │   │   ├── FutureWarning
/// │   │   ├── PendingDeprecationWarning
/// │   │   ├── UnicodeWarning
/// │   │   ├── BytesWarning
/// │   │   └── ResourceWarning
/// │   └── ExceptionGroup
/// ├── SystemExit
/// ├── KeyboardInterrupt
/// └── GeneratorExit
/// ```
pub fn register_builtin_exceptions() {
    use std::collections::HashMap;
    let empty = HashMap::new;

    // Root
    super::class::mb_class_register("BaseException", vec![], empty());

    // Direct BaseException children (not subclass of Exception)
    super::class::mb_class_register("SystemExit", vec!["BaseException".into()], empty());
    super::class::mb_class_register("KeyboardInterrupt", vec!["BaseException".into()], empty());
    super::class::mb_class_register("GeneratorExit", vec!["BaseException".into()], empty());

    // Exception
    super::class::mb_class_register("Exception", vec!["BaseException".into()], empty());

    // Arithmetic hierarchy
    super::class::mb_class_register("ArithmeticError", vec!["Exception".into()], empty());
    super::class::mb_class_register("ZeroDivisionError", vec!["ArithmeticError".into()], empty());
    super::class::mb_class_register("OverflowError", vec!["ArithmeticError".into()], empty());
    super::class::mb_class_register(
        "FloatingPointError",
        vec!["ArithmeticError".into()],
        empty(),
    );

    // Lookup hierarchy
    super::class::mb_class_register("LookupError", vec!["Exception".into()], empty());
    super::class::mb_class_register("IndexError", vec!["LookupError".into()], empty());
    super::class::mb_class_register("KeyError", vec!["LookupError".into()], empty());
    // zoneinfo.ZoneInfoNotFoundError derives from KeyError (CPython).
    super::class::mb_class_register("ZoneInfoNotFoundError", vec!["KeyError".into()], empty());

    // Value / Unicode hierarchy
    super::class::mb_class_register("ValueError", vec!["Exception".into()], empty());
    super::class::mb_class_register("UnicodeError", vec!["ValueError".into()], empty());
    super::class::mb_class_register("UnicodeDecodeError", vec!["UnicodeError".into()], empty());
    super::class::mb_class_register("UnicodeEncodeError", vec!["UnicodeError".into()], empty());
    super::class::mb_class_register(
        "UnicodeTranslateError",
        vec!["UnicodeError".into()],
        empty(),
    );

    // OS / IO hierarchy
    super::class::mb_class_register("OSError", vec!["Exception".into()], empty());
    super::class::mb_class_register("FileNotFoundError", vec!["OSError".into()], empty());
    super::class::mb_class_register("PermissionError", vec!["OSError".into()], empty());
    super::class::mb_class_register("IsADirectoryError", vec!["OSError".into()], empty());
    super::class::mb_class_register("NotADirectoryError", vec!["OSError".into()], empty());
    super::class::mb_class_register("InterruptedError", vec!["OSError".into()], empty());
    super::class::mb_class_register("ProcessLookupError", vec!["OSError".into()], empty());
    super::class::mb_class_register("ChildProcessError", vec!["OSError".into()], empty());
    super::class::mb_class_register("BlockingIOError", vec!["OSError".into()], empty());
    super::class::mb_class_register("FileExistsError", vec!["OSError".into()], empty());
    super::class::mb_class_register("TimeoutError", vec!["OSError".into()], empty());
    super::class::mb_class_register("ConnectionError", vec!["OSError".into()], empty());
    super::class::mb_class_register("BrokenPipeError", vec!["ConnectionError".into()], empty());
    super::class::mb_class_register(
        "ConnectionAbortedError",
        vec!["ConnectionError".into()],
        empty(),
    );
    super::class::mb_class_register(
        "ConnectionRefusedError",
        vec!["ConnectionError".into()],
        empty(),
    );
    super::class::mb_class_register(
        "ConnectionResetError",
        vec!["ConnectionError".into()],
        empty(),
    );
    // Reference / cycle hierarchy.
    super::class::mb_class_register("ReferenceError", vec!["Exception".into()], empty());

    // Simple Exception subclasses
    super::class::mb_class_register("TypeError", vec!["Exception".into()], empty());
    super::class::mb_class_register("AttributeError", vec!["Exception".into()], empty());
    super::class::mb_class_register("NameError", vec!["Exception".into()], empty());
    super::class::mb_class_register("UnboundLocalError", vec!["NameError".into()], empty());
    super::class::mb_class_register("StopIteration", vec!["Exception".into()], empty());
    super::class::mb_class_register("StopAsyncIteration", vec!["Exception".into()], empty());

    // Runtime hierarchy
    super::class::mb_class_register("RuntimeError", vec!["Exception".into()], empty());
    super::class::mb_class_register("NotImplementedError", vec!["RuntimeError".into()], empty());
    super::class::mb_class_register("RecursionError", vec!["RuntimeError".into()], empty());

    // Import hierarchy
    super::class::mb_class_register("ImportError", vec!["Exception".into()], empty());
    super::class::mb_class_register("ModuleNotFoundError", vec!["ImportError".into()], empty());

    // Syntax hierarchy
    super::class::mb_class_register("SyntaxError", vec!["Exception".into()], empty());
    super::class::mb_class_register("IndentationError", vec!["SyntaxError".into()], empty());
    super::class::mb_class_register("TabError", vec!["IndentationError".into()], empty());

    // Warning hierarchy
    super::class::mb_class_register("Warning", vec!["Exception".into()], empty());
    super::class::mb_class_register("DeprecationWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("RuntimeWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("UserWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("SyntaxWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("FutureWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("PendingDeprecationWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("UnicodeWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("BytesWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("ResourceWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("InvalidTZPathWarning", vec!["RuntimeWarning".into()], empty());
    super::class::mb_class_register("ImportWarning", vec!["Warning".into()], empty());
    super::class::mb_class_register("EncodingWarning", vec!["Warning".into()], empty());

    // ExceptionGroup (PEP 654): BaseExceptionGroup ⊂ BaseException;
    // ExceptionGroup ⊂ (BaseExceptionGroup, Exception).
    super::class::mb_class_register("BaseExceptionGroup", vec!["BaseException".into()], empty());
    super::class::mb_class_register(
        "ExceptionGroup",
        vec!["BaseExceptionGroup".into(), "Exception".into()],
        empty(),
    );
}

// ── R4: Non-destructive exception retrieval ──

/// Retrieve the current exception without clearing the pending state.
/// Returns `MbValue::none()` if no exception is pending.
pub fn mb_get_exception() -> MbValue {
    let has_current = CURRENT_EXCEPTION.with(|cell| cell.borrow().is_some());
    if has_current {
        if let Some(instance) = super::class::peek_last_raised_instance() {
            return instance;
        }
    }
    CURRENT_EXCEPTION.with(|cell| match cell.borrow().as_ref() {
        Some(exc) => {
            let val = store_exception_as_value(MbException {
                exc_type: exc.exc_type.clone(),
                message: exc.message.clone(),
                cause: exc.cause.as_ref().map(|c| {
                    Box::new(MbException {
                        exc_type: c.exc_type.clone(),
                        message: c.message.clone(),
                        cause: None,
                        context: None,
                        suppress_context: c.suppress_context,
                        traceback: c.traceback.clone(),
                    })
                }),
                context: exc.context.as_ref().map(|c| {
                    Box::new(MbException {
                        exc_type: c.exc_type.clone(),
                        message: c.message.clone(),
                        cause: None,
                        context: None,
                        suppress_context: c.suppress_context,
                        traceback: c.traceback.clone(),
                    })
                }),
                suppress_context: exc.suppress_context,
                traceback: exc.traceback.clone(),
            });
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            val
        }
        None => MbValue::none(),
    })
}

// ── R5: ExceptionGroup additional methods (PEP 654) ──

/// A `subgroup`/`split` condition. On mamba a "type" object reaches native code
/// as its name-string (`ValueError` ≡ `"ValueError"`), a tuple of types as a
/// tuple of name-strings, and a predicate as a callable function value.
enum EgCondition {
    /// Single exception type name, e.g. "ValueError".
    Type(String),
    /// Tuple of exception type names, e.g. ["ValueError", "TypeError"].
    Types(Vec<String>),
    /// Callable predicate `f(exc) -> bool`.
    Predicate(MbValue),
}

/// True if `val` is a runtime value that names a registered exception class
/// (i.e. a type object on mamba, represented as a name-string).
fn is_exception_type_name(val: MbValue) -> bool {
    if let Some(s) = extract_str(val) {
        // A type used as a condition must be an exception type. A registered
        // exception class name qualifies; a plain string like "bad arg" does not.
        return is_builtin_exception_name(&s)
            || super::class::class_mro_any(&s, is_builtin_exception_name)
            || is_subclass_of(&s, "BaseException");
    }
    false
}

/// Parse a `subgroup`/`split` condition argument, raising TypeError (the same
/// failure mode as CPython) when the argument is neither an exception type, a
/// tuple of exception types, nor a callable.
fn parse_eg_condition(cond: MbValue) -> Result<EgCondition, ()> {
    // Tuple-of-types: every element must itself be an exception type name.
    if let Some(ptr) = cond.as_ptr() {
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let mut names = Vec::with_capacity(items.len());
                for it in items.iter() {
                    if is_exception_type_name(*it) {
                        names.push(extract_str(*it).unwrap_or_default());
                    } else {
                        mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "expected an exception type, a tuple of exception types, or a callable (not tuple element)".to_string(),
                            )),
                        );
                        return Err(());
                    }
                }
                return Ok(EgCondition::Types(names));
            }
        }
    }
    // Single type object (name-string that names an exception class).
    if is_exception_type_name(cond) {
        return Ok(EgCondition::Type(extract_str(cond).unwrap_or_default()));
    }
    // Callable predicate. A type reaches here as a Str and is NOT callable on
    // mamba, so this only fires for genuine function values.
    if super::builtins::mb_callable(cond).as_bool() == Some(true) {
        return Ok(EgCondition::Predicate(cond));
    }
    // Anything else (plain string, exception instance, list, bad tuple) → TypeError.
    mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "expected an exception type, a tuple of exception types, or a callable".to_string(),
        )),
    );
    Err(())
}

/// True if a single (leaf or group) exception value matches the condition.
fn eg_condition_matches(exc: MbValue, cond: &EgCondition) -> bool {
    match cond {
        EgCondition::Type(t) => {
            let actual = get_exception_type(exc).unwrap_or_default();
            actual == *t || is_subclass_of(&actual, t)
        }
        EgCondition::Types(ts) => {
            let actual = get_exception_type(exc).unwrap_or_default();
            ts.iter()
                .any(|t| actual == *t || is_subclass_of(&actual, t))
        }
        EgCondition::Predicate(f) => {
            // CPython matches on the general truthiness of the predicate's
            // return value, not only a literal `True`. `as_bool()` is `None`
            // for any non-bool return (e.g. a non-empty string or nonzero int),
            // so use full truthiness semantics here.
            let args = MbValue::from_ptr(MbObject::new_list(vec![exc]));
            super::builtins::mb_is_truthy(super::builtins::mb_call_spread(*f, args)) != 0
        }
    }
}

/// True if `exc` is an ExceptionGroup-shaped instance (carries an `exceptions`
/// field). Used to decide whether to recurse.
fn is_exception_group_value(exc: MbValue) -> bool {
    if let Some(ptr) = exc.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "ExceptionGroup"
                    || class_name == "BaseExceptionGroup"
                    || fields.read().unwrap().contains_key("exceptions")
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Read the message string of a group value.
fn eg_message(group: MbValue) -> String {
    group
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .read()
                    .unwrap()
                    .get("message")
                    .and_then(|v| extract_str(*v))
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Read the direct sub-exceptions of a group value as a Vec.
fn eg_child_exceptions(group: MbValue) -> Vec<MbValue> {
    let exc_field = group.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get("exceptions").copied()
        } else {
            None
        }
    });
    if let Some(coll) = exc_field {
        if let Some(ptr) = coll.as_ptr() {
            unsafe {
                return match &(*ptr).data {
                    ObjData::Tuple(items) => items.clone(),
                    ObjData::List(lock) => lock.read().unwrap().to_vec(),
                    _ => vec![],
                };
            }
        }
    }
    vec![]
}

fn eg_class_name(group: MbValue) -> String {
    group
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "ExceptionGroup".to_string())
}

fn eg_copy_notes(from: MbValue, to: MbValue) {
    let notes = from.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get("__notes__").copied()
        } else {
            None
        }
    });
    let Some(notes) = notes else {
        return;
    };
    let Some(notes_ptr) = notes.as_ptr() else {
        return;
    };
    let copied = unsafe {
        match &(*notes_ptr).data {
            ObjData::List(lock) => {
                let items = lock.read().unwrap().to_vec();
                Some(MbValue::from_ptr(MbObject::new_list_borrowed(items)))
            }
            _ => None,
        }
    };
    let Some(copied) = copied else {
        return;
    };
    if let Some(to_ptr) = to.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*to_ptr).data {
                fields
                    .write()
                    .unwrap()
                    .insert("__notes__".to_string(), copied);
            }
        }
    }
}

/// Derive a new group from `group` carrying `excs`, preserving the message
/// (CPython's `derive` + metadata copy). Retains the borrowed children.
fn eg_derive(group: MbValue, excs: Vec<MbValue>) -> MbValue {
    let class_name = eg_class_name(group);
    if class_name != "ExceptionGroup" && class_name != "BaseExceptionGroup" {
        let derive = super::class::lookup_method(&class_name, "derive");
        if !derive.is_none() {
            let exc_tuple = MbValue::from_ptr(MbObject::new_tuple_borrowed(excs));
            let args = MbValue::from_ptr(MbObject::new_list(vec![exc_tuple]));
            let derived = super::class::mb_call_method(
                group,
                MbValue::from_ptr(MbObject::new_str("derive".to_string())),
                args,
            );
            if is_exception_group_value(derived) {
                eg_copy_notes(group, derived);
                return derived;
            }
            mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "derive must return an instance of BaseExceptionGroup".to_string(),
                )),
            );
            return MbValue::none();
        }
    }

    for e in &excs {
        unsafe {
            super::rc::retain_if_ptr(*e);
        }
    }
    let derived = mb_exception_group_new_as(
        MbValue::from_ptr(MbObject::new_str(eg_message(group))),
        MbValue::from_ptr(MbObject::new_list_borrowed(excs)),
        &class_name,
    );
    eg_copy_notes(group, derived);
    derived
}

/// Recursive `split` per PEP 654. Returns `(matched, rest)` where each side is a
/// derived ExceptionGroup or `MbValue::none()`. If the group itself matches the
/// condition, the whole group passes through by identity (matched=group).
fn eg_split_rec(group: MbValue, cond: &EgCondition) -> (MbValue, MbValue) {
    // Whole-group match short-circuit → identity passthrough.
    // The matched side is handed back to the caller as an owned value (split
    // returns it in the tuple; subgroup returns it directly), so retain the
    // group before passing it through by identity — mirroring class.rs's
    // __enter__ which retains the receiver before returning it.
    if eg_condition_matches(group, cond) {
        unsafe {
            super::rc::retain_if_ptr(group);
        }
        return (group, MbValue::none());
    }
    let mut matched = Vec::new();
    let mut rest = Vec::new();
    for exc in eg_child_exceptions(group) {
        if is_exception_group_value(exc) {
            let (m, r) = eg_split_rec(exc, cond);
            if !m.is_none() {
                matched.push(m);
            }
            if !r.is_none() {
                rest.push(r);
            }
        } else if eg_condition_matches(exc, cond) {
            unsafe {
                super::rc::retain_if_ptr(exc);
            }
            matched.push(exc);
        } else {
            unsafe {
                super::rc::retain_if_ptr(exc);
            }
            rest.push(exc);
        }
    }
    let matched_group = if matched.is_empty() {
        MbValue::none()
    } else {
        eg_derive(group, matched)
    };
    let rest_group = if rest.is_empty() {
        MbValue::none()
    } else {
        eg_derive(group, rest)
    };
    (matched_group, rest_group)
}

/// `ExceptionGroup.split(condition)` → `(match, rest)` tuple (PEP 654).
/// `condition` is an exception type, a tuple of exception types, or a callable.
/// Raises TypeError on a bad condition.
pub fn mb_exception_group_split(group: MbValue, condition: MbValue) -> MbValue {
    let cond = match parse_eg_condition(condition) {
        Ok(c) => c,
        Err(()) => return MbValue::none(),
    };
    let (matched, rest) = eg_split_rec(group, &cond);
    MbValue::from_ptr(MbObject::new_tuple(vec![matched, rest]))
}

/// `ExceptionGroup.subgroup(condition)` → matched group or None (PEP 654).
/// Equivalent to `split(condition)[0]`.
pub fn mb_exception_group_subgroup(group: MbValue, condition: MbValue) -> MbValue {
    let cond = match parse_eg_condition(condition) {
        Ok(c) => c,
        Err(()) => return MbValue::none(),
    };
    let (matched, _rest) = eg_split_rec(group, &cond);
    matched
}

/// Access the sub-exceptions of an ExceptionGroup as a tuple.
/// Returns the `exceptions` field of the group, or None if not an ExceptionGroup.
pub fn mb_exception_group_exceptions(group: MbValue) -> MbValue {
    if let Some(ptr) = group.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                if let Some(exc) = fields.get("exceptions") {
                    return *exc;
                }
            }
        }
    }
    MbValue::none()
}

// ── Built-in Exception Constructors (additional) ──

pub fn mb_name_error(msg: &str) -> MbValue {
    mb_exception_new(
        MbValue::from_ptr(MbObject::new_str("NameError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    )
}

// ── Cleanup ──

/// Reset all exception-related thread_local state to defaults.
/// Called as part of centralized runtime cleanup between test executions.
pub(crate) fn cleanup_all_exceptions() {
    let _ = CURRENT_EXCEPTION.with(|c| c.try_borrow_mut().map(|mut m| *m = None));
    let _ = EXCEPTION_HANDLERS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raise_and_catch() {
        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
        let msg = MbValue::from_ptr(MbObject::new_str("bad value".to_string()));
        mb_raise(typ, msg);
        assert_eq!(mb_has_exception().as_bool(), Some(true));
        let exc = mb_catch_exception();
        assert!(!exc.is_none());
        assert_eq!(mb_has_exception().as_bool(), Some(false));
    }

    #[test]
    fn test_exception_matches() {
        let exc = mb_value_error("test");
        let ve = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
        let te = MbValue::from_ptr(MbObject::new_str("TypeError".to_string()));
        let base = MbValue::from_ptr(MbObject::new_str("Exception".to_string()));
        assert_eq!(mb_exception_matches(exc, ve).as_bool(), Some(true));
        assert_eq!(mb_exception_matches(exc, te).as_bool(), Some(false));
        assert_eq!(mb_exception_matches(exc, base).as_bool(), Some(true));
    }

    #[test]
    fn test_subclass_hierarchy() {
        assert!(is_subclass_of("IndexError", "LookupError"));
        assert!(is_subclass_of("KeyError", "LookupError"));
        assert!(is_subclass_of("ZeroDivisionError", "ArithmeticError"));
        assert!(!is_subclass_of("ValueError", "LookupError"));
    }

    // ── Additional tests ──

    #[test]
    fn test_exception_new_creates_instance() {
        let typ = MbValue::from_ptr(MbObject::new_str("RuntimeError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("oops".into()));
        let exc = mb_exception_new(typ, msg);
        assert!(exc.is_ptr());
        let actual_type = get_exception_type(exc).unwrap();
        assert_eq!(actual_type, "RuntimeError");
        let actual_msg = get_exception_message(exc).unwrap();
        assert_eq!(actual_msg, "oops");
    }

    #[test]
    fn test_type_error_constructor() {
        let exc = mb_type_error("bad type");
        let t = get_exception_type(exc).unwrap();
        assert_eq!(t, "TypeError");
        let m = get_exception_message(exc).unwrap();
        assert_eq!(m, "bad type");
    }

    #[test]
    fn test_index_error_constructor() {
        let exc = mb_index_error("out of range");
        assert_eq!(get_exception_type(exc).unwrap(), "IndexError");
        assert_eq!(get_exception_message(exc).unwrap(), "out of range");
    }

    #[test]
    fn test_key_error_constructor() {
        let exc = mb_key_error("missing");
        assert_eq!(get_exception_type(exc).unwrap(), "KeyError");
    }

    #[test]
    fn test_attribute_error_constructor() {
        let exc = mb_attribute_error("no attr");
        assert_eq!(get_exception_type(exc).unwrap(), "AttributeError");
    }

    #[test]
    fn test_stop_iteration_constructor() {
        let exc = mb_stop_iteration();
        assert_eq!(get_exception_type(exc).unwrap(), "StopIteration");
    }

    #[test]
    fn test_runtime_error_constructor() {
        let exc = mb_runtime_error("fail");
        assert_eq!(get_exception_type(exc).unwrap(), "RuntimeError");
        assert_eq!(get_exception_message(exc).unwrap(), "fail");
    }

    #[test]
    fn test_zero_division_error_constructor() {
        let exc = mb_zero_division_error();
        assert_eq!(get_exception_type(exc).unwrap(), "ZeroDivisionError");
        assert_eq!(get_exception_message(exc).unwrap(), "division by zero");
    }

    #[test]
    fn test_import_error_constructor() {
        let exc = mb_import_error("no module");
        assert_eq!(get_exception_type(exc).unwrap(), "ImportError");
    }

    #[test]
    fn test_raise_from_chains_exceptions() {
        mb_clear_exception();
        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("outer".into()));
        let cause = mb_type_error("inner");
        mb_raise_from(typ, msg, cause);
        assert_eq!(mb_has_exception().as_bool(), Some(true));
        let exc = mb_catch_exception();
        assert!(!exc.is_none());
        assert_eq!(get_exception_type(exc).unwrap(), "ValueError");
        // Verify __cause__ field is populated
        let cause_attr = MbValue::from_ptr(MbObject::new_str("__cause__".into()));
        let cause_val = crate::runtime::class::mb_getattr(exc, cause_attr);
        assert!(!cause_val.is_none(), "__cause__ should not be None");
        assert_eq!(get_exception_type(cause_val).unwrap(), "TypeError");
        assert_eq!(get_exception_message(cause_val).unwrap(), "inner");
    }

    #[test]
    fn test_clear_exception() {
        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("test".into()));
        mb_raise(typ, msg);
        assert_eq!(mb_has_exception().as_bool(), Some(true));
        mb_clear_exception();
        assert_eq!(mb_has_exception().as_bool(), Some(false));
    }

    #[test]
    fn test_catch_when_no_exception_returns_none() {
        mb_clear_exception();
        let exc = mb_catch_exception();
        assert!(exc.is_none());
    }

    #[test]
    fn test_exception_group_new() {
        let msg = MbValue::from_ptr(MbObject::new_str("group msg".into()));
        let e1 = mb_value_error("err1");
        let e2 = mb_type_error("err2");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2]));
        let group = mb_exception_group_new(msg, exceptions);
        assert!(group.is_ptr());
        let t = get_exception_type(group).unwrap();
        assert_eq!(t, "ExceptionGroup");
    }

    #[test]
    fn test_push_pop_handler() {
        mb_push_handler(true);
        mb_push_handler(false);
        mb_pop_handler();
        mb_pop_handler();
        // No panic = success
    }

    #[test]
    fn test_subclass_arithmetic_error() {
        assert!(is_subclass_of("OverflowError", "ArithmeticError"));
        assert!(is_subclass_of("FloatingPointError", "ArithmeticError"));
    }

    #[test]
    fn test_subclass_os_error() {
        assert!(is_subclass_of("FileNotFoundError", "OSError"));
        assert!(is_subclass_of("PermissionError", "OSError"));
        assert!(is_subclass_of("TimeoutError", "OSError"));
    }

    #[test]
    fn test_subclass_connection_error() {
        assert!(is_subclass_of("BrokenPipeError", "ConnectionError"));
        assert!(is_subclass_of("ConnectionRefusedError", "ConnectionError"));
    }

    #[test]
    fn test_all_exceptions_match_base_exception() {
        let exc = mb_type_error("x");
        let base = MbValue::from_ptr(MbObject::new_str("BaseException".into()));
        assert_eq!(mb_exception_matches(exc, base).as_bool(), Some(true));
    }

    #[test]
    fn test_exception_with_cause() {
        let cause = MbException::new("IOError", "disk full");
        let exc = MbException::new("RuntimeError", "failed").with_cause(cause);
        assert!(exc.cause.is_some());
        assert_eq!(exc.cause.unwrap().exc_type, "IOError");
    }

    #[test]
    fn test_subclass_value_error_unicode() {
        assert!(is_subclass_of("UnicodeDecodeError", "ValueError"));
        assert!(is_subclass_of("UnicodeEncodeError", "ValueError"));
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_exception_types_exist() {
        let types = [
            "OverflowError",
            "ZeroDivisionError",
            "FloatingPointError",
            "IndexError",
            "KeyError",
            "UnicodeDecodeError",
            "UnicodeEncodeError",
            "UnicodeTranslateError",
            "FileNotFoundError",
            "PermissionError",
            "IsADirectoryError",
            "NotADirectoryError",
            "FileExistsError",
            "TimeoutError",
            "BlockingIOError",
            "ChildProcessError",
            "BrokenPipeError",
            "ConnectionAbortedError",
            "ConnectionRefusedError",
            "ConnectionResetError",
            "RecursionError",
            "NotImplementedError",
            "IndentationError",
            "TabError",
            "UnboundLocalError",
            "ModuleNotFoundError",
            "StopAsyncIteration",
            "SystemError",
            "DeprecationWarning",
            "RuntimeWarning",
            "SyntaxWarning",
            "UnicodeWarning",
            "BytesWarning",
            "ResourceWarning",
            "FutureWarning",
            "PendingDeprecationWarning",
            "UserWarning",
        ];
        for t in types {
            let exc = MbException::new(t, "msg");
            assert_eq!(exc.exc_type, t, "type {t} failed");
        }
    }

    // R4.2: implicit chaining — __context__ is set to the active exception
    #[test]
    fn test_py312_implicit_chaining_sets_context() {
        let inner = MbException::new("ValueError", "original");
        let outer = MbException::new("RuntimeError", "wrapper").with_context(inner);
        assert!(outer.context.is_some());
        assert_eq!(outer.context.as_ref().unwrap().exc_type, "ValueError");
        assert!(!outer.suppress_context);
    }

    // R4.2: mb_raise_with_context populates __context__ on the MbValue instance
    #[test]
    fn test_raise_with_context_sets_context_field() {
        mb_clear_exception();
        let ve_type = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let ve_msg = MbValue::from_ptr(MbObject::new_str("original".into()));
        let context = mb_exception_new(ve_type, ve_msg);

        let rt_type = MbValue::from_ptr(MbObject::new_str("RuntimeError".into()));
        let rt_msg = MbValue::from_ptr(MbObject::new_str("wrapper".into()));
        mb_raise_with_context(rt_type, rt_msg, context);

        assert_eq!(mb_has_exception().as_bool(), Some(true));
        let exc = mb_catch_exception();
        assert_eq!(get_exception_type(exc).unwrap(), "RuntimeError");

        // __context__ should be the ValueError
        let ctx_attr = MbValue::from_ptr(MbObject::new_str("__context__".into()));
        let ctx_val = crate::runtime::class::mb_getattr(exc, ctx_attr);
        assert!(!ctx_val.is_none(), "__context__ should not be None");
        assert_eq!(get_exception_type(ctx_val).unwrap(), "ValueError");
        assert_eq!(get_exception_message(ctx_val).unwrap(), "original");
        mb_clear_exception();
    }

    // R4.1: raise X from None sets __suppress_context__ = True
    #[test]
    fn test_raise_from_none_sets_suppress_context() {
        mb_clear_exception();
        let typ = MbValue::from_ptr(MbObject::new_str("RuntimeError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("clean".into()));
        // None cause → suppress_context = true
        mb_raise_from(typ, msg, MbValue::none());

        assert_eq!(mb_has_exception().as_bool(), Some(true));
        let exc = mb_catch_exception();
        assert_eq!(get_exception_type(exc).unwrap(), "RuntimeError");

        let sup_attr = MbValue::from_ptr(MbObject::new_str("__suppress_context__".into()));
        let sup_val = crate::runtime::class::mb_getattr(exc, sup_attr);
        assert_eq!(
            sup_val.as_bool(),
            Some(true),
            "__suppress_context__ should be true for `raise X from None`"
        );

        // __cause__ should be None
        let cause_attr = MbValue::from_ptr(MbObject::new_str("__cause__".into()));
        let cause_val = crate::runtime::class::mb_getattr(exc, cause_attr);
        assert!(
            cause_val.is_none(),
            "__cause__ should be None for `raise X from None`"
        );
        mb_clear_exception();
    }

    #[test]
    fn test_py312_explicit_chaining_sets_cause() {
        let cause = MbException::new("ZeroDivisionError", "div by zero");
        let exc = MbException::new("ValueError", "derived").with_cause(cause);
        assert!(exc.cause.is_some());
        assert_eq!(exc.cause.as_ref().unwrap().exc_type, "ZeroDivisionError");
    }

    // R4.1: suppress_context can be set directly on MbException struct
    #[test]
    fn test_py312_raise_from_none_suppresses_context_struct() {
        let mut exc = MbException::new("RuntimeError", "clean");
        exc.suppress_context = true;
        assert!(exc.suppress_context);
        assert!(exc.cause.is_none());
    }

    #[test]
    fn test_py312_stop_iteration_has_value() {
        let si = MbException::new("StopIteration", "42");
        assert_eq!(si.exc_type, "StopIteration");
        assert_eq!(si.message, "42");
    }

    // ── Cleanup tests (R1: per-module cleanup for exceptions) ──

    #[test]
    fn test_cleanup_all_exceptions_clears_current() {
        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("cleanup test".into()));
        mb_raise(typ, msg);
        assert_eq!(mb_has_exception().as_bool(), Some(true));

        cleanup_all_exceptions();

        assert_eq!(
            mb_has_exception().as_bool(),
            Some(false),
            "CURRENT_EXCEPTION should be None after cleanup"
        );
    }

    #[test]
    fn test_cleanup_all_exceptions_clears_handlers() {
        mb_push_handler(true);
        mb_push_handler(false);
        mb_push_handler(true);

        cleanup_all_exceptions();

        // After cleanup, handler stack is empty — pop should not panic
        // but we verify by calling cleanup again (idempotent)
        cleanup_all_exceptions();
    }

    #[test]
    fn test_cleanup_all_exceptions_on_empty() {
        mb_clear_exception();
        cleanup_all_exceptions();
        // No panic = success
        assert_eq!(mb_has_exception().as_bool(), Some(false));
    }

    #[test]
    fn test_py312_unicode_error_hierarchy() {
        assert!(is_subclass_of("UnicodeDecodeError", "UnicodeError"));
        assert!(is_subclass_of("UnicodeEncodeError", "UnicodeError"));
        assert!(is_subclass_of("UnicodeTranslateError", "UnicodeError"));
        assert!(is_subclass_of("UnicodeError", "ValueError"));
        assert!(is_subclass_of("UnicodeError", "Exception"));
        assert!(is_subclass_of("UnicodeError", "BaseException"));
    }

    #[test]
    fn test_py312_warning_hierarchy() {
        for w in [
            "DeprecationWarning",
            "RuntimeWarning",
            "UserWarning",
            "SyntaxWarning",
            "FutureWarning",
            "PendingDeprecationWarning",
            "UnicodeWarning",
            "BytesWarning",
            "ResourceWarning",
        ] {
            assert!(is_subclass_of(w, "Warning"), "{w} should be Warning");
            assert!(is_subclass_of(w, "Exception"), "{w} should be Exception");
            assert!(
                is_subclass_of(w, "BaseException"),
                "{w} should be BaseException"
            );
        }
    }

    #[test]
    fn test_py312_connection_error_subtypes() {
        for e in [
            "BrokenPipeError",
            "ConnectionAbortedError",
            "ConnectionRefusedError",
            "ConnectionResetError",
        ] {
            assert!(is_subclass_of(e, "ConnectionError"));
            assert!(is_subclass_of(e, "OSError"));
        }
    }

    #[test]
    fn test_py312_exception_group_new() {
        let msg = MbValue::from_ptr(MbObject::new_str("test".to_string()));
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![
            mb_value_error("v1"),
            mb_type_error("v2"),
        ]));
        let group = mb_exception_group_new(msg, exceptions);
        assert!(group.is_ptr());
        let t = get_exception_type(group).unwrap();
        assert_eq!(t, "ExceptionGroup");
    }

    // ── R1: register_builtin_exceptions verifies class registry hierarchy ──

    #[test]
    fn test_register_builtin_exceptions_populates_registry() {
        register_builtin_exceptions();
        // After registration, check_class_hierarchy should resolve MRO chains
        assert!(
            crate::runtime::class::check_class_hierarchy("Exception", "BaseException"),
            "Exception should be subclass of BaseException via registry"
        );
        assert!(
            crate::runtime::class::check_class_hierarchy("ValueError", "Exception"),
            "ValueError should be subclass of Exception via registry"
        );
        assert!(
            crate::runtime::class::check_class_hierarchy("TypeError", "Exception"),
            "TypeError should be subclass of Exception via registry"
        );
        assert!(
            crate::runtime::class::check_class_hierarchy("ZeroDivisionError", "ArithmeticError"),
            "ZeroDivisionError should be subclass of ArithmeticError via registry"
        );
        assert!(
            crate::runtime::class::check_class_hierarchy("IndexError", "LookupError"),
            "IndexError should be subclass of LookupError via registry"
        );
        assert!(
            crate::runtime::class::check_class_hierarchy("KeyError", "LookupError"),
            "KeyError should be subclass of LookupError via registry"
        );
        assert!(
            crate::runtime::class::check_class_hierarchy("FileNotFoundError", "OSError"),
            "FileNotFoundError should be subclass of OSError via registry"
        );
    }

    #[test]
    fn test_register_builtin_exceptions_base_exception_children() {
        register_builtin_exceptions();
        assert!(crate::runtime::class::check_class_hierarchy(
            "SystemExit",
            "BaseException"
        ));
        assert!(crate::runtime::class::check_class_hierarchy(
            "KeyboardInterrupt",
            "BaseException"
        ));
        assert!(crate::runtime::class::check_class_hierarchy(
            "GeneratorExit",
            "BaseException"
        ));
        // These should NOT be subclasses of Exception
        assert!(!crate::runtime::class::check_class_hierarchy(
            "SystemExit",
            "Exception"
        ));
        assert!(!crate::runtime::class::check_class_hierarchy(
            "KeyboardInterrupt",
            "Exception"
        ));
        assert!(!crate::runtime::class::check_class_hierarchy(
            "GeneratorExit",
            "Exception"
        ));
    }

    #[test]
    fn test_register_builtin_exceptions_exception_group() {
        register_builtin_exceptions();
        assert!(
            crate::runtime::class::check_class_hierarchy("ExceptionGroup", "Exception"),
            "ExceptionGroup should inherit from Exception via registry"
        );
    }

    // ── R2: mb_exception_new_with_args preserves constructor arguments ──

    #[test]
    fn test_exception_new_with_args_single_arg() {
        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("bad input".into()),
        )]));
        let exc = mb_exception_new_with_args(typ, args);
        assert!(exc.is_ptr());
        assert_eq!(get_exception_type(exc).unwrap(), "ValueError");
        assert_eq!(get_exception_message(exc).unwrap(), "bad input");
        // Check args field is a tuple with one element
        let args_attr = MbValue::from_ptr(MbObject::new_str("args".into()));
        let args_val = crate::runtime::class::mb_getattr(exc, args_attr);
        assert!(!args_val.is_none(), "args should not be None");
    }

    #[test]
    fn test_exception_new_with_args_multiple_args() {
        let typ = MbValue::from_ptr(MbObject::new_str("TypeError".into()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("first".into())),
            MbValue::from_int(42),
        ]));
        let exc = mb_exception_new_with_args(typ, args);
        assert_eq!(get_exception_type(exc).unwrap(), "TypeError");
        assert_eq!(get_exception_message(exc).unwrap(), "first");
        // args tuple should preserve all constructor arguments
        let args_attr = MbValue::from_ptr(MbObject::new_str("args".into()));
        let args_val = crate::runtime::class::mb_getattr(exc, args_attr);
        assert!(args_val.is_ptr(), "args should be a tuple");
    }

    #[test]
    fn test_exception_new_with_args_no_args() {
        let typ = MbValue::from_ptr(MbObject::new_str("Exception".into()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let exc = mb_exception_new_with_args(typ, args);
        assert_eq!(get_exception_type(exc).unwrap(), "Exception");
        // Empty args → message is empty string
        assert_eq!(get_exception_message(exc).unwrap(), "");
    }

    // ── R3: mb_exception_matches with registry-based MRO ──

    #[test]
    fn test_exception_matches_inheritance_via_registry() {
        register_builtin_exceptions();
        let exc = mb_value_error("test");
        let exc_type = MbValue::from_ptr(MbObject::new_str("Exception".into()));
        assert_eq!(
            mb_exception_matches(exc, exc_type).as_bool(),
            Some(true),
            "ValueError should match Exception (parent class)"
        );
    }

    #[test]
    fn test_exception_matches_specific_mismatch() {
        let exc = mb_value_error("test");
        let te = MbValue::from_ptr(MbObject::new_str("TypeError".into()));
        assert_eq!(
            mb_exception_matches(exc, te).as_bool(),
            Some(false),
            "ValueError should NOT match TypeError"
        );
    }

    #[test]
    fn test_exception_matches_exact_type() {
        let exc = mb_value_error("test");
        let ve = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        assert_eq!(
            mb_exception_matches(exc, ve).as_bool(),
            Some(true),
            "ValueError should match ValueError exactly"
        );
    }

    #[test]
    fn test_exception_matches_zero_division_to_arithmetic() {
        let exc = mb_zero_division_error();
        let arith = MbValue::from_ptr(MbObject::new_str("ArithmeticError".into()));
        assert_eq!(
            mb_exception_matches(exc, arith).as_bool(),
            Some(true),
            "ZeroDivisionError should match ArithmeticError (parent)"
        );
    }

    #[test]
    fn test_exception_matches_index_to_lookup() {
        let exc = mb_index_error("oob");
        let lookup = MbValue::from_ptr(MbObject::new_str("LookupError".into()));
        assert_eq!(
            mb_exception_matches(exc, lookup).as_bool(),
            Some(true),
            "IndexError should match LookupError (parent)"
        );
    }

    // ── R4: mb_get_exception — non-destructive retrieval ──

    #[test]
    fn test_get_exception_returns_current_without_clearing() {
        mb_clear_exception();
        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("pending".into()));
        mb_raise(typ, msg);
        assert_eq!(mb_has_exception().as_bool(), Some(true));

        // mb_get_exception should return the exception without clearing it
        let exc = mb_get_exception();
        assert!(
            !exc.is_none(),
            "mb_get_exception should return the pending exception"
        );
        assert_eq!(get_exception_type(exc).unwrap(), "ValueError");
        assert_eq!(get_exception_message(exc).unwrap(), "pending");

        // Exception should STILL be pending (non-destructive)
        assert_eq!(
            mb_has_exception().as_bool(),
            Some(true),
            "Exception should still be pending after mb_get_exception"
        );

        mb_clear_exception();
    }

    #[test]
    fn test_get_exception_returns_none_when_no_exception() {
        mb_clear_exception();
        let exc = mb_get_exception();
        assert!(
            exc.is_none(),
            "mb_get_exception should return None when no exception is pending"
        );
    }

    #[test]
    fn test_get_exception_preserves_chaining_fields() {
        mb_clear_exception();
        let typ = MbValue::from_ptr(MbObject::new_str("RuntimeError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("outer".into()));
        let cause = mb_type_error("inner cause");
        mb_raise_from(typ, msg, cause);

        let exc = mb_get_exception();
        assert_eq!(get_exception_type(exc).unwrap(), "RuntimeError");

        // __cause__ should be preserved
        let cause_attr = MbValue::from_ptr(MbObject::new_str("__cause__".into()));
        let cause_val = crate::runtime::class::mb_getattr(exc, cause_attr);
        assert!(
            !cause_val.is_none(),
            "__cause__ should be preserved in get_exception"
        );
        assert_eq!(get_exception_type(cause_val).unwrap(), "TypeError");

        mb_clear_exception();
    }

    // ── R4: mb_raise / mb_clear_exception lifecycle ──

    #[test]
    fn test_raise_clear_raise_cycle() {
        mb_clear_exception();
        // First raise
        let t1 = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let m1 = MbValue::from_ptr(MbObject::new_str("first".into()));
        mb_raise(t1, m1);
        assert_eq!(mb_has_exception().as_bool(), Some(true));

        // Clear
        mb_clear_exception();
        assert_eq!(mb_has_exception().as_bool(), Some(false));

        // Second raise
        let t2 = MbValue::from_ptr(MbObject::new_str("TypeError".into()));
        let m2 = MbValue::from_ptr(MbObject::new_str("second".into()));
        mb_raise(t2, m2);
        assert_eq!(mb_has_exception().as_bool(), Some(true));

        let exc = mb_catch_exception();
        assert_eq!(get_exception_type(exc).unwrap(), "TypeError");
        assert_eq!(get_exception_message(exc).unwrap(), "second");
    }

    // ── R4: mb_raise_from_with_context (both cause and context) ──

    #[test]
    fn test_raise_from_with_context_sets_both() {
        mb_clear_exception();
        let typ = MbValue::from_ptr(MbObject::new_str("RuntimeError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("outer".into()));
        let cause = mb_type_error("explicit cause");
        let context = mb_value_error("implicit context");

        mb_raise_from_with_context(typ, msg, cause, context);
        assert_eq!(mb_has_exception().as_bool(), Some(true));

        let exc = mb_catch_exception();
        assert_eq!(get_exception_type(exc).unwrap(), "RuntimeError");

        // __cause__ should be set
        let cause_attr = MbValue::from_ptr(MbObject::new_str("__cause__".into()));
        let cause_val = crate::runtime::class::mb_getattr(exc, cause_attr);
        assert!(!cause_val.is_none(), "__cause__ should be set");
        assert_eq!(get_exception_type(cause_val).unwrap(), "TypeError");

        // __context__ should be set
        let ctx_attr = MbValue::from_ptr(MbObject::new_str("__context__".into()));
        let ctx_val = crate::runtime::class::mb_getattr(exc, ctx_attr);
        assert!(!ctx_val.is_none(), "__context__ should be set");
        assert_eq!(get_exception_type(ctx_val).unwrap(), "ValueError");

        // __suppress_context__ should be true (raise from always suppresses)
        let sup_attr = MbValue::from_ptr(MbObject::new_str("__suppress_context__".into()));
        let sup_val = crate::runtime::class::mb_getattr(exc, sup_attr);
        assert_eq!(
            sup_val.as_bool(),
            Some(true),
            "__suppress_context__ should be true for raise-from"
        );
    }

    #[test]
    fn test_raise_from_with_context_none_cause() {
        mb_clear_exception();
        let typ = MbValue::from_ptr(MbObject::new_str("RuntimeError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("outer".into()));
        let context = mb_value_error("implicit context");

        mb_raise_from_with_context(typ, msg, MbValue::none(), context);
        let exc = mb_catch_exception();
        assert_eq!(get_exception_type(exc).unwrap(), "RuntimeError");

        // __cause__ should be None
        let cause_attr = MbValue::from_ptr(MbObject::new_str("__cause__".into()));
        let cause_val = crate::runtime::class::mb_getattr(exc, cause_attr);
        assert!(
            cause_val.is_none(),
            "__cause__ should be None when cause is None"
        );

        // __context__ should still be set
        let ctx_attr = MbValue::from_ptr(MbObject::new_str("__context__".into()));
        let ctx_val = crate::runtime::class::mb_getattr(exc, ctx_attr);
        assert!(!ctx_val.is_none(), "__context__ should still be set");
    }

    // ── R4: mb_reraise ──

    #[test]
    fn test_reraise_puts_exception_back() {
        mb_clear_exception();
        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("original".into()));
        mb_raise(typ, msg);

        // Catch it (clears pending state)
        let exc = mb_catch_exception();
        assert_eq!(mb_has_exception().as_bool(), Some(false));
        assert_eq!(get_exception_type(exc).unwrap(), "ValueError");

        // Re-raise it
        mb_reraise(exc);
        assert_eq!(mb_has_exception().as_bool(), Some(true));

        let reraised = mb_catch_exception();
        assert_eq!(get_exception_type(reraised).unwrap(), "ValueError");
        assert_eq!(get_exception_message(reraised).unwrap(), "original");
    }

    // ── R5: ExceptionGroup split / subgroup / exceptions ──

    #[test]
    fn test_exception_group_split_by_type() {
        let msg = MbValue::from_ptr(MbObject::new_str("group".into()));
        let e1 = mb_value_error("v1");
        let e2 = mb_type_error("t1");
        let e3 = mb_value_error("v2");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2, e3]));
        let group = mb_exception_group_new(msg, exceptions);

        let predicate = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let result = mb_exception_group_split(group, predicate);

        // Result is a (matched, rest) tuple
        assert!(result.is_ptr());
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 2, "split should return a 2-tuple");
                let matched = items[0];
                let rest = items[1];

                // matched should be an ExceptionGroup with 2 ValueErrors
                assert!(!matched.is_none(), "matched should not be None");
                assert_eq!(get_exception_type(matched).unwrap(), "ExceptionGroup");
                let matched_excs = mb_exception_group_exceptions(matched);
                if let Some(mptr) = matched_excs.as_ptr() {
                    if let ObjData::Tuple(ref mitems) = (*mptr).data {
                        assert_eq!(mitems.len(), 2, "matched group should have 2 exceptions");
                        assert_eq!(get_exception_type(mitems[0]).unwrap(), "ValueError");
                        assert_eq!(get_exception_type(mitems[1]).unwrap(), "ValueError");
                    }
                }

                // rest should be an ExceptionGroup with 1 TypeError
                assert!(!rest.is_none(), "rest should not be None");
                assert_eq!(get_exception_type(rest).unwrap(), "ExceptionGroup");
                let rest_excs = mb_exception_group_exceptions(rest);
                if let Some(rptr) = rest_excs.as_ptr() {
                    if let ObjData::Tuple(ref ritems) = (*rptr).data {
                        assert_eq!(ritems.len(), 1, "rest group should have 1 exception");
                        assert_eq!(get_exception_type(ritems[0]).unwrap(), "TypeError");
                    }
                }
            } else {
                panic!("split result should be a tuple");
            }
        }
    }

    #[test]
    fn test_exception_group_split_all_match() {
        let msg = MbValue::from_ptr(MbObject::new_str("all match".into()));
        let e1 = mb_value_error("v1");
        let e2 = mb_value_error("v2");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2]));
        let group = mb_exception_group_new(msg, exceptions);

        let predicate = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let result = mb_exception_group_split(group, predicate);

        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let matched = items[0];
                let rest = items[1];
                assert!(!matched.is_none(), "matched should contain all exceptions");
                assert!(rest.is_none(), "rest should be None when all match");
            }
        }
    }

    #[test]
    fn test_exception_group_split_none_match() {
        let msg = MbValue::from_ptr(MbObject::new_str("none match".into()));
        let e1 = mb_type_error("t1");
        let e2 = mb_type_error("t2");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2]));
        let group = mb_exception_group_new(msg, exceptions);

        let predicate = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let result = mb_exception_group_split(group, predicate);

        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let matched = items[0];
                let rest = items[1];
                assert!(matched.is_none(), "matched should be None when none match");
                assert!(!rest.is_none(), "rest should contain all exceptions");
            }
        }
    }

    #[test]
    fn test_exception_group_subgroup_filters() {
        let msg = MbValue::from_ptr(MbObject::new_str("filter".into()));
        let e1 = mb_value_error("v1");
        let e2 = mb_type_error("t1");
        let e3 = mb_value_error("v2");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2, e3]));
        let group = mb_exception_group_new(msg, exceptions);

        let predicate = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let subgroup = mb_exception_group_subgroup(group, predicate);

        assert!(!subgroup.is_none(), "subgroup should not be None");
        assert_eq!(get_exception_type(subgroup).unwrap(), "ExceptionGroup");

        // Should contain only the 2 ValueErrors
        let sub_excs = mb_exception_group_exceptions(subgroup);
        unsafe {
            if let Some(ptr) = sub_excs.as_ptr() {
                if let ObjData::Tuple(ref items) = (*ptr).data {
                    assert_eq!(items.len(), 2, "subgroup should have 2 ValueErrors");
                    for item in items {
                        assert_eq!(get_exception_type(*item).unwrap(), "ValueError");
                    }
                }
            }
        }
    }

    #[test]
    fn test_exception_group_subgroup_no_match_returns_none() {
        let msg = MbValue::from_ptr(MbObject::new_str("empty".into()));
        let e1 = mb_type_error("t1");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1]));
        let group = mb_exception_group_new(msg, exceptions);

        let predicate = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let subgroup = mb_exception_group_subgroup(group, predicate);
        assert!(
            subgroup.is_none(),
            "subgroup should be None when no exceptions match"
        );
    }

    #[test]
    fn test_exception_group_exceptions_returns_tuple() {
        let msg = MbValue::from_ptr(MbObject::new_str("exclist".into()));
        let e1 = mb_value_error("v1");
        let e2 = mb_type_error("t1");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2]));
        let group = mb_exception_group_new(msg, exceptions);

        let excs = mb_exception_group_exceptions(group);
        assert!(
            !excs.is_none(),
            "exceptions() should return the sub-exceptions"
        );
        unsafe {
            if let Some(ptr) = excs.as_ptr() {
                if let ObjData::Tuple(ref items) = (*ptr).data {
                    assert_eq!(items.len(), 2, "should have 2 sub-exceptions");
                    assert_eq!(get_exception_type(items[0]).unwrap(), "ValueError");
                    assert_eq!(get_exception_type(items[1]).unwrap(), "TypeError");
                } else {
                    panic!("exceptions should be stored as tuple");
                }
            }
        }
    }

    #[test]
    fn test_exception_group_exceptions_on_non_group_returns_none() {
        let exc = mb_value_error("not a group");
        let result = mb_exception_group_exceptions(exc);
        // A regular exception has no "exceptions" field → returns None
        assert!(
            result.is_none(),
            "exceptions() on a non-group should return None"
        );
    }

    // ── R6: except* syntax — mb_except_star ──

    #[test]
    fn test_except_star_splits_group_by_type() {
        // Scenario from spec: ExceptionGroup with [ValueError, TypeError],
        // except* ValueError → handler catches ValueError, TypeError propagates
        let msg = MbValue::from_ptr(MbObject::new_str("mixed".into()));
        let e1 = mb_value_error("v");
        let e2 = mb_type_error("t");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2]));
        let group = mb_exception_group_new(msg, exceptions);

        let target = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let result = mb_except_star(group, target);

        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 2);
                let matched = items[0];
                let rest = items[1];

                // matched: ExceptionGroup with ValueError
                assert!(!matched.is_none());
                let matched_excs = mb_exception_group_exceptions(matched);
                if let Some(mptr) = matched_excs.as_ptr() {
                    if let ObjData::Tuple(ref mitems) = (*mptr).data {
                        assert_eq!(mitems.len(), 1);
                        assert_eq!(get_exception_type(mitems[0]).unwrap(), "ValueError");
                    }
                }

                // rest: ExceptionGroup with TypeError (propagates)
                assert!(!rest.is_none());
                let rest_excs = mb_exception_group_exceptions(rest);
                if let Some(rptr) = rest_excs.as_ptr() {
                    if let ObjData::Tuple(ref ritems) = (*rptr).data {
                        assert_eq!(ritems.len(), 1);
                        assert_eq!(get_exception_type(ritems[0]).unwrap(), "TypeError");
                    }
                }
            }
        }
    }

    #[test]
    fn test_except_star_with_inheritance() {
        // except* Exception should match all sub-exceptions
        let msg = MbValue::from_ptr(MbObject::new_str("all".into()));
        let e1 = mb_value_error("v");
        let e2 = mb_type_error("t");
        let e3 = mb_key_error("k");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2, e3]));
        let group = mb_exception_group_new(msg, exceptions);

        let target = MbValue::from_ptr(MbObject::new_str("Exception".into()));
        let result = mb_except_star(group, target);

        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let matched = items[0];
                let rest = items[1];
                assert!(!matched.is_none(), "all exceptions should match Exception");
                assert!(rest.is_none(), "no exceptions should remain");
            }
        }
    }

    // ── Acceptance Scenario: Raise ValueError ──

    #[test]
    fn test_scenario_raise_value_error() {
        mb_clear_exception();
        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let msg = MbValue::from_ptr(MbObject::new_str("bad input".into()));
        mb_raise(typ, msg);

        let exc = mb_catch_exception();
        assert_eq!(
            get_exception_type(exc).unwrap(),
            "ValueError",
            "Exception instance should have class=ValueError"
        );
        assert_eq!(
            get_exception_message(exc).unwrap(),
            "bad input",
            "Exception instance should have message='bad input'"
        );
    }

    // ── Acceptance Scenario: Except matching via inheritance ──

    #[test]
    fn test_scenario_except_catches_value_error_via_inheritance() {
        let exc = mb_value_error("test");
        let exc_type = MbValue::from_ptr(MbObject::new_str("Exception".into()));
        assert_eq!(
            mb_exception_matches(exc, exc_type).as_bool(),
            Some(true),
            "try/except Exception should catch ValueError via inheritance"
        );
    }

    // ── Acceptance Scenario: Except specific mismatch ──

    #[test]
    fn test_scenario_except_type_error_does_not_catch_value_error() {
        let exc = mb_value_error("test");
        let te = MbValue::from_ptr(MbObject::new_str("TypeError".into()));
        assert_eq!(
            mb_exception_matches(exc, te).as_bool(),
            Some(false),
            "try/except TypeError should NOT catch ValueError"
        );
    }

    // ── Acceptance Scenario: except* catches matching exceptions ──

    #[test]
    fn test_scenario_except_star_catches_value_error_from_group() {
        let msg = MbValue::from_ptr(MbObject::new_str("errors".into()));
        let e1 = mb_value_error("v");
        let e2 = mb_type_error("t");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2]));
        let group = mb_exception_group_new(msg, exceptions);

        let target = MbValue::from_ptr(MbObject::new_str("ValueError".into()));
        let result = mb_except_star(group, target);

        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                let matched = items[0];
                let rest = items[1];

                // Handler catches ValueError
                assert!(
                    !matched.is_none(),
                    "except* ValueError should catch ValueError from group"
                );
                let matched_excs = mb_exception_group_exceptions(matched);
                if let Some(mptr) = matched_excs.as_ptr() {
                    if let ObjData::Tuple(ref mitems) = (*mptr).data {
                        assert_eq!(mitems.len(), 1);
                        assert_eq!(get_exception_type(mitems[0]).unwrap(), "ValueError");
                    }
                }

                // Remaining TypeError propagates
                assert!(
                    !rest.is_none(),
                    "TypeError should remain/propagate after except* ValueError"
                );
                let rest_excs = mb_exception_group_exceptions(rest);
                if let Some(rptr) = rest_excs.as_ptr() {
                    if let ObjData::Tuple(ref ritems) = (*rptr).data {
                        assert_eq!(ritems.len(), 1);
                        assert_eq!(get_exception_type(ritems[0]).unwrap(), "TypeError");
                    }
                }
            }
        }
    }

    #[test]
    fn test_except_star_no_match_returns_none_matched() {
        let msg = MbValue::from_ptr(MbObject::new_str("group".into()));
        let e1 = mb_value_error("v");
        let e2 = mb_type_error("t");
        let exceptions = MbValue::from_ptr(MbObject::new_list(vec![e1, e2]));
        let group = mb_exception_group_new(msg, exceptions);

        let target = MbValue::from_ptr(MbObject::new_str("KeyError".into()));
        let result = mb_except_star(group, target);

        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert!(
                    items[0].is_none(),
                    "matched must be None when no exceptions match"
                );
                assert!(
                    !items[1].is_none(),
                    "rest must contain all original exceptions"
                );
            }
        }
    }

    // ── mb_name_error constructor ──

    #[test]
    fn test_name_error_constructor() {
        let exc = mb_name_error("name 'x' is not defined");
        assert_eq!(get_exception_type(exc).unwrap(), "NameError");
        assert_eq!(
            get_exception_message(exc).unwrap(),
            "name 'x' is not defined"
        );
    }

    // ── set_current_exception / clear_current_exception (public API) ──

    #[test]
    fn test_set_current_exception_directly() {
        mb_clear_exception();
        let exc = MbException::new("AttributeError", "no such attr");
        set_current_exception(exc);
        assert_eq!(mb_has_exception().as_bool(), Some(true));
        let caught = mb_catch_exception();
        assert_eq!(get_exception_type(caught).unwrap(), "AttributeError");
        assert_eq!(get_exception_message(caught).unwrap(), "no such attr");
    }

    #[test]
    fn test_clear_current_exception_directly() {
        let exc = MbException::new("RuntimeError", "test");
        set_current_exception(exc);
        assert_eq!(mb_has_exception().as_bool(), Some(true));
        clear_current_exception();
        assert_eq!(mb_has_exception().as_bool(), Some(false));
    }

    // ── MbException struct methods ──

    #[test]
    fn test_mb_exception_with_context() {
        let context = MbException::new("ValueError", "ctx");
        let exc = MbException::new("RuntimeError", "main").with_context(context);
        assert!(exc.context.is_some());
        assert_eq!(exc.context.as_ref().unwrap().exc_type, "ValueError");
        assert_eq!(exc.context.as_ref().unwrap().message, "ctx");
        assert!(!exc.suppress_context);
    }

    #[test]
    fn test_mb_exception_traceback_default_empty() {
        let exc = MbException::new("Exception", "test");
        assert!(exc.traceback.is_empty());
        assert!(exc.cause.is_none());
        assert!(exc.context.is_none());
        assert!(!exc.suppress_context);
    }
}
