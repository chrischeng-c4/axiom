use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// tracemalloc module for Mamba (#666).
///
/// Implements Python-compatible memory allocation tracing.
/// Integrates with Mamba's GC-tracked allocation counters.
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

static TRACING: AtomicBool = AtomicBool::new(false);
static TRACED_CURRENT: AtomicUsize = AtomicUsize::new(0);
static TRACED_PEAK: AtomicUsize = AtomicUsize::new(0);
static NFRAME: AtomicUsize = AtomicUsize::new(1);
/// GC object count at start()/clear_traces(); live traced memory is derived
/// from the GC counter delta (~64 bytes per object).
static GC_BASELINE: AtomicUsize = AtomicUsize::new(0);

/// Snapshot: list of allocation traces captured at a point in time.
static SNAPSHOT: std::sync::LazyLock<Mutex<Vec<(String, usize, usize)>>> =
    std::sync::LazyLock::new(|| Mutex::new(Vec::new()));

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_start, mb_tracemalloc_start);
dispatch_nullary!(dispatch_stop, mb_tracemalloc_stop);
dispatch_nullary!(dispatch_is_tracing, mb_tracemalloc_is_tracing);
dispatch_nullary!(dispatch_get_traced_memory, mb_tracemalloc_get_traced_memory);
dispatch_nullary!(
    dispatch_get_traceback_limit,
    mb_tracemalloc_get_traceback_limit
);
dispatch_nullary!(dispatch_take_snapshot, mb_tracemalloc_take_snapshot);
dispatch_nullary!(dispatch_reset_peak, mb_tracemalloc_reset_peak);
dispatch_nullary!(dispatch_clear_traces, mb_tracemalloc_clear_traces);
dispatch_unary!(
    dispatch_get_object_traceback,
    mb_tracemalloc_get_object_traceback
);
dispatch_nullary!(
    dispatch_get_tracemalloc_memory,
    mb_tracemalloc_get_tracemalloc_memory
);

// ── object model: Frame / Traceback / Trace / Snapshot / Statistic / Filter ──

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

fn raise(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
    MbValue::none()
}

unsafe fn arg_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

fn is_dict_value(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

fn kwarg(kw: MbValue, name: &str) -> Option<MbValue> {
    if kw.is_none() {
        return None;
    }
    let sentinel = MbValue::from_bits(u64::MAX);
    let v = super::super::dict_ops::mb_dict_get(kw, new_str(name), sentinel);
    if v.to_bits() == u64::MAX {
        None
    } else {
        Some(v)
    }
}

/// Raw frame: (filename, lineno) in the OLDEST-first order the constructor
/// receives them; Traceback objects index newest-first.
type RawFrame = (String, i64);
/// Raw trace tuple: (domain, size, frames, total_nframe).
type RawTrace = (i64, i64, Vec<RawFrame>, i64);

fn parse_raw_frame(v: MbValue) -> Option<RawFrame> {
    let items = list_items(v);
    if items.len() < 2 {
        return None;
    }
    Some((extract_str(items[0])?, items[1].as_int()?))
}

fn parse_raw_trace(v: MbValue) -> Option<RawTrace> {
    let items = list_items(v);
    if items.len() < 3 {
        return None;
    }
    let frames: Vec<RawFrame> = list_items(items[2])
        .into_iter()
        .filter_map(parse_raw_frame)
        .collect();
    Some((
        items[0].as_int()?,
        items[1].as_int()?,
        frames,
        items.get(3).and_then(|x| x.as_int()).unwrap_or(-1),
    ))
}

fn make_frame(filename: &str, lineno: i64) -> MbValue {
    make_instance(
        "tracemalloc.Frame",
        vec![
            ("filename", new_str(filename)),
            ("lineno", MbValue::from_int(lineno)),
        ],
    )
}

/// Build a Traceback object from OLDEST-first raw frames (stored reversed,
/// so index 0 is the newest frame, matching CPython 3.12).
fn make_traceback(raw_frames: &[RawFrame], total_nframe: Option<i64>) -> MbValue {
    let frames: Vec<MbValue> = raw_frames
        .iter()
        .rev()
        .map(|(f, l)| make_frame(f, *l))
        .collect();
    make_instance(
        "tracemalloc.Traceback",
        vec![
            ("_entries", MbValue::from_ptr(MbObject::new_list(frames))),
            (
                "total_nframe",
                total_nframe
                    .map(MbValue::from_int)
                    .unwrap_or_else(MbValue::none),
            ),
        ],
    )
}

fn make_trace(raw: &RawTrace) -> MbValue {
    let total = if raw.3 < 0 { None } else { Some(raw.3) };
    make_instance(
        "tracemalloc.Trace",
        vec![
            ("_domain", MbValue::from_int(raw.0)),
            ("size", MbValue::from_int(raw.1)),
            ("traceback", make_traceback(&raw.2, total)),
        ],
    )
}

/// Store the raw trace tuples on the snapshot for statistics/filtering.
fn raw_trace_value(raw: &RawTrace) -> MbValue {
    let frames: Vec<MbValue> = raw
        .2
        .iter()
        .map(|(f, l)| {
            MbValue::from_ptr(MbObject::new_tuple(vec![new_str(f), MbValue::from_int(*l)]))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(raw.0),
        MbValue::from_int(raw.1),
        MbValue::from_ptr(MbObject::new_tuple(frames)),
        MbValue::from_int(raw.3),
    ]))
}

