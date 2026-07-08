use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn make_type_obj(name: &str, module: &str) -> MbValue {
    let obj = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*obj).data {
            let mut map = fields.write().unwrap();
            map.insert("__name__".to_string(), new_str(name));
            map.insert("__qualname__".to_string(), new_str(name));
            map.insert("__module__".to_string(), new_str(module));
        }
    }
    MbValue::from_ptr(obj)
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str("TypeError"), new_str(msg));
    MbValue::none()
}

fn flat_args(args_ptr: *const MbValue, nargs: usize) -> &'static [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

fn method_args(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                Some(lock.read().unwrap().to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn is_sequence(val: MbValue) -> bool {
    val.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::List(_) | ObjData::Tuple(_)) })
        .unwrap_or(false)
}

fn is_path(val: MbValue) -> bool {
    val.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Str(_) | ObjData::Bytes(_)) })
        .unwrap_or(false)
}

fn is_location(val: MbValue) -> bool {
    val.is_none()
        || val.as_int_pyint().is_some()
        || val
            .as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Tuple(_)) })
            .unwrap_or(false)
}

fn require_int(items: &[MbValue], idx: usize, name: &str) -> Option<MbValue> {
    if let Some(value) = items.get(idx) {
        if value.as_int_pyint().is_none() {
            return Some(raise_type_error(&format!("{name} must be int")));
        }
    }
    None
}

fn require_sequence(items: &[MbValue], idx: usize, name: &str) -> Option<MbValue> {
    if let Some(value) = items.get(idx) {
        if !is_sequence(*value) {
            return Some(raise_type_error(&format!("{name} must be a sequence")));
        }
    }
    None
}

fn require_path(items: &[MbValue], idx: usize, name: &str) -> Option<MbValue> {
    if let Some(value) = items.get(idx) {
        if !is_path(*value) {
            return Some(raise_type_error(&format!("{name} must be str, bytes, or path-like")));
        }
    }
    None
}

unsafe extern "C" fn init_sample_interval(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(err) = require_int(&items, 0, "sample_interval_usec") {
        return err;
    }
    MbValue::none()
}

unsafe extern "C" fn collect_sequence(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(err) = require_sequence(&items, 0, "stack_frames") {
        return err;
    }
    MbValue::none()
}

unsafe extern "C" fn process_frames_sequence(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(err) = require_sequence(&items, 0, "frames") {
        return err;
    }
    MbValue::none()
}

unsafe extern "C" fn export_path(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(err) = require_path(&items, 0, "filename") {
        return err;
    }
    MbValue::none()
}

unsafe extern "C" fn set_stats(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(err) = require_int(&items, 0, "sample_interval_usec") {
        return err;
    }
    MbValue::none()
}

unsafe extern "C" fn print_stats(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(err) = require_int(&items, 0, "sort") {
        return err;
    }
    MbValue::none()
}

unsafe extern "C" fn collect_failed_sample(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn create_stats(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn normalize_location(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = flat_args(args_ptr, nargs);
    if let Some(value) = args.first() {
        if !is_location(*value) {
            return raise_type_error("location must be int, tuple, LocationInfo, or None");
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(0),
        MbValue::from_int(0),
        MbValue::from_int(0),
        MbValue::from_int(0),
    ]))
}

unsafe extern "C" fn extract_lineno(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = flat_args(args_ptr, nargs);
    if let Some(value) = args.first() {
        if !is_location(*value) {
            return raise_type_error("location must be int, tuple, LocationInfo, or None");
        }
    }
    MbValue::from_int(0)
}

unsafe extern "C" fn filter_internal_frames(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = flat_args(args_ptr, nargs);
    if let Some(err) = require_sequence(args, 0, "frames") {
        return err;
    }
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

unsafe extern "C" fn iter_async_frames(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = flat_args(args_ptr, nargs);
    if let Some(err) = require_sequence(args, 0, "awaited_info_list") {
        return err;
    }
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

fn register_flat_funcs(addrs: &[usize]) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for addr in addrs {
            set.insert(*addr as u64);
        }
    });
}

