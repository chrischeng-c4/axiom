use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// unittest.mock module for Mamba.
///
/// Mock engine: mocks are Instances (class "MagicMock" / "Mock" /
/// "AsyncMock") with
/// - call recording (`call_count`, `called`, `call_args`, `call_args_list`,
///   `mock_calls` with ancestor propagation),
/// - `return_value` / `side_effect` (value, exception instance, callable, or
///   iterable) configuration via constructor kwargs or attribute writes,
/// - attribute autovivification of child mocks through a registered
///   `__getattr__` (same child on repeated access),
/// - assertion helpers raising AssertionError with CPython-shaped messages,
/// - MagicMock magic methods (`__enter__`/`__exit__`/`__len__`/`__bool__`),
/// - `call` objects with name/args/kwargs equality (and `call.a(...)` named
///   forms), the `ANY` always-equal sentinel,
/// - `patch(target, ...)` as context manager / decorator / start-stop pair
///   rebinding a module attribute, plus `patch.dict` and `patch.object`,
/// - `seal()` and `Mock(spec=...)` attribute restriction, and `PropertyMock`
///   as a recording descriptor.
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

// ── small helpers ─────────────────────────────────────────────────────────────

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
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

fn make_instance(class_name: &str, fields_kv: Vec<(&str, MbValue)>) -> MbValue {
    let mut fields = FxHashMap::default();
    for (k, v) in fields_kv {
        fields.insert(k.to_string(), v);
    }
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

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
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

fn instance_class(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

/// Is this class name one of the mock instance classes?
pub(crate) fn is_mock_class(name: &str) -> bool {
    matches!(
        name,
        "Mock" | "MagicMock" | "AsyncMock" | "NonCallableMock" | "PropertyMock"
    )
}

fn is_dict_value(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Split a variadic args_list into (positional args, kwargs dict or None).
/// mamba folds keyword args into one trailing dict positional.
fn split_call_args(args_list: MbValue) -> (Vec<MbValue>, MbValue) {
    let items: Vec<MbValue> = args_list
        .as_ptr()
        .and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock.read().ok().map(|g| g.to_vec()),
                ObjData::Tuple(items) => Some(items.clone()),
                _ => None,
            }
        })
        .unwrap_or_default();
    if let Some(last) = items.last().copied() {
        if is_dict_value(last) {
            return (items[..items.len() - 1].to_vec(), last);
        }
    }
    (items, MbValue::none())
}

fn kwarg_get(kwargs: MbValue, name: &str) -> Option<MbValue> {
    if kwargs.is_none() {
        return None;
    }
    let sentinel = MbValue::from_bits(u64::MAX);
    let v = super::super::dict_ops::mb_dict_get(kwargs, new_str(name), sentinel);
    if v.to_bits() == u64::MAX {
        None
    } else {
        Some(v)
    }
}

/// Null-safe arg-slice view (zero-arg calls may pass a null pointer).
unsafe fn arg_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

fn raise(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
    MbValue::none()
}

fn list_items(v: MbValue) -> Vec<MbValue> {
    v.as_ptr()
        .and_then(|p| unsafe {
            match &(*p).data {
                ObjData::List(lock) => lock.read().ok().map(|g| g.to_vec()),
                ObjData::Tuple(items) => Some(items.clone()),
                _ => None,
            }
        })
        .unwrap_or_default()
}

// ── call objects ──────────────────────────────────────────────────────────────

/// Build a `call` object: name "" for direct calls, "a"/"a.b" for child calls.
fn make_call(name: &str, pos: Vec<MbValue>, kwargs: MbValue) -> MbValue {
    let kw = if kwargs.is_none() {
        MbValue::from_ptr(MbObject::new_dict())
    } else {
        kwargs
    };
    make_instance(
        "call",
        vec![
            ("_call_name", new_str(name)),
            ("args", MbValue::from_ptr(MbObject::new_tuple(pos))),
            ("kwargs", kw),
        ],
    )
}

fn call_parts(c: MbValue) -> Option<(String, MbValue, MbValue)> {
    if instance_class(c).as_deref() != Some("call") {
        return None;
    }
    Some((
        get_field(c, "_call_name")
            .and_then(extract_str)
            .unwrap_or_default(),
        get_field(c, "args").unwrap_or_else(MbValue::none),
        get_field(c, "kwargs").unwrap_or_else(MbValue::none),
    ))
}

/// Element compare with `expected` as the lhs so an ANY sentinel on the
/// expected side matches anything via its registered __eq__.
fn values_equal(expected: MbValue, actual: MbValue) -> bool {
    // Dunder-aware equality (mb_eq alone skips instance __eq__, so the ANY
    // sentinel would never match); opcode 7 is `eq` in the binop table.
    super::super::class::mb_dispatch_binop(7, expected, actual).as_bool() == Some(true)
}

/// call equality with `expected` as the comparison lhs.
fn calls_equal(expected: MbValue, actual: MbValue) -> bool {
    let (Some((na, aa, ka)), Some((nb, ab, kb))) = (call_parts(expected), call_parts(actual))
    else {
        return false;
    };
    if na != nb {
        return false;
    }
    // Positional args: element-wise, expected side first.
    let ea = list_items(aa);
    let ab_items = list_items(ab);
    if ea.len() != ab_items.len() {
        return false;
    }
    if !ea
        .iter()
        .zip(ab_items.iter())
        .all(|(e, a)| values_equal(*e, *a))
    {
        return false;
    }
    // Kwargs: same key sets, expected value as lhs per key.
    let ek = list_items(super::super::dict_ops::mb_dict_items(ka));
    let akm = kb;
    if super::super::dict_ops::mb_dict_len(ka).as_int()
        != super::super::dict_ops::mb_dict_len(kb).as_int()
    {
        return false;
    }
    for pair in ek {
        let kv = list_items(pair);
        if kv.len() != 2 {
            return false;
        }
        let sentinel = MbValue::from_bits(u64::MAX);
        let av = super::super::dict_ops::mb_dict_get(akm, kv[0], sentinel);
        if av.to_bits() == u64::MAX || !values_equal(kv[1], av) {
            return false;
        }
    }
    true
}