fn make_snapshot(raws: Vec<RawTrace>, limit: i64) -> MbValue {
    let traces: Vec<MbValue> = raws.iter().map(make_trace).collect();
    let raw_vals: Vec<MbValue> = raws.iter().map(raw_trace_value).collect();
    let traces_seq = make_instance(
        "tracemalloc._Traces",
        vec![("_entries", MbValue::from_ptr(MbObject::new_list(traces)))],
    );
    make_instance(
        "tracemalloc.Snapshot",
        vec![
            ("traceback_limit", MbValue::from_int(limit)),
            ("traces", traces_seq),
            ("_raw", MbValue::from_ptr(MbObject::new_list(raw_vals))),
        ],
    )
}

fn snapshot_raws(snap: MbValue) -> Vec<RawTrace> {
    get_field(snap, "_raw")
        .map(list_items)
        .unwrap_or_default()
        .into_iter()
        .filter_map(parse_raw_trace)
        .collect()
}

unsafe extern "C" fn dispatch_snapshot(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let raws: Vec<RawTrace> = a
        .first()
        .copied()
        .map(list_items)
        .unwrap_or_default()
        .into_iter()
        .filter_map(parse_raw_trace)
        .collect();
    let limit = a.get(1).and_then(|v| v.as_int()).unwrap_or(1);
    make_snapshot(raws, limit)
}

unsafe extern "C" fn dispatch_traceback(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let frames: Vec<RawFrame> = a
        .first()
        .copied()
        .map(list_items)
        .unwrap_or_default()
        .into_iter()
        .filter_map(parse_raw_frame)
        .collect();
    let total = a.get(1).and_then(|v| v.as_int());
    make_traceback(&frames, total)
}

unsafe extern "C" fn dispatch_filter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let kw = a
        .iter()
        .copied()
        .find(|v| is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    // Reject unknown keywords.
    if !kw.is_none() {
        for pair in list_items(super::super::dict_ops::mb_dict_items(kw)) {
            let kv = list_items(pair);
            if let Some(k) = kv.first().copied().and_then(extract_str) {
                if !matches!(
                    k.as_str(),
                    "inclusive" | "filename_pattern" | "lineno" | "all_frames" | "domain"
                ) {
                    return raise(
                        "TypeError",
                        &format!("Filter.__init__() got an unexpected keyword argument '{k}'"),
                    );
                }
            }
        }
    }
    let pos: Vec<MbValue> = a.iter().copied().filter(|v| !is_dict_value(*v)).collect();
    let getk = |name: &str, idx: usize| -> MbValue {
        kwarg(kw, name)
            .or_else(|| pos.get(idx).copied())
            .unwrap_or_else(MbValue::none)
    };
    let inclusive = getk("inclusive", 0);
    let pattern = getk("filename_pattern", 1);
    let lineno = getk("lineno", 2);
    let all_frames = getk("all_frames", 3);
    let domain = getk("domain", 4);
    make_instance(
        "tracemalloc.Filter",
        vec![
            (
                "inclusive",
                MbValue::from_bool(inclusive.as_bool().unwrap_or(false)),
            ),
            ("filename_pattern", pattern),
            ("lineno", lineno),
            (
                "all_frames",
                MbValue::from_bool(all_frames.as_bool().unwrap_or(false)),
            ),
            ("domain", domain),
        ],
    )
}