fn register_class(name: &str, methods: &[(&str, usize)]) {
    let mut map = HashMap::new();
    for (method, addr) in methods {
        super::super::module::register_variadic_func(*addr as u64);
        map.insert((*method).to_string(), MbValue::from_func(*addr));
    }
    super::super::class::mb_class_register(name, vec!["object".to_string()], map);
}

fn register_module_with(
    name: &str,
    classes: &[&str],
    funcs: &[(&str, usize)],
    consts: &[(&str, i64)],
) {
    let mut attrs = HashMap::new();
    for class in classes {
        attrs.insert((*class).to_string(), make_type_obj(class, name));
    }
    for (func, addr) in funcs {
        attrs.insert((*func).to_string(), MbValue::from_func(*addr));
    }
    for (key, value) in consts {
        attrs.insert((*key).to_string(), MbValue::from_int(*value));
    }
    register_flat_funcs(&funcs.iter().map(|(_, addr)| *addr).collect::<Vec<_>>());
    super::register_module(name, attrs);
}

pub fn register() {
    super::register_module("profiling", HashMap::new());

    register_module_with(
        "profiling.sampling.collector",
        &["Collector"],
        &[
            ("normalize_location", normalize_location as *const () as usize),
            ("extract_lineno", extract_lineno as *const () as usize),
            (
                "filter_internal_frames",
                filter_internal_frames as *const () as usize,
            ),
            ("iter_async_frames", iter_async_frames as *const () as usize),
        ],
        &[],
    );
    register_module_with(
        "profiling.sampling.gecko_collector",
        &["GeckoCollector"],
        &[],
        &[
            ("CATEGORY_OTHER", 0),
            ("CATEGORY_PYTHON", 1),
            ("GECKO_FORMAT_VERSION", 32),
        ],
    );
    register_module_with(
        "profiling.sampling.heatmap_collector",
        &["HeatmapCollector"],
        &[],
        &[],
    );
    register_module_with(
        "profiling.sampling.jsonl_collector",
        &["JsonlCollector"],
        &[],
        &[],
    );
    register_module_with(
        "profiling.sampling.pstats_collector",
        &["PstatsCollector"],
        &[],
        &[],
    );
    register_module_with(
        "profiling.sampling.stack_collector",
        &[
            "StackTraceCollector",
            "CollapsedStackCollector",
            "FlamegraphCollector",
            "DiffFlamegraphCollector",
        ],
        &[],
        &[],
    );

    let mut sampling_attrs = HashMap::new();
    for (class, module) in [
        ("Collector", "profiling.sampling.collector"),
        ("GeckoCollector", "profiling.sampling.gecko_collector"),
        ("HeatmapCollector", "profiling.sampling.heatmap_collector"),
        ("JsonlCollector", "profiling.sampling.jsonl_collector"),
        ("PstatsCollector", "profiling.sampling.pstats_collector"),
        ("StackTraceCollector", "profiling.sampling.stack_collector"),
        ("CollapsedStackCollector", "profiling.sampling.stack_collector"),
        ("FlamegraphCollector", "profiling.sampling.stack_collector"),
        ("DiffFlamegraphCollector", "profiling.sampling.stack_collector"),
    ] {
        sampling_attrs.insert(class.to_string(), make_type_obj(class, module));
    }
    super::register_module("profiling.sampling", sampling_attrs);

    register_class(
        "Collector",
        &[
            ("collect", collect_sequence as *const () as usize),
            ("collect_failed_sample", collect_failed_sample as *const () as usize),
            ("export", export_path as *const () as usize),
        ],
    );

    for class in [
        "GeckoCollector",
        "HeatmapCollector",
        "JsonlCollector",
        "PstatsCollector",
        "StackTraceCollector",
        "CollapsedStackCollector",
        "FlamegraphCollector",
        "DiffFlamegraphCollector",
    ] {
        register_class(
            class,
            &[
                ("__init__", init_sample_interval as *const () as usize),
                ("collect", collect_sequence as *const () as usize),
                ("export", export_path as *const () as usize),
                ("process_frames", process_frames_sequence as *const () as usize),
                ("set_stats", set_stats as *const () as usize),
                ("print_stats", print_stats as *const () as usize),
                ("create_stats", create_stats as *const () as usize),
            ],
        );
    }
}