unsafe extern "C" fn call_eq(self_v: MbValue, other: MbValue) -> MbValue {
    if instance_class(other).as_deref() != Some("call") {
        return MbValue::not_implemented();
    }
    MbValue::from_bool(calls_equal(self_v, other))
}

/// Render a call object as CPython does: `call(1, 2)` / `call.a(k=3)`.
fn call_repr(c: MbValue) -> String {
    let Some((name, args, kwargs)) = call_parts(c) else {
        return "call".to_string();
    };
    let mut parts: Vec<String> = Vec::new();
    for it in list_items(args) {
        let r = super::super::builtins::mb_repr(it);
        parts.push(extract_str(r).unwrap_or_default());
    }
    for pair in list_items(super::super::dict_ops::mb_dict_items(kwargs)) {
        let kv = list_items(pair);
        if kv.len() == 2 {
            let k = extract_str(kv[0]).unwrap_or_default();
            let v = extract_str(super::super::builtins::mb_repr(kv[1])).unwrap_or_default();
            parts.push(format!("{k}={v}"));
        }
    }
    if name.is_empty() {
        format!("call({})", parts.join(", "))
    } else {
        format!("call.{name}({})", parts.join(", "))
    }
}

/// `unittest.mock.call(...)` factory.
unsafe extern "C" fn dispatch_call_factory(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let list = MbValue::from_ptr(MbObject::new_list(a.to_vec()));
    let (pos, kw) = split_call_args(list);
    make_call("", pos, kw)
}

/// `call.a` name-builder: calling it produces a named call object.
pub(crate) fn make_call_namebuilder(name: &str) -> MbValue {
    make_instance("_call_namebuilder", vec![("_call_name", new_str(name))])
}

/// `call.a(1)` lowered as a method call on the factory: build a named call.
pub(crate) fn make_named_call(name: &str, args_list: MbValue) -> MbValue {
    let (pos, kw) = split_call_args(args_list);
    make_call(name, pos, kw)
}

unsafe extern "C" fn call_namebuilder_call(self_v: MbValue, args_list: MbValue) -> MbValue {
    let name = get_field(self_v, "_call_name")
        .and_then(extract_str)
        .unwrap_or_default();
    let (pos, kw) = split_call_args(args_list);
    make_call(&name, pos, kw)
}

// ── mock construction ─────────────────────────────────────────────────────────

fn build_mock(class_name: &str, name: &str) -> MbValue {
    make_instance(
        class_name,
        vec![
            ("_mock_name", new_str(name)),
            ("call_count", MbValue::from_int(0)),
            ("called", MbValue::from_bool(false)),
            ("call_args", MbValue::none()),
            (
                "call_args_list",
                MbValue::from_ptr(MbObject::new_list(Vec::new())),
            ),
            (
                "mock_calls",
                MbValue::from_ptr(MbObject::new_list(Vec::new())),
            ),
            ("side_effect", MbValue::none()),
            ("await_count", MbValue::from_int(0)),
        ],
    )
}

/// Apply constructor kwargs (return_value / side_effect / spec / name).
fn apply_mock_kwargs(mock: MbValue, kwargs: MbValue) {
    if kwargs.is_none() {
        return;
    }
    for key in ["return_value", "side_effect", "name"] {
        if let Some(v) = kwarg_get(kwargs, key) {
            let field = if key == "name" { "_mock_name" } else { key };
            set_field(mock, field, v);
        }
    }
    for key in ["spec", "spec_set"] {
        if let Some(spec) = kwarg_get(kwargs, key) {
            if let Some(cls) = super::super::class::resolve_class_name(spec) {
                let names = super::super::class::mb_dir_mro_keys(&cls);
                let items: Vec<MbValue> = names.iter().map(|n| new_str(n)).collect();
                set_field(
                    mock,
                    "_mock_methods",
                    MbValue::from_ptr(MbObject::new_list(items)),
                );
                set_field(mock, "_spec_class", new_str(&cls));
                if key == "spec_set" {
                    set_field(mock, "_spec_set", MbValue::from_bool(true));
                }
            }
        }
    }
}

fn str_list(names: Vec<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(
        names.into_iter().map(|n| new_str(&n)).collect(),
    ))
}

unsafe extern "C" fn dispatch_create_autospec(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let spec = a.first().copied().unwrap_or_else(MbValue::none);
    let m = build_mock("MagicMock", "mock");
    set_field(m, "_spec_target", spec);

    if let Some(params) = super::super::closure::func_params(spec) {
        let positional: Vec<String> = params
            .iter()
            .filter(|p| p.kind <= 1)
            .map(|p| p.name.clone())
            .collect();
        let required: Vec<String> = params
            .iter()
            .filter(|p| p.kind <= 1 && !p.has_default)
            .map(|p| p.name.clone())
            .collect();
        let has_varargs = params.iter().any(|p| p.kind == 2);
        set_field(m, "_autospec_positional", str_list(positional.clone()));
        set_field(m, "_autospec_required", str_list(required));
        set_field(
            m,
            "_autospec_max_pos",
            MbValue::from_int(if has_varargs { -1 } else { positional.len() as i64 }),
        );
    }
    m
}

unsafe extern "C" fn dispatch_magic_mock(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let m = build_mock("MagicMock", "mock");
    if let Some(kw) = a.iter().copied().find(|v| is_dict_value(*v)) {
        apply_mock_kwargs(m, kw);
    }
    m
}

unsafe extern "C" fn dispatch_plain_mock(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let m = build_mock("Mock", "mock");
    if let Some(kw) = a.iter().copied().find(|v| is_dict_value(*v)) {
        apply_mock_kwargs(m, kw);
    }
    m
}

unsafe extern "C" fn dispatch_async_mock(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let m = build_mock("AsyncMock", "mock");
    if let Some(kw) = a.iter().copied().find(|v| is_dict_value(*v)) {
        apply_mock_kwargs(m, kw);
    }
    m
}

unsafe extern "C" fn dispatch_property_mock(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let m = build_mock("PropertyMock", "mock");
    if let Some(kw) = a.iter().copied().find(|v| is_dict_value(*v)) {
        apply_mock_kwargs(m, kw);
    }
    m
}

// ── attribute autovivification ────────────────────────────────────────────────