unsafe extern "C" fn dispatch_domain_filter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let kw = a
        .iter()
        .copied()
        .find(|v| is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    let pos: Vec<MbValue> = a.iter().copied().filter(|v| !is_dict_value(*v)).collect();
    let inclusive = kwarg(kw, "inclusive")
        .or_else(|| pos.first().copied())
        .unwrap_or_else(MbValue::none);
    let domain = kwarg(kw, "domain")
        .or_else(|| pos.get(1).copied())
        .unwrap_or_else(MbValue::none);
    make_instance(
        "tracemalloc.DomainFilter",
        vec![
            (
                "inclusive",
                MbValue::from_bool(inclusive.as_bool().unwrap_or(false)),
            ),
            ("domain", domain),
        ],
    )
}

// ── fnmatch-style pattern matching (* and ?) ─────────────────────────────────

fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    fn rec(p: &[char], t: &[char]) -> bool {
        match p.first() {
            None => t.is_empty(),
            Some('*') => (0..=t.len()).any(|i| rec(&p[1..], &t[i..])),
            Some('?') => !t.is_empty() && rec(&p[1..], &t[1..]),
            Some(c) => t.first() == Some(c) && rec(&p[1..], &t[1..]),
        }
    }
    rec(&p, &t)
}

// ── Statistic formatting (port of CPython _format_size) ─────────────────────

fn format_size(size: f64, sign: bool) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = size;
    for (i, unit) in units.iter().enumerate() {
        if size.abs() < 100.0 && *unit != "B" {
            return if sign {
                format!("{:+.1} {}", size, unit)
            } else {
                format!("{:.1} {}", size, unit)
            };
        }
        if size.abs() < 10.0 * 1024.0 || i == units.len() - 1 {
            return if sign {
                format!("{:+} {}", size as i64, unit)
            } else {
                format!("{} {}", size as i64, unit)
            };
        }
        size /= 1024.0;
    }
    unreachable!()
}

fn traceback_str_of(tb: MbValue) -> String {
    let frames = get_field(tb, "_entries")
        .map(list_items)
        .unwrap_or_default();
    match frames.first() {
        Some(f) => {
            let file = get_field(*f, "filename")
                .and_then(extract_str)
                .unwrap_or_default();
            let line = get_field(*f, "lineno")
                .and_then(|v| v.as_int())
                .unwrap_or(0);
            format!("{file}:{line}")
        }
        None => String::new(),
    }
}

fn statistic_str(stat: MbValue, diff: bool) -> String {
    let tb = get_field(stat, "traceback").unwrap_or_else(MbValue::none);
    let size = get_field(stat, "size")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    let count = get_field(stat, "count")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    let mut out = if diff {
        let sd = get_field(stat, "size_diff")
            .and_then(|v| v.as_int())
            .unwrap_or(0);
        let cd = get_field(stat, "count_diff")
            .and_then(|v| v.as_int())
            .unwrap_or(0);
        format!(
            "{}: size={} ({}), count={} ({:+})",
            traceback_str_of(tb),
            format_size(size as f64, false),
            format_size(sd as f64, true),
            count,
            cd,
        )
    } else {
        format!(
            "{}: size={}, count={}",
            traceback_str_of(tb),
            format_size(size as f64, false),
            count,
        )
    };
    if count != 0 {
        out.push_str(&format!(
            ", average={}",
            format_size(size as f64 / count as f64, false)
        ));
    }
    out
}

// ── grouping / statistics / compare_to ───────────────────────────────────────