/// Magic dunders MagicMock children may autovivify for.
fn is_supported_magic(name: &str) -> bool {
    matches!(
        name,
        "__enter__"
            | "__exit__"
            | "__len__"
            | "__iter__"
            | "__next__"
            | "__bool__"
            | "__contains__"
            | "__getitem__"
            | "__setitem__"
            | "__aenter__"
            | "__aexit__"
    )
}

/// Fetch (autovivifying) the child mock for `name` on `parent`.
pub(crate) fn mock_attr_child(parent: MbValue, name: &str) -> MbValue {
    if let Some(existing) = get_field(parent, name) {
        return existing;
    }
    let parent_cls = instance_class(parent).unwrap_or_else(|| "MagicMock".to_string());
    // seal() blocks NEW children.
    if get_field(parent, "_sealed").and_then(|v| v.as_bool()) == Some(true) {
        return raise(
            "AttributeError",
            &format!("Mock object has no attribute '{name}'"),
        );
    }
    // spec restriction: only declared names autovivify.
    if let Some(methods) = get_field(parent, "_mock_methods") {
        if !methods.is_none() && !name.starts_with("__") {
            let allowed = list_items(methods)
                .iter()
                .any(|v| extract_str(*v).as_deref() == Some(name));
            if !allowed {
                return raise(
                    "AttributeError",
                    &format!("Mock object has no attribute '{name}'"),
                );
            }
        }
    }
    if name.starts_with("__") && name.ends_with("__") && !is_supported_magic(name) {
        // Unsupported dunder: behave like a missing attribute (no mock child),
        // so internal dunder probes keep their default behavior.
        return MbValue::none();
    }
    let child_cls = if parent_cls == "AsyncMock" {
        "MagicMock"
    } else {
        parent_cls.as_str()
    };
    let child = build_mock(child_cls, name);
    set_field(child, "_mock_parent", parent);
    set_field(parent, name, child);
    child
}

/// spec_set guard for attribute WRITES: a mock built with spec_set=cls only
/// accepts declared names. Returns true when the write must be rejected (an
/// AttributeError has been raised).
pub(crate) fn mock_setattr_blocked(obj: MbValue, name: &str) -> bool {
    if name.starts_with('_') {
        return false;
    }
    if get_field(obj, "_spec_set").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let Some(methods) = get_field(obj, "_mock_methods") else {
        return false;
    };
    let allowed = list_items(methods)
        .iter()
        .any(|v| extract_str(*v).as_deref() == Some(name));
    if !allowed {
        raise(
            "AttributeError",
            &format!("Mock object has no attribute '{name}'"),
        );
        return true;
    }
    false
}

/// Early getattr hook for mock instances: instance fields win, then
/// return_value / supported magic names autovivify children BEFORE the class
/// method table would shadow them (m.__len__ must be the configurable child
/// mock, not the registered __len__ implementation). Returns None to let the
/// normal resolution (class methods, then __getattr__ autovivify) continue.
pub(crate) fn mock_getattr_hook(obj: MbValue, name: &str) -> Option<MbValue> {
    if let Some(v) = get_field(obj, name) {
        return Some(v);
    }
    if name == "return_value" {
        let child = build_mock("MagicMock", "()");
        set_field(child, "_mock_parent", obj);
        set_field(obj, "return_value", child);
        return Some(child);
    }
    if is_supported_magic(name) {
        return Some(mock_attr_child(obj, name));
    }
    None
}

/// Registered `__getattr__(self, name)` for mock classes (fixed 2-arg ABI).
unsafe extern "C" fn mock_getattr(self_v: MbValue, name_v: MbValue) -> MbValue {
    let name = extract_str(name_v).unwrap_or_default();
    if name == "return_value" {
        // First read autovivifies the default return-value mock named "()".
        if let Some(existing) = get_field(self_v, "return_value") {
            return existing;
        }
        let child = build_mock("MagicMock", "()");
        set_field(child, "_mock_parent", self_v);
        set_field(self_v, "return_value", child);
        return child;
    }
    mock_attr_child(self_v, &name)
}

// ── call recording ────────────────────────────────────────────────────────────

fn push_to_list_field(inst: MbValue, field: &str, val: MbValue) {
    if let Some(list) = get_field(inst, field) {
        super::super::list_ops::mb_list_append(list, val);
    }
}

fn validate_autospec_call(mock: MbValue, args_list: MbValue) -> Option<MbValue> {
    let positional_names: Vec<String> = get_field(mock, "_autospec_positional")
        .map(list_items)
        .unwrap_or_default()
        .into_iter()
        .filter_map(extract_str)
        .collect();
    if positional_names.is_empty() && get_field(mock, "_autospec_required").is_none() {
        return None;
    }

    let (pos, kwargs) = split_call_args(args_list);
    if let Some(max_pos) = get_field(mock, "_autospec_max_pos").and_then(|v| v.as_int()) {
        if max_pos >= 0 && pos.len() > max_pos as usize {
            return Some(raise("TypeError", "too many positional arguments"));
        }
    }

    for (idx, name) in positional_names.iter().enumerate() {
        if idx < pos.len() && kwarg_get(kwargs, name).is_some() {
            return Some(raise(
                "TypeError",
                &format!("multiple values for argument '{name}'"),
            ));
        }
    }

    let required: Vec<String> = get_field(mock, "_autospec_required")
        .map(list_items)
        .unwrap_or_default()
        .into_iter()
        .filter_map(extract_str)
        .collect();
    for name in required {
        let filled_by_pos = positional_names
            .iter()
            .position(|n| n == &name)
            .map(|idx| idx < pos.len())
            .unwrap_or(false);
        let filled_by_kw = kwarg_get(kwargs, &name).is_some();
        if !filled_by_pos && !filled_by_kw {
            return Some(raise(
                "TypeError",
                &format!("missing a required argument: '{name}'"),
            ));
        }
    }
    None
}

/// Dotted name of `mock` relative to `stop_at` ancestor, e.g. "a" or "a.b".
fn dotted_name_to(mock: MbValue, stop_at: MbValue) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut cur = mock;
    loop {
        if cur.to_bits() == stop_at.to_bits() {
            break;
        }
        let n = get_field(cur, "_mock_name")
            .and_then(extract_str)
            .unwrap_or_default();
        parts.push(n);
        match get_field(cur, "_mock_parent") {
            Some(p) if !p.is_none() => cur = p,
            _ => break,
        }
    }
    parts.reverse();
    parts.join(".")
}