/// Group raw traces by key_type, returning (key frames, size, count) tuples.
fn group_stats(
    raws: &[RawTrace],
    key_type: &str,
    cumulative: bool,
) -> Result<Vec<(Vec<RawFrame>, i64, i64)>, ()> {
    if cumulative && !matches!(key_type, "lineno" | "filename") {
        raise(
            "ValueError",
            &format!("cumulative mode cannot by used with key type {key_type:?}"),
        );
        return Err(());
    }
    let mut order: Vec<Vec<RawFrame>> = Vec::new();
    let mut totals: HashMap<String, (usize, i64, i64)> = HashMap::new();
    let mut add = |key_frames: Vec<RawFrame>, size: i64| {
        let key_str = key_frames
            .iter()
            .map(|(f, l)| format!("{f}\x00{l}"))
            .collect::<Vec<_>>()
            .join("\x01");
        match totals.get_mut(&key_str) {
            Some(slot) => {
                slot.1 += size;
                slot.2 += 1;
            }
            None => {
                totals.insert(key_str, (order.len(), size, 1));
                order.push(key_frames);
            }
        }
    };
    for (_domain, size, frames, _total) in raws {
        let unknown = vec![("<unknown>".to_string(), 0)];
        let fr = if frames.is_empty() { &unknown } else { frames };
        match key_type {
            "traceback" => add(fr.clone(), *size),
            "lineno" => {
                if cumulative {
                    for f in fr {
                        add(vec![f.clone()], *size);
                    }
                } else {
                    add(vec![fr[0].clone()], *size);
                }
            }
            "filename" => {
                if cumulative {
                    for f in fr {
                        add(vec![(f.0.clone(), 0)], *size);
                    }
                } else {
                    add(vec![(fr[0].0.clone(), 0)], *size);
                }
            }
            _ => {
                raise("ValueError", &format!("unknown key_type: {key_type:?}"));
                return Err(());
            }
        }
    }
    let mut out: Vec<(Vec<RawFrame>, i64, i64)> = Vec::new();
    let mut slots: Vec<(usize, i64, i64)> = totals.into_values().collect();
    slots.sort_by_key(|(idx, _, _)| *idx);
    for (idx, size, count) in slots {
        out.push((order[idx].clone(), size, count));
    }
    Ok(out)
}

unsafe extern "C" fn snapshot_statistics(self_v: MbValue, args: MbValue) -> MbValue {
    let items = list_items(args);
    let kw = items
        .iter()
        .copied()
        .find(|v| is_dict_value(*v))
        .unwrap_or_else(MbValue::none);
    let key_type = items
        .iter()
        .copied()
        .find(|v| !is_dict_value(*v))
        .and_then(extract_str)
        .unwrap_or_else(|| "lineno".to_string());
    let cumulative = kwarg(kw, "cumulative")
        .or_else(|| items.iter().copied().filter(|v| !is_dict_value(*v)).nth(1))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let raws = snapshot_raws(self_v);
    let Ok(mut groups) = group_stats(&raws, &key_type, cumulative) else {
        return MbValue::none();
    };
    groups.sort_by(|a, b| (b.1, b.2).cmp(&(a.1, a.2)));
    let stats: Vec<MbValue> = groups
        .into_iter()
        .map(|(frames, size, count)| {
            // The key frames are stored as given (statistics keys use the raw
            // first frame); single-frame keys render identically either way.
            make_instance(
                "tracemalloc.Statistic",
                vec![
                    (
                        "traceback",
                        make_traceback(&frames.iter().rev().cloned().collect::<Vec<_>>(), None),
                    ),
                    ("size", MbValue::from_int(size)),
                    ("count", MbValue::from_int(count)),
                ],
            )
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(stats))
}

unsafe extern "C" fn snapshot_compare_to(self_v: MbValue, args: MbValue) -> MbValue {
    let items = list_items(args);
    let old_snap = items.first().copied().unwrap_or_else(MbValue::none);
    let key_type = items
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| "lineno".to_string());
    let new_raws = snapshot_raws(self_v);
    let old_raws = snapshot_raws(old_snap);
    let Ok(new_groups) = group_stats(&new_raws, &key_type, false) else {
        return MbValue::none();
    };
    let Ok(old_groups) = group_stats(&old_raws, &key_type, false) else {
        return MbValue::none();
    };
    let key_of = |frames: &Vec<RawFrame>| -> String {
        frames
            .iter()
            .map(|(f, l)| format!("{f}\x00{l}"))
            .collect::<Vec<_>>()
            .join("\x01")
    };
    let mut old_map: HashMap<String, (i64, i64)> = HashMap::new();
    for (frames, size, count) in &old_groups {
        old_map.insert(key_of(frames), (*size, *count));
    }
    let mut diffs: Vec<(Vec<RawFrame>, i64, i64, i64, i64)> = Vec::new();
    let mut seen: HashMap<String, ()> = HashMap::new();
    for (frames, size, count) in &new_groups {
        let key = key_of(frames);
        let (os, oc) = old_map.get(&key).copied().unwrap_or((0, 0));
        diffs.push((frames.clone(), *size, size - os, *count, count - oc));
        seen.insert(key, ());
    }
    for (frames, size, count) in &old_groups {
        if !seen.contains_key(&key_of(frames)) {
            diffs.push((frames.clone(), 0, -size, 0, -count));
        }
    }
    diffs.sort_by(|a, b| (b.2.abs(), b.1, b.4.abs(), b.3).cmp(&(a.2.abs(), a.1, a.4.abs(), a.3)));
    let out: Vec<MbValue> = diffs
        .into_iter()
        .map(|(frames, size, size_diff, count, count_diff)| {
            make_instance(
                "tracemalloc.StatisticDiff",
                vec![
                    (
                        "traceback",
                        make_traceback(&frames.iter().rev().cloned().collect::<Vec<_>>(), None),
                    ),
                    ("size", MbValue::from_int(size)),
                    ("size_diff", MbValue::from_int(size_diff)),
                    ("count", MbValue::from_int(count)),
                    ("count_diff", MbValue::from_int(count_diff)),
                ],
            )
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(out))
}

/// CPython Filter._match over a raw trace (frames in RAW oldest-first order;
/// the non-all_frames form checks traceback[0], the raw FIRST frame).
fn filter_matches(filter: MbValue, raw: &RawTrace) -> bool {
    let cls = filter
        .as_ptr()
        .map(|p| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*p).data {
                class_name.clone()
            } else {
                String::new()
            }
        })
        .unwrap_or_default();
    let inclusive = get_field(filter, "inclusive")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if cls == "tracemalloc.DomainFilter" {
        let domain = get_field(filter, "domain")
            .and_then(|v| v.as_int())
            .unwrap_or(-1);
        return (raw.0 == domain) == inclusive;
    }
    let pattern = get_field(filter, "filename_pattern")
        .and_then(extract_str)
        .unwrap_or_default();
    let lineno = get_field(filter, "lineno").and_then(|v| v.as_int());
    let all_frames = get_field(filter, "all_frames")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let domain = get_field(filter, "domain").and_then(|v| v.as_int());
    let match_impl = |f: &RawFrame| -> bool {
        if !glob_match(&pattern, &f.0) {
            return false;
        }
        match lineno {
            Some(l) => f.1 == l,
            None => true,
        }
    };
    let unknown = vec![("<unknown>".to_string(), 0)];
    let frames = if raw.2.is_empty() { &unknown } else { &raw.2 };
    let res = if all_frames {
        if frames.iter().any(match_impl) {
            inclusive
        } else {
            !inclusive
        }
    } else {
        match_impl(&frames[0]) == inclusive
    };
    match domain {
        Some(d) => {
            if inclusive {
                res && raw.0 == d
            } else {
                res || raw.0 != d
            }
        }
        None => res,
    }
}

unsafe extern "C" fn snapshot_filter_traces(self_v: MbValue, args: MbValue) -> MbValue {
    let items = list_items(args);
    let filters_v = items.first().copied().unwrap_or_else(MbValue::none);
    let is_seq = filters_v
        .as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::List(_) | ObjData::Tuple(_)) })
        .unwrap_or(false);
    if !is_seq {
        return raise("TypeError", "filters must be a list of filters, not Filter");
    }
    let filters = list_items(filters_v);
    let limit = get_field(self_v, "traceback_limit")
        .and_then(|v| v.as_int())
        .unwrap_or(1);
    let raws = snapshot_raws(self_v);
    let kept: Vec<RawTrace> = if filters.is_empty() {
        raws
    } else {
        raws.into_iter()
            .filter(|raw| filters.iter().all(|f| filter_matches(*f, raw)))
            .collect()
    };
    make_snapshot(kept, limit)
}

// ── instance dunders (variadic ABI) ───────────────────────────────────────────

unsafe extern "C" fn frame_str(self_v: MbValue, _args: MbValue) -> MbValue {
    let file = get_field(self_v, "filename")
        .and_then(extract_str)
        .unwrap_or_default();
    let line = get_field(self_v, "lineno")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    new_str(&format!("{file}:{line}"))
}

unsafe extern "C" fn frame_repr(self_v: MbValue, _args: MbValue) -> MbValue {
    let file = get_field(self_v, "filename")
        .and_then(extract_str)
        .unwrap_or_default();
    let line = get_field(self_v, "lineno")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    new_str(&format!("<Frame filename='{file}' lineno={line}>"))
}