/// Record a call on `mock` and produce the return value.
pub(crate) fn mock_record_call(mock: MbValue, args_list: MbValue) -> MbValue {
    if let Some(err) = validate_autospec_call(mock, args_list) {
        return err;
    }
    let (pos, kw) = split_call_args(args_list);
    let n = get_field(mock, "call_count")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    set_field(mock, "call_count", MbValue::from_int(n + 1));
    set_field(mock, "called", MbValue::from_bool(true));
    let c = make_call("", pos.clone(), kw);
    set_field(mock, "call_args", c);
    push_to_list_field(mock, "call_args_list", c);
    push_to_list_field(mock, "mock_calls", c);
    if instance_class(mock).as_deref() == Some("AsyncMock") {
        let aw = get_field(mock, "await_count")
            .and_then(|v| v.as_int())
            .unwrap_or(0);
        set_field(mock, "await_count", MbValue::from_int(aw + 1));
    }
    // Propagate into each ancestor's mock_calls with the relative dotted name.
    let mut ancestor = get_field(mock, "_mock_parent");
    while let Some(anc) = ancestor {
        if anc.is_none() {
            break;
        }
        let rel = dotted_name_to(mock, anc);
        push_to_list_field(anc, "mock_calls", make_call(&rel, pos.clone(), kw));
        ancestor = get_field(anc, "_mock_parent");
    }
    // side_effect: exception instance → raise; iterable → next element per
    // call; callable → call it (its result is returned).
    let se = get_field(mock, "side_effect").unwrap_or_else(MbValue::none);
    if !se.is_none() {
        if let Some(cls) = instance_class(se) {
            if super::super::exception::is_subclass_of(&cls, "BaseException")
                || super::super::exception::is_subclass_of(&cls, "Exception")
                || cls == "Exception"
                || cls == "BaseException"
            {
                let msg = extract_str(super::super::builtins::mb_str(se)).unwrap_or_default();
                super::super::exception::mb_raise(new_str(&cls), new_str(&msg));
                return MbValue::none();
            }
        }
        let is_seq = se
            .as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::List(_) | ObjData::Tuple(_)) })
            .unwrap_or(false);
        if is_seq {
            let idx = get_field(mock, "_se_idx")
                .and_then(|v| v.as_int())
                .unwrap_or(0);
            set_field(mock, "_se_idx", MbValue::from_int(idx + 1));
            let items = list_items(se);
            return match items.get(idx as usize) {
                Some(v) => *v,
                None => raise("StopIteration", ""),
            };
        }
        if super::super::builtins::mb_callable(se).as_bool() == Some(true) {
            let pos_list = MbValue::from_ptr(MbObject::new_list(pos));
            return super::super::builtins::mb_call_spread(se, pos_list);
        }
    }
    // return_value (autovivified on first use).
    match get_field(mock, "return_value") {
        Some(rv) => rv,
        None => {
            let child = build_mock("MagicMock", "()");
            set_field(child, "_mock_parent", mock);
            set_field(mock, "return_value", child);
            child
        }
    }
}

unsafe extern "C" fn mock_dunder_call(self_v: MbValue, args_list: MbValue) -> MbValue {
    mock_record_call(self_v, args_list)
}

// ── assertion helpers ─────────────────────────────────────────────────────────

fn mock_display_name(mock: MbValue) -> String {
    get_field(mock, "_mock_name")
        .and_then(extract_str)
        .unwrap_or_else(|| "mock".to_string())
}

unsafe extern "C" fn mock_assert_called(self_v: MbValue, _args: MbValue) -> MbValue {
    if get_field(self_v, "called").and_then(|v| v.as_bool()) != Some(true) {
        let n = mock_display_name(self_v);
        return raise(
            "AssertionError",
            &format!("Expected '{n}' to have been called."),
        );
    }
    MbValue::none()
}

unsafe extern "C" fn mock_assert_called_once(self_v: MbValue, _args: MbValue) -> MbValue {
    let n = get_field(self_v, "call_count")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    if n != 1 {
        let name = mock_display_name(self_v);
        return raise(
            "AssertionError",
            &format!("Expected '{name}' to have been called once. Called {n} times."),
        );
    }
    MbValue::none()
}

fn assert_called_with_inner(self_v: MbValue, args_list: MbValue) -> MbValue {
    let (pos, kw) = split_call_args(args_list);
    let expected = make_call("", pos, kw);
    let actual = get_field(self_v, "call_args").unwrap_or_else(MbValue::none);
    if actual.is_none() {
        let name = mock_display_name(self_v);
        return raise(
            "AssertionError",
            &format!(
                "expected call not found.\nExpected: {}\n  Actual: not called.",
                call_repr(expected).replacen("call", &name, 1)
            ),
        );
    }
    if !calls_equal(expected, actual) {
        let name = mock_display_name(self_v);
        return raise(
            "AssertionError",
            &format!(
                "expected call not found.\nExpected: {}\n  Actual: {}",
                call_repr(expected).replacen("call", &name, 1),
                call_repr(actual).replacen("call", &name, 1)
            ),
        );
    }
    MbValue::none()
}

unsafe extern "C" fn mock_assert_called_with(self_v: MbValue, args_list: MbValue) -> MbValue {
    assert_called_with_inner(self_v, args_list)
}

unsafe extern "C" fn mock_assert_called_once_with(self_v: MbValue, args_list: MbValue) -> MbValue {
    let n = get_field(self_v, "call_count")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    if n != 1 {
        let name = mock_display_name(self_v);
        return raise(
            "AssertionError",
            &format!("Expected '{name}' to be called once. Called {n} times."),
        );
    }
    assert_called_with_inner(self_v, args_list)
}

unsafe extern "C" fn mock_assert_not_called(self_v: MbValue, _args: MbValue) -> MbValue {
    let n = get_field(self_v, "call_count")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    if n != 0 {
        let name = mock_display_name(self_v);
        return raise(
            "AssertionError",
            &format!("Expected '{name}' to not have been called. Called {n} times."),
        );
    }
    MbValue::none()
}