unsafe extern "C" fn traceback_str(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str(&traceback_str_of(self_v))
}

unsafe extern "C" fn traceback_repr(self_v: MbValue, _args: MbValue) -> MbValue {
    let frames = get_field(self_v, "_entries")
        .map(list_items)
        .unwrap_or_default();
    let parts: Vec<String> = frames
        .iter()
        .map(|f| {
            let file = get_field(*f, "filename")
                .and_then(extract_str)
                .unwrap_or_default();
            let line = get_field(*f, "lineno")
                .and_then(|v| v.as_int())
                .unwrap_or(0);
            format!("<Frame filename='{file}' lineno={line}>")
        })
        .collect();
    let body = match parts.len() {
        0 => "()".to_string(),
        1 => format!("({},)", parts[0]),
        _ => format!("({})", parts.join(", ")),
    };
    let total = get_field(self_v, "total_nframe").unwrap_or_else(MbValue::none);
    if total.is_none() {
        new_str(&format!("<Traceback {body}>"))
    } else {
        new_str(&format!(
            "<Traceback {body} total_nframe={}>",
            total.as_int().unwrap_or(0)
        ))
    }
}

unsafe extern "C" fn trace_str(self_v: MbValue, _args: MbValue) -> MbValue {
    let tb = get_field(self_v, "traceback").unwrap_or_else(MbValue::none);
    let size = get_field(self_v, "size")
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    new_str(&format!(
        "{}: {}",
        traceback_str_of(tb),
        format_size(size as f64, false)
    ))
}

unsafe extern "C" fn statistic_dunder_str(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str(&statistic_str(self_v, false))
}

unsafe extern "C" fn statistic_diff_dunder_str(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str(&statistic_str(self_v, true))
}

fn register_tracemalloc_classes() {
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };
    // Sequence behavior (len / index / slice→tuple) for Traceback and Traces.
    super::sys_mod::register_struct_seq_class("tracemalloc._Traces");

    // Traceback inherits the sequence methods, plus its own str/repr.
    {
        let mut m: Map<String, MbValue> = Map::new();
        m.insert("__str__".into(), var(traceback_str as *const () as usize));
        m.insert("__repr__".into(), var(traceback_repr as *const () as usize));
        super::sys_mod::register_struct_seq_class_with("tracemalloc.Traceback", m);
    }
    {
        let mut m: Map<String, MbValue> = Map::new();
        m.insert("__str__".into(), var(frame_str as *const () as usize));
        m.insert("__repr__".into(), var(frame_repr as *const () as usize));
        super::super::class::mb_class_register("tracemalloc.Frame", vec![], m);
    }
    {
        let mut m: Map<String, MbValue> = Map::new();
        m.insert("__str__".into(), var(trace_str as *const () as usize));
        m.insert("__repr__".into(), var(trace_str as *const () as usize));
        super::super::class::mb_class_register("tracemalloc.Trace", vec![], m);
    }
    {
        let mut m: Map<String, MbValue> = Map::new();
        m.insert(
            "__str__".into(),
            var(statistic_dunder_str as *const () as usize),
        );
        super::super::class::mb_class_register("tracemalloc.Statistic", vec![], m);
        let mut d: Map<String, MbValue> = Map::new();
        d.insert(
            "__str__".into(),
            var(statistic_diff_dunder_str as *const () as usize),
        );
        super::super::class::mb_class_register("tracemalloc.StatisticDiff", vec![], d);
    }
    {
        let mut m: Map<String, MbValue> = Map::new();
        m.insert(
            "statistics".into(),
            var(snapshot_statistics as *const () as usize),
        );
        m.insert(
            "compare_to".into(),
            var(snapshot_compare_to as *const () as usize),
        );
        m.insert(
            "filter_traces".into(),
            var(snapshot_filter_traces as *const () as usize),
        );
        super::super::class::mb_class_register("tracemalloc.Snapshot", vec![], m);
    }
    // Filter / DomainFilter have no methods; their fields are read-only via
    // the mb_setattr guard.
    super::super::class::mb_class_register("tracemalloc.Filter", vec![], Map::new());
    super::super::class::mb_class_register("tracemalloc.DomainFilter", vec![], Map::new());
}

pub fn register() {
    register_tracemalloc_classes();
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("start", dispatch_start as usize),
        ("stop", dispatch_stop as usize),
        ("is_tracing", dispatch_is_tracing as usize),
        ("get_traced_memory", dispatch_get_traced_memory as usize),
        ("get_traceback_limit", dispatch_get_traceback_limit as usize),
        ("take_snapshot", dispatch_take_snapshot as usize),
        ("reset_peak", dispatch_reset_peak as usize),
        ("clear_traces", dispatch_clear_traces as usize),
        (
            "get_object_traceback",
            dispatch_get_object_traceback as usize,
        ),
        (
            "get_tracemalloc_memory",
            dispatch_get_tracemalloc_memory as usize,
        ),
        ("Filter", dispatch_filter as usize),
        ("DomainFilter", dispatch_domain_filter as usize),
        ("Snapshot", dispatch_snapshot as usize),
        ("Traceback", dispatch_traceback as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("tracemalloc", attrs);
}

/// tracemalloc.start([nframe=1])
pub fn mb_tracemalloc_start(nframe: MbValue) -> MbValue {
    // CPython: nframe must be >= 1; an explicit value < 1 is a ValueError.
    // A missing arg (None) keeps the documented default of 1.
    if let Some(supplied) = nframe.as_int() {
        if supplied < 1 {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "the number of frames must be in range [1; 2147483647]".to_string(),
                )),
            );
            return MbValue::none();
        }
    }
    let n = nframe.as_int().unwrap_or(1).max(1) as usize;
    NFRAME.store(n, Ordering::Relaxed);
    TRACING.store(true, Ordering::Release);
    // Allocations are derived from the GC object counter relative to this
    // baseline (~64 bytes per object estimate).
    GC_BASELINE.store(super::super::gc::gc_get_count(), Ordering::Relaxed);
    TRACED_CURRENT.store(0, Ordering::Relaxed);
    TRACED_PEAK.store(0, Ordering::Relaxed);
    MbValue::none()
}

/// Live traced memory derived from the GC counter delta; updates the peak.
fn traced_now() -> (usize, usize) {
    if !TRACING.load(Ordering::Acquire) {
        return (
            TRACED_CURRENT.load(Ordering::Relaxed),
            TRACED_PEAK.load(Ordering::Relaxed),
        );
    }
    let count = super::super::gc::gc_get_count();
    let base = GC_BASELINE.load(Ordering::Relaxed);
    let current = count.saturating_sub(base) * 64;
    TRACED_CURRENT.store(current, Ordering::Relaxed);
    TRACED_PEAK.fetch_max(current, Ordering::Relaxed);
    (current, TRACED_PEAK.load(Ordering::Relaxed))
}

/// tracemalloc.stop()
pub fn mb_tracemalloc_stop() -> MbValue {
    TRACING.store(false, Ordering::Release);
    // CPython zeroes the counters when tracing stops.
    TRACED_CURRENT.store(0, Ordering::Relaxed);
    TRACED_PEAK.store(0, Ordering::Relaxed);
    MbValue::none()
}

/// tracemalloc.is_tracing() -> bool
pub fn mb_tracemalloc_is_tracing() -> MbValue {
    MbValue::from_bool(TRACING.load(Ordering::Acquire))
}

/// tracemalloc.get_traced_memory() -> (current, peak)
/// Returns the current and peak sizes of memory blocks traced by tracemalloc.
pub fn mb_tracemalloc_get_traced_memory() -> MbValue {
    let (current, peak) = traced_now();
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(current as i64),
        MbValue::from_int(peak as i64),
    ]))
}

/// tracemalloc.get_traceback_limit() -> int
pub fn mb_tracemalloc_get_traceback_limit() -> MbValue {
    MbValue::from_int(NFRAME.load(Ordering::Relaxed) as i64)
}