unsafe extern "C" fn mock_assert_any_call(self_v: MbValue, args_list: MbValue) -> MbValue {
    let (pos, kw) = split_call_args(args_list);
    let expected = make_call("", pos, kw);
    let list = get_field(self_v, "call_args_list").unwrap_or_else(MbValue::none);
    if !list_items(list).iter().any(|c| calls_equal(expected, *c)) {
        let name = mock_display_name(self_v);
        return raise(
            "AssertionError",
            &format!(
                "{} call not found",
                call_repr(expected).replacen("call", &name, 1)
            ),
        );
    }
    MbValue::none()
}

unsafe extern "C" fn mock_assert_awaited_once(self_v: MbValue, _args: MbValue) -> MbValue {
    let n = get_field(self_v, "await_count")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    if n != 1 {
        return raise(
            "AssertionError",
            &format!("Expected mock to have been awaited once. Awaited {n} times."),
        );
    }
    MbValue::none()
}

unsafe extern "C" fn mock_reset_mock(self_v: MbValue, _args: MbValue) -> MbValue {
    set_field(self_v, "call_count", MbValue::from_int(0));
    set_field(self_v, "called", MbValue::from_bool(false));
    set_field(self_v, "call_args", MbValue::none());
    set_field(
        self_v,
        "call_args_list",
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    set_field(
        self_v,
        "mock_calls",
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    set_field(self_v, "await_count", MbValue::from_int(0));
    MbValue::none()
}

unsafe extern "C" fn mock_configure_mock(self_v: MbValue, args_list: MbValue) -> MbValue {
    let (_pos, kw) = split_call_args(args_list);
    apply_mock_kwargs(self_v, kw);
    MbValue::none()
}

// ── MagicMock magic methods ───────────────────────────────────────────────────

unsafe extern "C" fn magic_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    let child = mock_attr_child(self_v, "__enter__");
    if child.is_none() {
        return self_v;
    }
    mock_record_call(child, MbValue::from_ptr(MbObject::new_list(Vec::new())))
}

unsafe extern "C" fn magic_exit(self_v: MbValue, _args: MbValue) -> MbValue {
    let _ = self_v;
    MbValue::from_bool(false)
}

unsafe extern "C" fn magic_len(self_v: MbValue, _args: MbValue) -> MbValue {
    // len(m) reads the configured __len__ child's return_value (default 0).
    if let Some(child) = get_field(self_v, "__len__") {
        if let Some(rv) = get_field(child, "return_value") {
            if let Some(n) = rv.as_int() {
                return MbValue::from_int(n);
            }
        }
    }
    MbValue::from_int(0)
}

unsafe extern "C" fn magic_bool(self_v: MbValue, _args: MbValue) -> MbValue {
    let _ = self_v;
    MbValue::from_bool(true)
}

// ── ANY sentinel ──────────────────────────────────────────────────────────────

unsafe extern "C" fn any_eq(_self_v: MbValue, _other: MbValue) -> MbValue {
    MbValue::from_bool(true)
}

unsafe extern "C" fn any_ne(_self_v: MbValue, _other: MbValue) -> MbValue {
    MbValue::from_bool(false)
}

// ── PropertyMock descriptor ───────────────────────────────────────────────────

/// `__get__(desc, instance, objtype)` — fixed 3-arg descriptor ABI.
unsafe extern "C" fn property_mock_get(
    desc: MbValue,
    _instance: MbValue,
    _objtype: MbValue,
) -> MbValue {
    mock_record_call(desc, MbValue::from_ptr(MbObject::new_list(Vec::new())))
}

// ── seal ──────────────────────────────────────────────────────────────────────

unsafe extern "C" fn dispatch_seal(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    if let Some(m) = a.first().copied() {
        seal_recursive(m, 0);
    }
    MbValue::none()
}

fn seal_recursive(mock: MbValue, depth: usize) {
    if depth > 16 {
        return;
    }
    if !instance_class(mock)
        .map(|c| is_mock_class(&c))
        .unwrap_or(false)
    {
        return;
    }
    set_field(mock, "_sealed", MbValue::from_bool(true));
    // Seal existing children too (CPython seals recursively).
    let children: Vec<MbValue> = mock
        .as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .read()
                    .unwrap()
                    .iter()
                    .filter(|(k, _)| !k.starts_with('_') && *k != "return_value")
                    .map(|(_, v)| *v)
                    .collect()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default();
    for c in children {
        seal_recursive(c, depth + 1);
    }
}

// ── patch ─────────────────────────────────────────────────────────────────────

/// `patch(target, new=..., return_value=..., ...)` → a _patch instance.
unsafe extern "C" fn dispatch_patch(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let target = a.first().copied().unwrap_or_else(MbValue::none);
    let kwargs = a
        .iter()
        .copied()
        .find(|v| is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    let new = a
        .get(1)
        .copied()
        .filter(|v| !is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    make_instance(
        "_patch",
        vec![
            ("_target", target),
            ("_new", new),
            ("_kwargs", kwargs),
            ("_saved", MbValue::none()),
            ("_had", MbValue::from_bool(false)),
            ("_active_mock", MbValue::none()),
        ],
    )
}

/// Apply the patch: rebind the module attribute, remember the original.
fn patch_apply(self_v: MbValue) -> MbValue {
    let target = get_field(self_v, "_target")
        .and_then(extract_str)
        .unwrap_or_default();
    let Some((mod_path, attr)) = target.rsplit_once('.') else {
        return raise(
            "TypeError",
            &format!("Need a valid target to patch. You supplied: {target:?}"),
        );
    };
    let module_val = super::super::module::mb_import(new_str(mod_path));
    if module_val.is_none() {
        return raise(
            "ModuleNotFoundError",
            &format!("No module named '{mod_path}'"),
        );
    }
    let sentinel = MbValue::from_bits(u64::MAX);
    let prev = super::super::dict_ops::mb_dict_get(module_val, new_str(attr), sentinel);
    let had = prev.to_bits() != u64::MAX;
    set_field(self_v, "_saved", if had { prev } else { MbValue::none() });
    set_field(self_v, "_had", MbValue::from_bool(had));
    set_field(self_v, "_module", module_val);
    set_field(self_v, "_attr", new_str(attr));
    // Replacement: explicit `new` if given, else a MagicMock configured with
    // the patch kwargs (return_value etc.).
    let new_v = get_field(self_v, "_new").unwrap_or_else(MbValue::none);
    let replacement = if new_v.is_none() {
        let m = build_mock("MagicMock", attr);
        apply_mock_kwargs(
            m,
            get_field(self_v, "_kwargs").unwrap_or_else(MbValue::none),
        );
        m
    } else {
        new_v
    };
    set_field(self_v, "_active_mock", replacement);
    super::super::dict_ops::mb_dict_setitem(module_val, new_str(attr), replacement);
    replacement
}

fn patch_restore(self_v: MbValue) {
    let module_val = get_field(self_v, "_module").unwrap_or_else(MbValue::none);
    let attr = get_field(self_v, "_attr")
        .and_then(extract_str)
        .unwrap_or_default();
    if module_val.is_none() || attr.is_empty() {
        return;
    }
    let had = get_field(self_v, "_had")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if had {
        let saved = get_field(self_v, "_saved").unwrap_or_else(MbValue::none);
        super::super::dict_ops::mb_dict_setitem(module_val, new_str(&attr), saved);
    } else {
        super::super::dict_ops::mb_dict_delitem(module_val, new_str(&attr));
    }
}

unsafe extern "C" fn patch_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    patch_apply(self_v)
}

unsafe extern "C" fn patch_exit(self_v: MbValue, _args: MbValue) -> MbValue {
    patch_restore(self_v);
    MbValue::from_bool(false)
}

/// `@patch(...)` decorator form: wraps the function, appending the mock.
unsafe extern "C" fn patch_call(self_v: MbValue, args_list: MbValue) -> MbValue {
    let (pos, _kw) = split_call_args(args_list);
    let func = pos.first().copied().unwrap_or_else(MbValue::none);
    make_instance("_patch_wrapper", vec![("_patch", self_v), ("_func", func)])
}

unsafe extern "C" fn patch_wrapper_call(self_v: MbValue, args_list: MbValue) -> MbValue {
    let patcher = get_field(self_v, "_patch").unwrap_or_else(MbValue::none);
    let func = get_field(self_v, "_func").unwrap_or_else(MbValue::none);
    let mock = patch_apply(patcher);
    let (mut pos, _kw) = split_call_args(args_list);
    pos.push(mock);
    let call_list = MbValue::from_ptr(MbObject::new_list(pos));
    let result = super::super::builtins::mb_call_spread(func, call_list);
    patch_restore(patcher);
    result
}

/// `patch.dict(d, values, clear=False)` → context manager.
unsafe extern "C" fn dispatch_patch_dict(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let in_dict = a.first().copied().unwrap_or_else(MbValue::none);
    let values = a.get(1).copied().unwrap_or_else(MbValue::none);
    make_instance(
        "_patch_dict",
        vec![
            ("_in_dict", in_dict),
            ("_values", values),
            ("_snapshot", MbValue::none()),
        ],
    )
}

fn dict_overwrite_from(dst: MbValue, src: MbValue) {
    for pair in list_items(super::super::dict_ops::mb_dict_items(src)) {
        let kv = list_items(pair);
        if kv.len() == 2 {
            super::super::dict_ops::mb_dict_setitem(dst, kv[0], kv[1]);
        }
    }
}

unsafe extern "C" fn patch_dict_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    let d = get_field(self_v, "_in_dict").unwrap_or_else(MbValue::none);
    let values = get_field(self_v, "_values").unwrap_or_else(MbValue::none);
    // Snapshot: shallow copy of the dict.
    let snapshot = super::super::dict_ops::mb_dict_from_pairs(d);
    set_field(self_v, "_snapshot", snapshot);
    if !values.is_none() {
        dict_overwrite_from(d, values);
    }
    d
}

unsafe extern "C" fn patch_dict_exit(self_v: MbValue, _args: MbValue) -> MbValue {
    let d = get_field(self_v, "_in_dict").unwrap_or_else(MbValue::none);
    let snapshot = get_field(self_v, "_snapshot").unwrap_or_else(MbValue::none);
    if !d.is_none() && !snapshot.is_none() {
        super::super::dict_ops::mb_dict_clear(d);
        dict_overwrite_from(d, snapshot);
    }
    MbValue::from_bool(false)
}

/// `patch.object(cls, name, new=..., return_value=...)` → context manager
/// replacing a registered class's method-table entry.
unsafe extern "C" fn dispatch_patch_object(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let target = a.first().copied().unwrap_or_else(MbValue::none);
    let attr = a.get(1).copied().unwrap_or_else(MbValue::none);
    let kwargs = a
        .iter()
        .copied()
        .find(|v| is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    make_instance(
        "_patch_object",
        vec![
            ("_cls", target),
            ("_attr_name", attr),
            ("_kwargs", kwargs),
            ("_saved", MbValue::none()),
            ("_had", MbValue::from_bool(false)),
        ],
    )
}

unsafe extern "C" fn patch_object_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    let cls_v = get_field(self_v, "_cls").unwrap_or_else(MbValue::none);
    let attr = get_field(self_v, "_attr_name")
        .and_then(extract_str)
        .unwrap_or_default();
    let Some(cls) = super::super::class::resolve_class_name(cls_v) else {
        return raise("TypeError", "patch.object target must be a class or object");
    };
    let existing = super::super::class::lookup_method(&cls, &attr);
    if existing.is_none() {
        return raise(
            "AttributeError",
            &format!("<class '{cls}'> does not have the attribute '{attr}'"),
        );
    }
    set_field(self_v, "_saved", existing);
    set_field(self_v, "_had", MbValue::from_bool(true));
    set_field(self_v, "_cls_name", new_str(&cls));
    let m = build_mock("MagicMock", &attr);
    apply_mock_kwargs(
        m,
        get_field(self_v, "_kwargs").unwrap_or_else(MbValue::none),
    );
    super::super::class::class_replace_method(&cls, &attr, m);
    m
}

unsafe extern "C" fn patch_object_exit(self_v: MbValue, _args: MbValue) -> MbValue {
    let cls = get_field(self_v, "_cls_name")
        .and_then(extract_str)
        .unwrap_or_default();
    let attr = get_field(self_v, "_attr_name")
        .and_then(extract_str)
        .unwrap_or_default();
    if !cls.is_empty() {
        let saved = get_field(self_v, "_saved").unwrap_or_else(MbValue::none);
        super::super::class::class_replace_method(&cls, &attr, saved);
    }
    MbValue::from_bool(false)
}

unsafe extern "C" fn dispatch_mock_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let kwargs = a.iter().copied().find(|v| is_dict_value(*v)).unwrap_or_else(MbValue::none);
    let read_data = kwarg_get(kwargs, "read_data")
        .unwrap_or_else(|| MbValue::from_ptr(MbObject::new_str(String::new())));
    let open_mock = build_mock("MagicMock", "open");
    let file_mock = build_mock("MagicMock", "open()");
    let read_child = mock_attr_child(file_mock, "read");
    if !read_child.is_none() {
        set_field(read_child, "return_value", read_data);
    }
    let enter_child = mock_attr_child(file_mock, "__enter__");
    if !enter_child.is_none() {
        set_field(enter_child, "return_value", file_mock);
    }
    set_field(open_mock, "return_value", file_mock);
    open_mock
}