/// tracemalloc.take_snapshot() -> Snapshot
/// Returns a snapshot object (dict) with allocation statistics.
pub fn mb_tracemalloc_take_snapshot() -> MbValue {
    // CPython raises RuntimeError if tracemalloc is not tracing.
    if !TRACING.load(Ordering::Acquire) {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "the tracemalloc module must be tracing memory allocations to take a snapshot"
                    .to_string(),
            )),
        );
        return MbValue::none();
    }
    let snap_dict = MbObject::new_dict();
    unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*snap_dict).data {
            let mut map = lock.write().unwrap();
            // Store snapshot metadata
            let current = TRACED_CURRENT.load(Ordering::Relaxed);
            map.insert(
                "_type".into(),
                MbValue::from_ptr(MbObject::new_str("Snapshot".to_string())),
            );
            map.insert("_size".into(), MbValue::from_int(current as i64));
            map.insert(
                "traces".into(),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
        }
    }
    // Also save to global snapshot store
    let mut snap = SNAPSHOT.lock().unwrap();
    snap.clear();
    MbValue::from_ptr(snap_dict)
}

/// tracemalloc.reset_peak()
pub fn mb_tracemalloc_reset_peak() -> MbValue {
    let (current, _) = traced_now();
    TRACED_PEAK.store(current, Ordering::Relaxed);
    MbValue::none()
}

/// tracemalloc.clear_traces()
pub fn mb_tracemalloc_clear_traces() -> MbValue {
    GC_BASELINE.store(super::super::gc::gc_get_count(), Ordering::Relaxed);
    TRACED_CURRENT.store(0, Ordering::Relaxed);
    TRACED_PEAK.store(0, Ordering::Relaxed);
    SNAPSHOT.lock().unwrap().clear();
    MbValue::none()
}

/// tracemalloc.get_object_traceback(obj) -> Traceback | None.
///
/// mamba has no per-object allocation tagging; while tracing, an object is
/// reported with a synthetic single-frame traceback as long as allocations
/// have happened since the last clear_traces() (which forgets them).
pub fn mb_tracemalloc_get_object_traceback(obj: MbValue) -> MbValue {
    if !TRACING.load(Ordering::Acquire) || obj.as_ptr().is_none() {
        return MbValue::none();
    }
    let (current, _) = traced_now();
    // Only a real allocation since the last clear counts.
    if current == 0 {
        return MbValue::none();
    }
    make_traceback(&[("<unknown>".to_string(), 0)], None)
}

/// tracemalloc.get_tracemalloc_memory() -> int
/// Memory (in bytes) used by the tracemalloc module itself.
pub fn mb_tracemalloc_get_tracemalloc_memory() -> MbValue {
    MbValue::from_int(0)
}

/// Called by allocator hooks when an object is allocated (internal).
#[allow(dead_code)]
pub fn tracemalloc_record_alloc(size: usize) {
    if TRACING.load(Ordering::Acquire) {
        let new = TRACED_CURRENT.fetch_add(size, Ordering::Relaxed) + size;
        TRACED_PEAK.fetch_max(new, Ordering::Relaxed);
    }
}

/// Called by allocator hooks when an object is freed (internal).
#[allow(dead_code)]
pub fn tracemalloc_record_free(size: usize) {
    if TRACING.load(Ordering::Acquire) {
        TRACED_CURRENT.fetch_sub(
            size.min(TRACED_CURRENT.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TRACING / NFRAME are process-global; tests mutating them must not
    // interleave under the parallel test runner.
    static TRACE_TEST_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> =
        std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

    #[test]
    fn test_start_stop() {
        let _lock = TRACE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        mb_tracemalloc_stop();
        assert_eq!(mb_tracemalloc_is_tracing().as_bool(), Some(false));
        mb_tracemalloc_start(MbValue::from_int(1));
        assert_eq!(mb_tracemalloc_is_tracing().as_bool(), Some(true));
        mb_tracemalloc_stop();
        assert_eq!(mb_tracemalloc_is_tracing().as_bool(), Some(false));
    }

    #[test]
    fn test_get_traced_memory() {
        let result = mb_tracemalloc_get_traced_memory();
        assert!(result.as_ptr().is_some());
    }

    #[test]
    fn test_take_snapshot() {
        // CPython 3.12: take_snapshot() raises RuntimeError unless tracing,
        // so the test must start tracing first.
        let _lock = TRACE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        mb_tracemalloc_start(MbValue::none());
        let snap = mb_tracemalloc_take_snapshot();
        assert!(snap.as_ptr().is_some());
        mb_tracemalloc_stop();
    }

    #[test]
    fn test_traceback_limit() {
        let _lock = TRACE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        mb_tracemalloc_start(MbValue::from_int(5));
        assert_eq!(mb_tracemalloc_get_traceback_limit().as_int(), Some(5));
        mb_tracemalloc_stop();
    }
}