unsafe extern "C" fn dispatch_mock_noop(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

// ── Registration ──────────────────────────────────────────────────────────────

/// Register the `unittest.mock` module.
pub fn register() {
    register_mock_classes();

    let mut attrs: HashMap<String, MbValue> = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("MagicMock", dispatch_magic_mock as *const () as usize),
        ("Mock", dispatch_plain_mock as *const () as usize),
        ("NonCallableMock", dispatch_plain_mock as *const () as usize),
        (
            "NonCallableMagicMock",
            dispatch_magic_mock as *const () as usize,
        ),
        ("AsyncMock", dispatch_async_mock as *const () as usize),
        ("PropertyMock", dispatch_property_mock as *const () as usize),
        ("create_autospec", dispatch_create_autospec as *const () as usize),
        ("patch", dispatch_patch as *const () as usize),
        ("call", dispatch_call_factory as *const () as usize),
        ("mock_open", dispatch_mock_open as *const () as usize),
        ("seal", dispatch_seal as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // The always-equal sentinel.
    attrs.insert("ANY".to_string(), make_instance("_mock_ANY", vec![]));

    // surface: missing CPython module constants (auto-added)
    attrs.insert("FILTER_DIR".into(), MbValue::from_int(1));
    attrs.insert(
        "inplace".into(),
        MbValue::from_ptr(MbObject::new_str(
            "iadd isub imul imatmul itruediv ifloordiv imod ilshift irshift iand ixor ior ipow"
                .to_string(),
        )),
    );
    attrs.insert("magic_methods".into(), MbValue::from_ptr(MbObject::new_str("lt le gt ge eq ne getitem setitem delitem len contains iter hash str sizeof enter exit divmod rdivmod neg pos abs invert complex int float index round trunc floor ceil bool next fspath aiter ".to_string())));
    attrs.insert(
        "numerics".into(),
        MbValue::from_ptr(MbObject::new_str(
            "add sub mul matmul truediv floordiv mod lshift rshift and xor or pow".to_string(),
        )),
    );
    attrs.insert(
        "right".into(),
        MbValue::from_ptr(MbObject::new_str(
            "radd rsub rmul rmatmul rtruediv rfloordiv rmod rlshift rrshift rand rxor ror rpow"
                .to_string(),
        )),
    );

    // surface: remaining CPython callables kept as present+callable stubs.
    {
        let noop = dispatch_mock_noop as *const () as usize;
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(noop as u64);
        });
        let callables: &[&str] = &[
            "AsyncMagicMixin",
            "AsyncMockMixin",
            "Base",
            "CallableMixin",
            "CodeType",
            "FunctionTypes",
            "InvalidSpecError",
            "MagicMixin",
            "MagicProxy",
            "MethodType",
            "ModuleType",
            "partial",
            "RLock",
            "iscoroutinefunction",
            "safe_repr",
            "wraps",
        ];
        for name in callables {
            attrs.insert((*name).to_string(), MbValue::from_func(noop));
        }
    }

    // surface: sentinel / module attributes (hasattr-only markers).
    for name in &[
        "DEFAULT",
        "sentinel",
        "file_spec",
        "open_spec",
        "asyncio",
        "builtins",
        "contextlib",
        "inspect",
        "io",
        "pkgutil",
        "pprint",
        "sys",
    ] {
        attrs.insert(
            (*name).to_string(),
            MbValue::from_ptr(MbObject::new_str(format!(
                "mb_mock_{}",
                name.to_lowercase()
            ))),
        );
    }

    super::register_module("unittest.mock", attrs);

    // patch.<attr> and call.<name> resolve through the func→native-class
    // bridge: map the constructor addrs to class names.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(
            dispatch_patch as *const () as usize as u64,
            "patch".to_string(),
        );
        map.insert(
            dispatch_call_factory as *const () as usize as u64,
            "_mock_call_factory".to_string(),
        );
        map.insert(
            dispatch_magic_mock as *const () as usize as u64,
            "MagicMock".to_string(),
        );
        map.insert(
            dispatch_plain_mock as *const () as usize as u64,
            "Mock".to_string(),
        );
        map.insert(
            dispatch_async_mock as *const () as usize as u64,
            "AsyncMock".to_string(),
        );
        map.insert(
            dispatch_property_mock as *const () as usize as u64,
            "PropertyMock".to_string(),
        );
    });
    {
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for (name, addr) in [
            ("dict", dispatch_patch_dict as *const () as usize),
            ("object", dispatch_patch_object as *const () as usize),
            ("multiple", dispatch_mock_noop as *const () as usize),
            ("stopall", dispatch_mock_noop as *const () as usize),
        ] {
            methods.insert(name.to_string(), MbValue::from_func(addr));
            super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
                s.borrow_mut().insert(addr as u64);
            });
        }
        super::super::class::mb_class_register("patch", vec![], methods);
    }
}

fn register_mock_classes() {
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };

    // Shared call-recording + assertion surface.
    let base_methods: Vec<(&str, MbValue)> = vec![
        ("__call__", var(mock_dunder_call as *const () as usize)),
        (
            "__getattr__",
            MbValue::from_func(mock_getattr as *const () as usize),
        ),
        (
            "assert_called",
            var(mock_assert_called as *const () as usize),
        ),
        (
            "assert_called_once",
            var(mock_assert_called_once as *const () as usize),
        ),
        (
            "assert_called_with",
            var(mock_assert_called_with as *const () as usize),
        ),
        (
            "assert_called_once_with",
            var(mock_assert_called_once_with as *const () as usize),
        ),
        (
            "assert_not_called",
            var(mock_assert_not_called as *const () as usize),
        ),
        (
            "assert_any_call",
            var(mock_assert_any_call as *const () as usize),
        ),
        (
            "assert_awaited",
            var(mock_assert_called as *const () as usize),
        ),
        (
            "assert_awaited_once",
            var(mock_assert_awaited_once as *const () as usize),
        ),
        ("reset_mock", var(mock_reset_mock as *const () as usize)),
        (
            "configure_mock",
            var(mock_configure_mock as *const () as usize),
        ),
    ];

    // Plain Mock / AsyncMock: no magic-method table (len(Mock()) is TypeError).
    for cls in ["Mock", "AsyncMock", "NonCallableMock"] {
        let mut m: Map<String, MbValue> = Map::new();
        for (k, v) in &base_methods {
            m.insert((*k).to_string(), *v);
        }
        super::super::class::mb_class_register(cls, vec![], m);
    }

    // MagicMock: base + supported magic methods.
    {
        let mut m: Map<String, MbValue> = Map::new();
        for (k, v) in &base_methods {
            m.insert((*k).to_string(), *v);
        }
        m.insert("__enter__".into(), var(magic_enter as *const () as usize));
        m.insert("__exit__".into(), var(magic_exit as *const () as usize));
        m.insert("__len__".into(), var(magic_len as *const () as usize));
        m.insert("__bool__".into(), var(magic_bool as *const () as usize));
        super::super::class::mb_class_register("MagicMock", vec![], m);
    }

    // PropertyMock: base + descriptor __get__.
    {
        let mut m: Map<String, MbValue> = Map::new();
        for (k, v) in &base_methods {
            m.insert((*k).to_string(), *v);
        }
        m.insert(
            "__get__".into(),
            MbValue::from_func(property_mock_get as *const () as usize),
        );
        super::super::class::mb_class_register("PropertyMock", vec![], m);
    }

    // call objects + name builder + ANY.
    {
        let mut m: Map<String, MbValue> = Map::new();
        m.insert(
            "__eq__".into(),
            MbValue::from_func(call_eq as *const () as usize),
        );
        super::super::class::mb_class_register("call", vec![], m);

        let mut nb: Map<String, MbValue> = Map::new();
        nb.insert(
            "__call__".into(),
            var(call_namebuilder_call as *const () as usize),
        );
        super::super::class::mb_class_register("_call_namebuilder", vec![], nb);

        let mut any: Map<String, MbValue> = Map::new();
        any.insert(
            "__eq__".into(),
            MbValue::from_func(any_eq as *const () as usize),
        );
        any.insert(
            "__ne__".into(),
            MbValue::from_func(any_ne as *const () as usize),
        );
        super::super::class::mb_class_register("_mock_ANY", vec![], any);
    }

    // patch context managers.
    {
        let mut p: Map<String, MbValue> = Map::new();
        p.insert("__enter__".into(), var(patch_enter as *const () as usize));
        p.insert("__exit__".into(), var(patch_exit as *const () as usize));
        p.insert("start".into(), var(patch_enter as *const () as usize));
        p.insert("stop".into(), var(patch_exit as *const () as usize));
        p.insert("__call__".into(), var(patch_call as *const () as usize));
        super::super::class::mb_class_register("_patch", vec![], p);

        let mut w: Map<String, MbValue> = Map::new();
        w.insert(
            "__call__".into(),
            var(patch_wrapper_call as *const () as usize),
        );
        super::super::class::mb_class_register("_patch_wrapper", vec![], w);

        let mut pd: Map<String, MbValue> = Map::new();
        pd.insert(
            "__enter__".into(),
            var(patch_dict_enter as *const () as usize),
        );
        pd.insert(
            "__exit__".into(),
            var(patch_dict_exit as *const () as usize),
        );
        super::super::class::mb_class_register("_patch_dict", vec![], pd);

        let mut po: Map<String, MbValue> = Map::new();
        po.insert(
            "__enter__".into(),
            var(patch_object_enter as *const () as usize),
        );
        po.insert(
            "__exit__".into(),
            var(patch_object_exit as *const () as usize),
        );
        po.insert(
            "start".into(),
            var(patch_object_enter as *const () as usize),
        );
        po.insert("stop".into(), var(patch_object_exit as *const () as usize));
        super::super::class::mb_class_register("_patch_object", vec![], po);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_mock_initial_state() {
        register();
        let m = unsafe { dispatch_magic_mock(std::ptr::null(), 0) };
        assert_eq!(get_field(m, "call_count").and_then(|v| v.as_int()), Some(0));
        assert_eq!(
            get_field(m, "called").and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn magic_mock_records_call() {
        register();
        let m = unsafe { dispatch_magic_mock(std::ptr::null(), 0) };
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let _ = mock_record_call(m, args);
        assert_eq!(get_field(m, "call_count").and_then(|v| v.as_int()), Some(1));
        assert_eq!(get_field(m, "called").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn mock_autovivifies_same_child() {
        register();
        let m = unsafe { dispatch_magic_mock(std::ptr::null(), 0) };
        let a = mock_attr_child(m, "foo");
        let b = mock_attr_child(m, "foo");
        assert_eq!(a.to_bits(), b.to_bits());
    }

    #[test]
    fn assert_called_once_fails_on_zero() {
        register();
        super::super::super::exception::mb_clear_exception();
        let m = unsafe { dispatch_magic_mock(std::ptr::null(), 0) };
        let none_args = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        let _ = unsafe { mock_assert_called_once(m, none_args) };
        assert_eq!(
            super::super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn calls_compare_equal_by_args() {
        register();
        let a = make_call("", vec![MbValue::from_int(1)], MbValue::none());
        let b = make_call("", vec![MbValue::from_int(1)], MbValue::none());
        let c = make_call("", vec![MbValue::from_int(2)], MbValue::none());
        assert!(calls_equal(a, b));
        assert!(!calls_equal(a, c));
    }
}
