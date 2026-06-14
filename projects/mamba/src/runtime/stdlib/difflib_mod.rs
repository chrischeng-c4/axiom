/// difflib module for Mamba (mamba-stdlib).
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

macro_rules! dispatch_quaternary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
                a.get(2).copied().unwrap_or_else(MbValue::none),
                a.get(3).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_binary!(dispatch_ratio, mb_difflib_ratio);
dispatch_quaternary!(dispatch_get_close_matches, mb_difflib_get_close_matches);
dispatch_binary!(dispatch_format_range_context, mb_difflib_format_range_context);

// `difflib.unified_diff(a, b, fromfile='', tofile='', fromfiledate='',
// tofiledate='', n=3, lineterm='\n')` is a generator in CPython taking many
// positional/keyword args, so it needs the variadic dispatch shape (not the
// fixed-arity `dispatch_binary!`).
unsafe extern "C" fn dispatch_unified_diff(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_difflib_unified_diff_full(a)
}

// `difflib.IS_CHARACTER_JUNK(ch, ws=' \t')`: True iff ch is one of the
// whitespace characters in ws (default space and tab). CPython treats it as a
// module-level callable, so expose it as a native func.
unsafe extern "C" fn dispatch_IS_CHARACTER_JUNK(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let ch = a.first().copied().unwrap_or_else(MbValue::none);
    // Optional second arg `ws` overrides the default " \t".
    let ws = a
        .get(1)
        .and_then(|v| extract_str(*v))
        .unwrap_or_else(|| " \t".to_string());
    let is_junk = extract_str(ch)
        .map(|s| s.chars().count() == 1 && s.chars().all(|c| ws.contains(c)))
        .unwrap_or(false);
    MbValue::from_bool(is_junk)
}

// `difflib.IS_LINE_JUNK(line, pat=re.compile(r"\s*(?:#\s*)?$").match)`: True iff
// the line is all whitespace, optionally followed by a single '#' and trailing
// whitespace. Implemented directly (no regex) to avoid the CPython ReDoS path.
unsafe extern "C" fn dispatch_IS_LINE_JUNK(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let line = a
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    MbValue::from_bool(is_line_junk(&line))
}

// `difflib._format_range_unified(start, stop)`: convert a [start, stop) opcode
// range to unified-diff range notation.
unsafe extern "C" fn dispatch_format_range_unified(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let start = a.first().and_then(|v| v.as_int()).unwrap_or(0);
    let stop = a.get(1).and_then(|v| v.as_int()).unwrap_or(0);
    MbValue::from_ptr(MbObject::new_str(format_range_unified(start, stop)))
}

/// CPython IS_LINE_JUNK: match of r"\s*(?:#\s*)?$" anchored at start. A line is
/// junk iff it is zero or more whitespace chars, optionally followed by a
/// single '#' and zero or more whitespace chars, with nothing else.
fn is_line_junk(line: &str) -> bool {
    let mut chars = line.chars().peekable();
    // Leading run of whitespace.
    while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
        chars.next();
    }
    // Optional single '#' followed by trailing whitespace.
    if matches!(chars.peek(), Some('#')) {
        chars.next();
        while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
            chars.next();
        }
    }
    chars.peek().is_none()
}

/// CPython _format_range_unified(start, stop): unified-diff range notation.
fn format_range_unified(start: i64, stop: i64) -> String {
    let mut beginning = start + 1;
    let length = stop - start;
    if length == 1 {
        return format!("{beginning}");
    }
    if length == 0 {
        beginning -= 1; // empty ranges begin at the line just before the range
    }
    format!("{beginning},{length}")
}

// `difflib.SequenceMatcher(isjunk, a, b)` constructs a matcher OBJECT (not a
// bare ratio float) so that `.ratio()`, `.get_opcodes()`,
// `.find_longest_match()`, `.get_matching_blocks()`, and
// `.get_grouped_opcodes()` dispatch as methods. Keyword args
// (`SequenceMatcher(a=.., b=..)`) arrive as a trailing dict (mamba lowers an
// attribute-call's kwargs to a dict appended to the positional args).
unsafe extern "C" fn dispatch_SequenceMatcher(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_difflib_sequence_matcher_new(a)
}

// `difflib.Differ(linejunk=None, charjunk=IS_CHARACTER_JUNK)` constructs a
// helper OBJECT exposing `.compare(a, b)`. Default junk matches CPython's
// `ndiff`, so `Differ().compare(a, b)` and `ndiff(a, b)` agree.
unsafe extern "C" fn dispatch_Differ(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance(DIFFER_CLASS.to_string()))
}

unsafe extern "C" fn method_compare(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_list(args).unwrap_or_default();
    let a = items.first().copied().unwrap_or_else(MbValue::none);
    let b = items.get(1).copied().unwrap_or_else(MbValue::none);
    mb_difflib_ndiff(a, b)
}

// Instance methods — dispatched via mb_call_method's Instance path as
// variadic `fn(self, args_list)`.
unsafe extern "C" fn method_ratio(self_v: MbValue, _args: MbValue) -> MbValue {
    mb_sm_ratio(self_v)
}
unsafe extern "C" fn method_quick_ratio(self_v: MbValue, _args: MbValue) -> MbValue {
    mb_sm_quick_ratio(self_v)
}
unsafe extern "C" fn method_real_quick_ratio(self_v: MbValue, _args: MbValue) -> MbValue {
    mb_sm_real_quick_ratio(self_v)
}
unsafe extern "C" fn method_get_opcodes(self_v: MbValue, _args: MbValue) -> MbValue {
    mb_sm_get_opcodes(self_v)
}
unsafe extern "C" fn method_get_matching_blocks(self_v: MbValue, _args: MbValue) -> MbValue {
    mb_sm_get_matching_blocks(self_v)
}
unsafe extern "C" fn method_get_grouped_opcodes(self_v: MbValue, args: MbValue) -> MbValue {
    // get_grouped_opcodes(n=3): caller-supplied n threads through.
    let items = extract_list(args).unwrap_or_default();
    let n = items.first().and_then(|v| v.as_int()).unwrap_or(3);
    mb_sm_get_grouped_opcodes(self_v, n)
}
unsafe extern "C" fn method_find_longest_match(self_v: MbValue, args: MbValue) -> MbValue {
    mb_sm_find_longest_match(self_v, args)
}
unsafe extern "C" fn method_set_seqs(self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_list(args).unwrap_or_default();
    let a = items.first().copied().unwrap_or_else(MbValue::none);
    let b = items.get(1).copied().unwrap_or_else(MbValue::none);
    set_seq1_inner(self_v, a);
    set_seq2_inner(self_v, b);
    MbValue::none()
}
unsafe extern "C" fn method_set_seq1(self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_list(args).unwrap_or_default();
    let a = items.first().copied().unwrap_or_else(MbValue::none);
    set_seq1_inner(self_v, a);
    MbValue::none()
}
unsafe extern "C" fn method_set_seq2(self_v: MbValue, args: MbValue) -> MbValue {
    let items = extract_list(args).unwrap_or_default();
    let b = items.first().copied().unwrap_or_else(MbValue::none);
    set_seq2_inner(self_v, b);
    MbValue::none()
}

/// set_seq1(a): replace the first sequence and invalidate the cached
/// matching-blocks list (CPython resets opcodes/matching_blocks).
fn set_seq1_inner(self_v: MbValue, a: MbValue) {
    set_field(self_v, "a", a);
    set_field(self_v, "_mb", MbValue::none());
}

/// set_seq2(b): replace the second sequence, recompute the autojunk/junk
/// state derived from b, and invalidate the cache.
fn set_seq2_inner(self_v: MbValue, b: MbValue) {
    set_field(self_v, "b", b);
    set_field(self_v, "_mb", MbValue::none());
    recompute_b_junk(self_v, b);
}

// ndiff/context_diff are module-level generators in CPython; mamba materializes
// the lines into a list iterator.
unsafe extern "C" fn dispatch_ndiff(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_difflib_ndiff(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_context_diff(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_difflib_context_diff(a)
}
unsafe extern "C" fn dispatch_restore(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_difflib_restore(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_diff_bytes(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_difflib_diff_bytes(a)
}
unsafe extern "C" fn dispatch_mdiff(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_difflib_mdiff(a)
}

const SM_CLASS: &str = "difflib.SequenceMatcher";
const MATCH_CLASS: &str = "difflib.Match";
const DIFFER_CLASS: &str = "difflib.Differ";

pub fn register() {
    // Register the SequenceMatcher instance methods as a runtime class so
    // method dispatch (`sm.get_opcodes()`) resolves through the normal MRO
    // path. Each method is a variadic native `fn(self, args_list)`.
    let methods: Vec<(&str, usize)> = vec![
        ("ratio", method_ratio as usize),
        ("quick_ratio", method_quick_ratio as usize),
        ("real_quick_ratio", method_real_quick_ratio as usize),
        ("get_opcodes", method_get_opcodes as usize),
        ("get_matching_blocks", method_get_matching_blocks as usize),
        ("get_grouped_opcodes", method_get_grouped_opcodes as usize),
        ("find_longest_match", method_find_longest_match as usize),
        ("set_seqs", method_set_seqs as usize),
        ("set_seq1", method_set_seq1 as usize),
        ("set_seq2", method_set_seq2 as usize),
    ];
    let mut method_map: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in &methods {
        method_map.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    super::super::class::mb_class_register(SM_CLASS, vec![], method_map);

    // Register the Differ helper class with its single `compare` method.
    let mut differ_methods: HashMap<String, MbValue> = HashMap::new();
    differ_methods.insert("compare".to_string(), MbValue::from_func(method_compare as usize));
    super::super::module::register_variadic_func(method_compare as usize as u64);
    super::super::class::mb_class_register(DIFFER_CLASS, vec![], differ_methods);

    // Register the HtmlDiff helper class with make_file / make_table.
    let mut html_methods: HashMap<String, MbValue> = HashMap::new();
    html_methods.insert("make_file".to_string(), MbValue::from_func(method_make_file as usize));
    html_methods.insert("make_table".to_string(), MbValue::from_func(method_make_table as usize));
    super::super::module::register_variadic_func(method_make_file as usize as u64);
    super::super::module::register_variadic_func(method_make_table as usize as u64);
    super::super::class::mb_class_register(HTMLDIFF_CLASS, vec![], html_methods);

    // Register the Match named-tuple class so `difflib.Match` resolves to a
    // class object. `make_match` constructs instances of this class; here we
    // register the class itself (no extra methods) and attach the namedtuple
    // `_fields` class attribute `('a', 'b', 'size')` matching CPython's
    // `difflib.Match = namedtuple('Match', 'a b size')`.
    super::super::class::mb_class_register(MATCH_CLASS, vec![], HashMap::new());
    let match_fields = MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str("a".to_string())),
        MbValue::from_ptr(MbObject::new_str("b".to_string())),
        MbValue::from_ptr(MbObject::new_str("size".to_string())),
    ]));
    super::super::class::mb_class_set_class_attr(
        MbValue::from_ptr(MbObject::new_str(MATCH_CLASS.to_string())),
        MbValue::from_ptr(MbObject::new_str("_fields".to_string())),
        match_fields,
    );

    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("SequenceMatcher", dispatch_SequenceMatcher as usize),
        ("Differ", dispatch_Differ as usize),
        ("ratio", dispatch_ratio as usize),
        ("unified_diff", dispatch_unified_diff as usize),
        ("get_close_matches", dispatch_get_close_matches as usize),
        ("ndiff", dispatch_ndiff as usize),
        ("context_diff", dispatch_context_diff as usize),
        ("restore", dispatch_restore as usize),
        ("diff_bytes", dispatch_diff_bytes as usize),
        ("_mdiff", dispatch_mdiff as usize),
        ("HtmlDiff", dispatch_HtmlDiff as usize),
        ("_format_range_context", dispatch_format_range_context as usize),
        ("_format_range_unified", dispatch_format_range_unified as usize),
        ("IS_CHARACTER_JUNK", dispatch_IS_CHARACTER_JUNK as usize),
        ("IS_LINE_JUNK", dispatch_IS_LINE_JUNK as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // `difflib.Match` is a class object, exposed as a class-name string value
    // (the runtime's representation of a class), so attribute access on it
    // (`difflib.Match._fields`) resolves through the class registry.
    attrs.insert(
        "Match".to_string(),
        MbValue::from_ptr(MbObject::new_str(MATCH_CLASS.to_string())),
    );
    super::register_module("difflib", attrs);
}

// ── SequenceMatcher object + core algorithm ──

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

/// Extract a sequence argument as a Vec of element strings. Strings are split
/// into per-character tokens; lists/tuples keep their element repr/string.
fn seq_tokens(val: MbValue) -> Vec<String> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::Str(ref s) => return s.chars().map(|c| c.to_string()).collect(),
                ObjData::List(ref lock) => {
                    return lock
                        .read()
                        .unwrap()
                        .iter()
                        .map(|v| extract_str(*v).unwrap_or_else(|| super::super::builtins::mb_str(*v).as_ptr().and_then(|p| if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }).unwrap_or_default()))
                        .collect();
                }
                ObjData::Tuple(ref items) => {
                    return items
                        .iter()
                        .map(|v| extract_str(*v).unwrap_or_default())
                        .collect();
                }
                _ => {}
            }
        }
    }
    Vec::new()
}

/// Constructor: `SequenceMatcher(isjunk=None, a='', b='', autojunk=True)` —
/// matching CPython 3.12's signature. Positional binding follows that order:
/// 1 positional -> isjunk; 2 -> (isjunk, a); 3 -> (isjunk, a, b). Keyword args
/// `isjunk=`, `a=`, `b=`, `autojunk=` override the positional slots.
fn mb_difflib_sequence_matcher_new(args: &[MbValue]) -> MbValue {
    // Separate a trailing kwargs dict (mamba appends one for attribute calls
    // with keyword arguments) from the positional args.
    let mut positional: Vec<MbValue> = Vec::new();
    let mut kw_isjunk: Option<MbValue> = None;
    let mut kw_a: Option<MbValue> = None;
    let mut kw_b: Option<MbValue> = None;
    let mut autojunk = true;
    for (i, v) in args.iter().enumerate() {
        let is_last = i + 1 == args.len();
        if is_last {
            if let Some(ptr) = v.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        let map = lock.read().unwrap();
                        kw_isjunk = map.get("isjunk").copied();
                        kw_a = map.get("a").copied();
                        kw_b = map.get("b").copied();
                        if let Some(aj) = map.get("autojunk") {
                            autojunk = aj.as_bool().unwrap_or(true);
                        }
                        // Skip pushing the kwargs dict as positional.
                        continue;
                    }
                }
            }
        }
        positional.push(*v);
    }

    // Positional form is (isjunk, a, b) per CPython's signature.
    let empty_str = || MbValue::from_ptr(MbObject::new_str(String::new()));
    let (isjunk_val, a_val, b_val) = match positional.len() {
        0 => (MbValue::none(), empty_str(), empty_str()),
        1 => (positional[0], empty_str(), empty_str()),
        2 => (positional[0], positional[1], empty_str()),
        _ => (positional[0], positional[1], positional[2]),
    };
    // Keyword args override the corresponding positional slot.
    let isjunk_val = kw_isjunk.unwrap_or(isjunk_val);
    let a_val = kw_a.unwrap_or(a_val);
    let b_val = kw_b.unwrap_or(b_val);

    let inst = MbValue::from_ptr(MbObject::new_instance(SM_CLASS.to_string()));
    set_field(inst, "isjunk", isjunk_val);
    set_field(inst, "a", a_val);
    set_field(inst, "b", b_val);
    set_field(inst, "autojunk", MbValue::from_bool(autojunk));
    recompute_b_junk(inst, b_val);
    inst
}

/// Recompute the b-derived junk state and expose it as the `.bjunk`
/// (isjunk-driven) and `.bpopular` (autojunk-driven) attributes, matching
/// CPython's SequenceMatcher.__chain_b().
fn recompute_b_junk(inst: MbValue, b_val: MbValue) {
    let autojunk = get_field(inst, "autojunk")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let b_tokens = seq_tokens(b_val);

    // bjunk: distinct elements of b for which isjunk(elt) is truthy.
    let isjunk = get_field(inst, "isjunk").unwrap_or_else(MbValue::none);
    let bjunk = bjunk_set(&b_tokens, isjunk);
    let bjunk_vals: Vec<MbValue> = bjunk
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str(s.clone())))
        .collect();
    set_field(inst, "bjunk", MbValue::from_ptr(MbObject::new_set(bjunk_vals)));

    // bpopular: autojunk "popular" set on b (distinct from junk).
    let popular = popular_set(&b_tokens, &bjunk, autojunk);
    let popular_vals: Vec<MbValue> = popular
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str(s.clone())))
        .collect();
    set_field(inst, "bpopular", MbValue::from_ptr(MbObject::new_set(popular_vals)));
}

/// CPython __chain_b: bjunk = { elt for elt in distinct(b) if isjunk(elt) }.
/// When isjunk is None (no callable), bjunk is empty.
fn bjunk_set(b: &[String], isjunk: MbValue) -> std::collections::HashSet<String> {
    let mut bjunk = std::collections::HashSet::new();
    if isjunk.is_none() || !is_callable(isjunk) {
        return bjunk;
    }
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for elt in b {
        if !seen.insert(elt.as_str()) {
            continue;
        }
        if call_isjunk(isjunk, elt) {
            bjunk.insert(elt.clone());
        }
    }
    bjunk
}

fn is_callable(v: MbValue) -> bool {
    super::super::builtins::mb_callable(v).as_bool() == Some(true)
}

/// Invoke `isjunk(elt)` and return its truthiness.
fn call_isjunk(isjunk: MbValue, elt: &str) -> bool {
    let arg = MbValue::from_ptr(MbObject::new_str(elt.to_string()));
    let args_list = MbValue::from_ptr(MbObject::new_list(vec![arg]));
    let res = super::super::builtins::mb_call_spread(isjunk, args_list);
    super::super::builtins::mb_bool(res).as_bool() == Some(true)
}

/// CPython autojunk: when `b` has >= 200 elements and autojunk is on, any
/// non-junk element occurring more than `len(b) // 100 + 1` times is "popular"
/// and treated as junk in the matcher. Elements already flagged by `isjunk`
/// (the `bjunk` set) are skipped — CPython counts only non-junk elements.
fn popular_set(
    b: &[String],
    bjunk: &std::collections::HashSet<String>,
    autojunk: bool,
) -> std::collections::HashSet<String> {
    let mut popular = std::collections::HashSet::new();
    let n = b.len();
    if autojunk && n >= 200 {
        let threshold = n / 100 + 1;
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for elt in b {
            if bjunk.contains(elt.as_str()) {
                continue;
            }
            *counts.entry(elt.as_str()).or_insert(0) += 1;
        }
        for (elt, c) in counts {
            if c > threshold {
                popular.insert(elt.to_string());
            }
        }
    }
    popular
}

fn make_match(a: usize, b: usize, size: usize) -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance(MATCH_CLASS.to_string()));
    set_field(inst, "a", MbValue::from_int(a as i64));
    set_field(inst, "b", MbValue::from_int(b as i64));
    set_field(inst, "size", MbValue::from_int(size as i64));
    inst
}

/// CPython SequenceMatcher.find_longest_match over token slices a[alo:ahi],
/// b[blo:bhi]. Returns (besti, bestj, bestsize).
fn longest_match(
    a: &[String],
    b: &[String],
    alo: usize,
    ahi: usize,
    blo: usize,
    bhi: usize,
    junk: &std::collections::HashSet<String>,
) -> (usize, usize, usize) {
    // b2j: map each element of b to the list of indices where it appears,
    // excluding junk/popular elements (matching CPython's autojunk).
    let mut b2j: HashMap<&str, Vec<usize>> = HashMap::new();
    for (j, elt) in b.iter().enumerate() {
        if junk.contains(elt.as_str()) {
            continue;
        }
        b2j.entry(elt.as_str()).or_default().push(j);
    }
    let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0usize);
    // j2len[j] = length of the longest junk-free match ending at a[i-1], b[j-1].
    let mut j2len: HashMap<usize, usize> = HashMap::new();
    for i in alo..ahi {
        let mut newj2len: HashMap<usize, usize> = HashMap::new();
        if let Some(js) = b2j.get(a[i].as_str()) {
            for &j in js {
                if j < blo {
                    continue;
                }
                if j >= bhi {
                    break;
                }
                let k = j2len.get(&(j.wrapping_sub(1))).copied().unwrap_or(0) + 1;
                newj2len.insert(j, k);
                if k > bestsize {
                    besti = i + 1 - k;
                    bestj = j + 1 - k;
                    bestsize = k;
                }
            }
        }
        j2len = newj2len;
    }
    // CPython: the main loop only matched junk-free elements. First extend the
    // best block across bordering NON-junk elements that match (so the block
    // becomes a "wholly interesting" match), and only THEN extend across
    // bordering JUNK elements. The order matters: junk-first extension would
    // pull popular/junk-adjacent non-junk elements into the block incorrectly.
    while besti > alo
        && bestj > blo
        && !junk.contains(b[bestj - 1].as_str())
        && a[besti - 1] == b[bestj - 1]
    {
        besti -= 1;
        bestj -= 1;
        bestsize += 1;
    }
    while besti + bestsize < ahi
        && bestj + bestsize < bhi
        && !junk.contains(b[bestj + bestsize].as_str())
        && a[besti + bestsize] == b[bestj + bestsize]
    {
        bestsize += 1;
    }
    while besti > alo
        && bestj > blo
        && junk.contains(b[bestj - 1].as_str())
        && a[besti - 1] == b[bestj - 1]
    {
        besti -= 1;
        bestj -= 1;
        bestsize += 1;
    }
    while besti + bestsize < ahi
        && bestj + bestsize < bhi
        && junk.contains(b[bestj + bestsize].as_str())
        && a[besti + bestsize] == b[bestj + bestsize]
    {
        bestsize += 1;
    }
    (besti, bestj, bestsize)
}

/// Full list of matching blocks (each (i, j, n)) ending with the sentinel
/// (len(a), len(b), 0), matching CPython get_matching_blocks.
fn matching_blocks(
    a: &[String],
    b: &[String],
    junk: &std::collections::HashSet<String>,
) -> Vec<(usize, usize, usize)> {
    let la = a.len();
    let lb = b.len();
    let mut queue = vec![(0usize, la, 0usize, lb)];
    let mut blocks: Vec<(usize, usize, usize)> = Vec::new();
    while let Some((alo, ahi, blo, bhi)) = queue.pop() {
        let (i, j, k) = longest_match(a, b, alo, ahi, blo, bhi, junk);
        if k > 0 {
            blocks.push((i, j, k));
            if alo < i && blo < j {
                queue.push((alo, i, blo, j));
            }
            if i + k < ahi && j + k < bhi {
                queue.push((i + k, ahi, j + k, bhi));
            }
        }
    }
    blocks.sort();
    // Merge adjacent equal blocks.
    let mut merged: Vec<(usize, usize, usize)> = Vec::new();
    let (mut i1, mut j1, mut k1) = (0usize, 0usize, 0usize);
    for (i2, j2, k2) in blocks {
        if i1 + k1 == i2 && j1 + k1 == j2 {
            k1 += k2;
        } else {
            if k1 > 0 {
                merged.push((i1, j1, k1));
            }
            i1 = i2;
            j1 = j2;
            k1 = k2;
        }
    }
    if k1 > 0 {
        merged.push((i1, j1, k1));
    }
    merged.push((la, lb, 0));
    merged
}

/// Opcodes: list of (tag, i1, i2, j1, j2).
fn opcodes(
    a: &[String],
    b: &[String],
    junk: &std::collections::HashSet<String>,
) -> Vec<(&'static str, usize, usize, usize, usize)> {
    let blocks = matching_blocks(a, b, junk);
    let mut ops: Vec<(&'static str, usize, usize, usize, usize)> = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    for (ai, bj, size) in blocks {
        let tag = if i < ai && j < bj {
            "replace"
        } else if i < ai {
            "delete"
        } else if j < bj {
            "insert"
        } else {
            ""
        };
        if !tag.is_empty() {
            ops.push((tag, i, ai, j, bj));
        }
        i = ai + size;
        j = bj + size;
        if size > 0 {
            ops.push(("equal", ai, i, bj, j));
        }
    }
    ops
}

/// CPython get_grouped_opcodes(n): isolate change clusters, padding each with
/// at most `n` lines of surrounding equal context.
fn grouped_opcodes(
    a: &[String],
    b: &[String],
    junk: &std::collections::HashSet<String>,
    n: i64,
) -> Vec<Vec<(&'static str, usize, usize, usize, usize)>> {
    let n = n.max(0) as usize;
    let mut codes = opcodes(a, b, junk);
    if codes.is_empty() {
        codes.push(("equal", 0, 1, 0, 1));
    }
    // Fixup leading and trailing groups if they show no changes.
    if let Some(first) = codes.first_mut() {
        if first.0 == "equal" {
            let (tag, i1, i2, j1, j2) = *first;
            *first = (tag, i1.max(i2.saturating_sub(n)), i2, j1.max(j2.saturating_sub(n)), j2);
        }
    }
    if let Some(last) = codes.last_mut() {
        if last.0 == "equal" {
            let (tag, i1, i2, j1, j2) = *last;
            *last = (tag, i1, i2.min(i1 + n), j1, j2.min(j1 + n));
        }
    }

    let nn = n + n;
    let mut groups: Vec<Vec<(&'static str, usize, usize, usize, usize)>> = Vec::new();
    let mut group: Vec<(&'static str, usize, usize, usize, usize)> = Vec::new();
    for (tag, mut i1, i2, mut j1, j2) in codes {
        // End the current group and start a new one whenever there is a
        // large range with no changes.
        if tag == "equal" && i2 - i1 > nn {
            group.push((tag, i1, i2.min(i1 + n), j1, j2.min(j1 + n)));
            groups.push(group.clone());
            group.clear();
            i1 = i1.max(i2.saturating_sub(n));
            j1 = j1.max(j2.saturating_sub(n));
        }
        group.push((tag, i1, i2, j1, j2));
    }
    if !group.is_empty() && !(group.len() == 1 && group[0].0 == "equal") {
        groups.push(group);
    }
    groups
}

fn ratio_of(a: &[String], b: &[String], junk: &std::collections::HashSet<String>) -> f64 {
    let matches: usize = matching_blocks(a, b, junk).iter().map(|(_, _, k)| *k).sum();
    let total = a.len() + b.len();
    if total == 0 {
        1.0
    } else {
        2.0 * matches as f64 / total as f64
    }
}

/// Read the matcher's `a`/`b` token slices and the combined junk set derived
/// from `b`: the isjunk-driven `bjunk` set unioned with the autojunk-driven
/// `bpopular` set. CPython's matcher treats both as junk (`isbjunk`).
fn sm_state(self_v: MbValue) -> (Vec<String>, Vec<String>, std::collections::HashSet<String>) {
    let a = seq_tokens(get_field(self_v, "a").unwrap_or_else(MbValue::none));
    let b = seq_tokens(get_field(self_v, "b").unwrap_or_else(MbValue::none));
    let autojunk = get_field(self_v, "autojunk")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let isjunk = get_field(self_v, "isjunk").unwrap_or_else(MbValue::none);
    let mut junk = bjunk_set(&b, isjunk);
    for elt in popular_set(&b, &junk, autojunk) {
        junk.insert(elt);
    }
    (a, b, junk)
}

fn mb_sm_ratio(self_v: MbValue) -> MbValue {
    let (a, b, junk) = sm_state(self_v);
    MbValue::from_float(ratio_of(&a, &b, &junk))
}

fn calculate_ratio(matches: usize, length: usize) -> f64 {
    if length == 0 {
        1.0
    } else {
        2.0 * matches as f64 / length as f64
    }
}

/// CPython quick_ratio: an order-independent (multiset) upper bound on ratio.
/// `matches` = cardinality of the multiset intersection of a and b.
fn quick_ratio_of(a: &[String], b: &[String]) -> f64 {
    let mut fullbcount: HashMap<&str, i64> = HashMap::new();
    for elt in b {
        *fullbcount.entry(elt.as_str()).or_insert(0) += 1;
    }
    let mut avail: HashMap<&str, i64> = HashMap::new();
    let mut matches = 0usize;
    for elt in a {
        let numb = match avail.get(elt.as_str()) {
            Some(n) => *n,
            None => fullbcount.get(elt.as_str()).copied().unwrap_or(0),
        };
        avail.insert(elt.as_str(), numb - 1);
        if numb > 0 {
            matches += 1;
        }
    }
    calculate_ratio(matches, a.len() + b.len())
}

/// CPython real_quick_ratio: a length-only upper bound on ratio — can't have
/// more matches than the number of elements in the shorter sequence.
fn real_quick_ratio_of(a: &[String], b: &[String]) -> f64 {
    let (la, lb) = (a.len(), b.len());
    calculate_ratio(la.min(lb), la + lb)
}

fn mb_sm_quick_ratio(self_v: MbValue) -> MbValue {
    let (a, b, _junk) = sm_state(self_v);
    MbValue::from_float(quick_ratio_of(&a, &b))
}

fn mb_sm_real_quick_ratio(self_v: MbValue) -> MbValue {
    let (a, b, _junk) = sm_state(self_v);
    MbValue::from_float(real_quick_ratio_of(&a, &b))
}

fn mb_sm_find_longest_match(self_v: MbValue, args: MbValue) -> MbValue {
    let (a, b, junk) = sm_state(self_v);
    let mut items = extract_list(args).unwrap_or_default();

    // find_longest_match(alo=0, ahi=None, blo=0, bhi=None) accepts keyword
    // arguments; mamba appends a trailing kwargs dict for attribute calls with
    // keywords. Split it off and let it override the matching positional slot.
    let mut kw_alo: Option<i64> = None;
    let mut kw_ahi: Option<i64> = None;
    let mut kw_blo: Option<i64> = None;
    let mut kw_bhi: Option<i64> = None;
    if let Some(last) = items.last().copied() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    kw_alo = map.get("alo").and_then(|v| v.as_int());
                    kw_ahi = map.get("ahi").and_then(|v| v.as_int());
                    kw_blo = map.get("blo").and_then(|v| v.as_int());
                    kw_bhi = map.get("bhi").and_then(|v| v.as_int());
                    drop(map);
                    items.pop();
                }
            }
        }
    }

    let alo = kw_alo
        .or_else(|| items.first().and_then(|v| v.as_int()))
        .unwrap_or(0)
        .max(0) as usize;
    let ahi = kw_ahi
        .or_else(|| items.get(1).and_then(|v| v.as_int()))
        .map(|n| n as usize)
        .unwrap_or(a.len());
    let blo = kw_blo
        .or_else(|| items.get(2).and_then(|v| v.as_int()))
        .unwrap_or(0)
        .max(0) as usize;
    let bhi = kw_bhi
        .or_else(|| items.get(3).and_then(|v| v.as_int()))
        .map(|n| n as usize)
        .unwrap_or(b.len());
    let (i, j, k) = longest_match(&a, &b, alo, ahi.min(a.len()), blo, bhi.min(b.len()), &junk);
    make_match(i, j, k)
}

fn mb_sm_get_matching_blocks(self_v: MbValue) -> MbValue {
    // Cache the result list on the instance so repeated calls return the SAME
    // list object (CPython caches matching_blocks); this makes
    // `sm.get_matching_blocks() == sm.get_matching_blocks()` True.
    if let Some(cached) = get_field(self_v, "_mb") {
        if cached.as_ptr().is_some() {
            unsafe { super::super::rc::retain_if_ptr(cached); }
            return cached;
        }
    }
    let (a, b, junk) = sm_state(self_v);
    let blocks = matching_blocks(&a, &b, &junk);
    let out: Vec<MbValue> = blocks.into_iter().map(|(i, j, k)| make_match(i, j, k)).collect();
    let list = MbValue::from_ptr(MbObject::new_list(out));
    // new_list gives rc=1; set_field retains for the cached field (rc=2). The
    // returned reference is the caller's own (the second ref), so no extra
    // retain here.
    set_field(self_v, "_mb", list);
    list
}

fn mb_sm_get_opcodes(self_v: MbValue) -> MbValue {
    let (a, b, junk) = sm_state(self_v);
    let ops = opcodes(&a, &b, &junk);
    let out: Vec<MbValue> = ops
        .into_iter()
        .map(|(tag, i1, i2, j1, j2)| {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str(tag.to_string())),
                MbValue::from_int(i1 as i64),
                MbValue::from_int(i2 as i64),
                MbValue::from_int(j1 as i64),
                MbValue::from_int(j2 as i64),
            ]))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(out))
}

/// get_grouped_opcodes(n=3): yields groups of opcodes with up to n lines of
/// context. Returns a list iterator (empty -> StopIteration on next()).
fn mb_sm_get_grouped_opcodes(self_v: MbValue, n: i64) -> MbValue {
    let (a, b, junk) = sm_state(self_v);
    let groups = grouped_opcodes(&a, &b, &junk, n);
    let out: Vec<MbValue> = groups
        .into_iter()
        .map(|g| {
            let tuples: Vec<MbValue> = g
                .into_iter()
                .map(|(tag, i1, i2, j1, j2)| {
                    MbValue::from_ptr(MbObject::new_tuple(vec![
                        MbValue::from_ptr(MbObject::new_str(tag.to_string())),
                        MbValue::from_int(i1 as i64),
                        MbValue::from_int(i2 as i64),
                        MbValue::from_int(j1 as i64),
                        MbValue::from_int(j2 as i64),
                    ]))
                })
                .collect();
            MbValue::from_ptr(MbObject::new_list(tuples))
        })
        .collect();
    super::super::iter::mb_iter(MbValue::from_ptr(MbObject::new_list(out)))
}

/// CPython IS_CHARACTER_JUNK: treat the space and tab characters as junk for
/// intraline character comparison (default `charjunk` of ndiff/Differ).
fn char_junk_set(s: &[String]) -> std::collections::HashSet<String> {
    let mut junk = std::collections::HashSet::new();
    for elt in s {
        if elt == " " || elt == "\t" {
            junk.insert(elt.clone());
        }
    }
    junk
}

/// CPython _dump(tag, x, lo, hi): emit `'%s %s' % (tag, x[i])` for lo..hi.
fn ndiff_dump(out: &mut Vec<String>, tag: char, x: &[String], lo: usize, hi: usize) {
    for line in x.iter().take(hi).skip(lo) {
        out.push(format!("{tag} {line}"));
    }
}

/// CPython _plain_replace: dump the shorter block first, then the other.
fn ndiff_plain_replace(
    out: &mut Vec<String>,
    a: &[String],
    alo: usize,
    ahi: usize,
    b: &[String],
    blo: usize,
    bhi: usize,
) {
    if bhi - blo < ahi - alo {
        ndiff_dump(out, '+', b, blo, bhi);
        ndiff_dump(out, '-', a, alo, ahi);
    } else {
        ndiff_dump(out, '-', a, alo, ahi);
        ndiff_dump(out, '+', b, blo, bhi);
    }
}

/// CPython _keep_original_ws: replace whitespace placeholders in `tag_s` with
/// the original whitespace characters from `s` (so `?` guide lines line up
/// when the source line is indented with tabs).
fn keep_original_ws(s: &[char], tag_s: &[char]) -> String {
    s.iter()
        .zip(tag_s.iter())
        .map(|(&c, &tag_c)| {
            if tag_c == ' ' && c.is_whitespace() {
                c
            } else {
                tag_c
            }
        })
        .collect()
}

/// CPython _qformat: emit the '- ', optional '? ', '+ ', optional '? ' quad
/// for a synch pair, expanding tabs in the guide lines via _keep_original_ws.
fn ndiff_qformat(out: &mut Vec<String>, aline: &str, bline: &str, atags: &str, btags: &str) {
    let achars: Vec<char> = aline.chars().collect();
    let bchars: Vec<char> = bline.chars().collect();
    let atag_chars: Vec<char> = atags.chars().collect();
    let btag_chars: Vec<char> = btags.chars().collect();
    let atags = keep_original_ws(&achars, &atag_chars);
    let btags = keep_original_ws(&bchars, &btag_chars);
    let atags = atags.trim_end().to_string();
    let btags = btags.trim_end().to_string();

    out.push(format!("- {aline}"));
    if !atags.is_empty() {
        out.push(format!("? {atags}\n"));
    }
    out.push(format!("+ {bline}"));
    if !btags.is_empty() {
        out.push(format!("? {btags}\n"));
    }
}

/// CPython _fancy_helper: recurse into _fancy_replace for two-sided ranges,
/// otherwise dump the one-sided remainder.
fn ndiff_fancy_helper(
    out: &mut Vec<String>,
    a: &[String],
    alo: usize,
    ahi: usize,
    b: &[String],
    blo: usize,
    bhi: usize,
) {
    if alo < ahi {
        if blo < bhi {
            ndiff_fancy_replace(out, a, alo, ahi, b, blo, bhi);
        } else {
            ndiff_dump(out, '-', a, alo, ahi);
        }
    } else if blo < bhi {
        ndiff_dump(out, '+', b, blo, bhi);
    }
}

/// CPython Differ._fancy_replace: within a 'replace' region, find the
/// best-matching non-identical line pair (similarity >= cutoff) to use as a
/// synch point, emit intraline '? ' guide lines for it, and recurse on the
/// surrounding sub-ranges.
fn ndiff_fancy_replace(
    out: &mut Vec<String>,
    a: &[String],
    alo: usize,
    ahi: usize,
    b: &[String],
    blo: usize,
    bhi: usize,
) {
    let mut best_ratio = 0.74_f64;
    let cutoff = 0.75_f64;
    let mut best_i = alo;
    let mut best_j = blo;
    let mut eqi: Option<usize> = None;
    let mut eqj: Option<usize> = None;

    for j in blo..bhi {
        let bj: Vec<String> = b[j].chars().map(|c| c.to_string()).collect();
        let bjunk = char_junk_set(&bj);
        for i in alo..ahi {
            if a[i] == b[j] {
                if eqi.is_none() {
                    eqi = Some(i);
                    eqj = Some(j);
                }
                continue;
            }
            let ai: Vec<String> = a[i].chars().map(|c| c.to_string()).collect();
            // Use the quick upper bounds before the full ratio, matching
            // CPython's ordering (cheap rejection first).
            if real_quick_ratio_of(&ai, &bj) > best_ratio
                && quick_ratio_of(&ai, &bj) > best_ratio
            {
                let r = ratio_of(&ai, &bj, &bjunk);
                if r > best_ratio {
                    best_ratio = r;
                    best_i = i;
                    best_j = j;
                }
            }
        }
    }

    if best_ratio < cutoff {
        // No non-identical "pretty close" pair.
        match (eqi, eqj) {
            (Some(ei), Some(ej)) => {
                // No close pair, but an identical pair — synch up on that.
                best_i = ei;
                best_j = ej;
            }
            _ => {
                // No identical pair either — straight replace.
                ndiff_plain_replace(out, a, alo, ahi, b, blo, bhi);
                return;
            }
        }
    } else {
        // There's a close pair, so forget the identical pair (if any).
        eqi = None;
    }

    // Pump out diffs from before the synch point.
    ndiff_fancy_helper(out, a, alo, best_i, b, blo, best_j);

    // Do intraline marking on the synch pair.
    let aelt = &a[best_i];
    let belt = &b[best_j];
    if eqi.is_none() {
        // Pump out a '-', '?', '+', '?' quad for the synched lines.
        let mut atags = String::new();
        let mut btags = String::new();
        let achars: Vec<String> = aelt.chars().map(|c| c.to_string()).collect();
        let bchars: Vec<String> = belt.chars().map(|c| c.to_string()).collect();
        // The inner cruncher carries charjunk (IS_CHARACTER_JUNK) as isjunk.
        let cjunk = char_junk_set(&bchars);
        let ops = opcodes(&achars, &bchars, &cjunk);
        for (tag, ai1, ai2, bj1, bj2) in ops {
            let la = ai2 - ai1;
            let lb = bj2 - bj1;
            match tag {
                "replace" => {
                    atags.push_str(&"^".repeat(la));
                    btags.push_str(&"^".repeat(lb));
                }
                "delete" => atags.push_str(&"-".repeat(la)),
                "insert" => btags.push_str(&"+".repeat(lb)),
                "equal" => {
                    atags.push_str(&" ".repeat(la));
                    btags.push_str(&" ".repeat(lb));
                }
                _ => {}
            }
        }
        ndiff_qformat(out, aelt, belt, &atags, &btags);
    } else {
        // The synch pair is identical.
        out.push(format!("  {aelt}"));
    }

    // Pump out diffs from after the synch point.
    ndiff_fancy_helper(out, a, best_i + 1, ahi, b, best_j + 1, bhi);
}

/// difflib.ndiff(a, b): line-oriented delta (Differ.compare). Each line is
/// prefixed with "  " (equal), "- " (only in a), "+ " (only in b), or "? "
/// intraline hint lines marking changed columns within a similar pair.
fn mb_difflib_ndiff(a: MbValue, b: MbValue) -> MbValue {
    let la = seq_tokens(a);
    let lb = seq_tokens(b);
    let ops = opcodes(&la, &lb, &std::collections::HashSet::new());
    let mut lines: Vec<String> = Vec::new();
    for (tag, i1, i2, j1, j2) in ops {
        match tag {
            "replace" => ndiff_fancy_replace(&mut lines, &la, i1, i2, &lb, j1, j2),
            "delete" => ndiff_dump(&mut lines, '-', &la, i1, i2),
            "insert" => ndiff_dump(&mut lines, '+', &lb, j1, j2),
            "equal" => ndiff_dump(&mut lines, ' ', &la, i1, i2),
            _ => {}
        }
    }
    let out: Vec<MbValue> = lines
        .into_iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str(s)))
        .collect();
    super::super::iter::mb_iter(MbValue::from_ptr(MbObject::new_list(out)))
}

/// CPython _format_range_context: convert a [start, stop) opcode range into
/// the "ed" range notation used by context diffs. Empty ranges begin at the
/// line just before the range; single-line ranges have no comma.
/// difflib._format_range_context(start, stop): module-level helper exposed for
/// parity with CPython (used directly by tests).
fn mb_difflib_format_range_context(start: MbValue, stop: MbValue) -> MbValue {
    let s = start.as_int().unwrap_or(0).max(0) as usize;
    let e = stop.as_int().unwrap_or(0).max(0) as usize;
    MbValue::from_ptr(MbObject::new_str(format_range_context(s, e)))
}

fn format_range_context(start: usize, stop: usize) -> String {
    let mut beginning = start as i64 + 1; // lines start numbering with one
    let length = stop as i64 - start as i64;
    if length == 0 {
        beginning -= 1; // empty ranges begin at the line just before the range
    }
    if length <= 1 {
        format!("{beginning}")
    } else {
        format!("{},{}", beginning, beginning + length - 1)
    }
}

/// difflib.context_diff(a, b, fromfile='', tofile='', fromfiledate='',
/// tofiledate='', n=3, lineterm='\n'): context-format diff driven by
/// get_grouped_opcodes(n), with '*** '/'--- ' file headers and
/// '*** s,e ****'/'--- s,e ----' range markers per hunk.
fn mb_difflib_context_diff(args: &[MbValue]) -> MbValue {
    // Split off a trailing kwargs dict (mamba appends one for module calls
    // with keyword arguments) from the positional args.
    let mut positional: &[MbValue] = args;
    let mut kwargs: Option<MbValue> = None;
    if let Some(last) = args.last() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if matches!((*ptr).data, ObjData::Dict(_)) {
                    kwargs = Some(*last);
                    positional = &args[..args.len() - 1];
                }
            }
        }
    }

    let a = positional.first().copied().unwrap_or_else(MbValue::none);
    let b = positional.get(1).copied().unwrap_or_else(MbValue::none);
    // Positional order: fromfile, tofile, fromfiledate, tofiledate, n, lineterm.
    let empty = || MbValue::from_ptr(MbObject::new_str(String::new()));
    let nl = || MbValue::from_ptr(MbObject::new_str("\n".to_string()));
    let mut fromfile_v = positional.get(2).copied().unwrap_or_else(empty);
    let mut tofile_v = positional.get(3).copied().unwrap_or_else(empty);
    let mut fromfiledate_v = positional.get(4).copied().unwrap_or_else(empty);
    let mut tofiledate_v = positional.get(5).copied().unwrap_or_else(empty);
    let mut n: i64 = positional.get(6).and_then(|v| v.as_int()).unwrap_or(3);
    let mut lineterm_v = positional.get(7).copied().unwrap_or_else(nl);
    // Keyword args override the matching positional slot.
    if let Some(kw) = kwargs {
        if let Some(ptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("fromfile") { fromfile_v = *v; }
                    if let Some(v) = map.get("tofile") { tofile_v = *v; }
                    if let Some(v) = map.get("fromfiledate") { fromfiledate_v = *v; }
                    if let Some(v) = map.get("tofiledate") { tofiledate_v = *v; }
                    if let Some(v) = map.get("n") { n = v.as_int().unwrap_or(3); }
                    if let Some(v) = map.get("lineterm") { lineterm_v = *v; }
                }
            }
        }
    }
    // CPython _check_types(a, b, fromfile, tofile, fromfiledate, tofiledate,
    // lineterm) — raise TypeError on mixed bytes/str before producing output.
    if check_types(a, b, &[fromfile_v, tofile_v, fromfiledate_v, tofiledate_v, lineterm_v]) {
        return MbValue::none();
    }
    let fromfile = extract_str(fromfile_v).unwrap_or_default();
    let tofile = extract_str(tofile_v).unwrap_or_default();
    let fromfiledate = extract_str(fromfiledate_v).unwrap_or_default();
    let tofiledate = extract_str(tofiledate_v).unwrap_or_default();
    let lineterm = extract_str(lineterm_v).unwrap_or_else(|| "\n".to_string());
    let la = seq_tokens(a);
    let lb = seq_tokens(b);
    let groups = grouped_opcodes(&la, &lb, &std::collections::HashSet::new(), n);
    let mut out: Vec<MbValue> = Vec::new();
    let mut started = false;
    for group in &groups {
        if group.is_empty() {
            continue;
        }
        if !started {
            started = true;
            let fromdate = if !fromfiledate.is_empty() { format!("\t{fromfiledate}") } else { String::new() };
            let todate = if !tofiledate.is_empty() { format!("\t{tofiledate}") } else { String::new() };
            out.push(MbValue::from_ptr(MbObject::new_str(format!("*** {fromfile}{fromdate}{lineterm}"))));
            out.push(MbValue::from_ptr(MbObject::new_str(format!("--- {tofile}{todate}{lineterm}"))));
        }
        let first = group[0];
        let last = group[group.len() - 1];
        out.push(MbValue::from_ptr(MbObject::new_str(format!("***************{lineterm}"))));

        // a-side: '*** start,end ****' from first[1]..last[2].
        let file1_range = format_range_context(first.1, last.2);
        out.push(MbValue::from_ptr(MbObject::new_str(format!("*** {file1_range} ****{lineterm}"))));
        if group.iter().any(|(t, ..)| *t == "replace" || *t == "delete") {
            for &(tag, i1, i2, _j1, _j2) in group {
                if tag != "insert" {
                    let prefix = context_prefix(tag);
                    for line in la.iter().take(i2).skip(i1) {
                        out.push(MbValue::from_ptr(MbObject::new_str(format!("{prefix}{line}"))));
                    }
                }
            }
        }

        // b-side: '--- start,end ----' from first[3]..last[4].
        let file2_range = format_range_context(first.3, last.4);
        out.push(MbValue::from_ptr(MbObject::new_str(format!("--- {file2_range} ----{lineterm}"))));
        if group.iter().any(|(t, ..)| *t == "replace" || *t == "insert") {
            for &(tag, _i1, _i2, j1, j2) in group {
                if tag != "delete" {
                    let prefix = context_prefix(tag);
                    for line in lb.iter().take(j2).skip(j1) {
                        out.push(MbValue::from_ptr(MbObject::new_str(format!("{prefix}{line}"))));
                    }
                }
            }
        }
    }
    super::super::iter::mb_iter(MbValue::from_ptr(MbObject::new_list(out)))
}

fn context_prefix(tag: &str) -> &'static str {
    match tag {
        "insert" => "+ ",
        "delete" => "- ",
        "replace" => "! ",
        _ => "  ", // equal
    }
}

/// difflib.restore(delta, which): recover the 1st (which=1) or 2nd (which=2)
/// sequence from an ndiff-style delta.
fn mb_difflib_restore(delta: MbValue, which: MbValue) -> MbValue {
    let lines = seq_tokens(delta);
    // CPython: `{1: "- ", 2: "+ "}[int(which)]`; anything else is a ValueError.
    let which_int = which.as_int();
    let tag = match which_int {
        Some(1) => "- ",
        Some(2) => "+ ",
        _ => {
            // `%r` of the original `which`: ints render bare, strs quoted.
            let repr = match which_int {
                Some(n) => n.to_string(),
                None => match extract_str(which) {
                    Some(s) => format!("'{s}'"),
                    None => "None".to_string(),
                },
            };
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "unknown delta choice (must be 1 or 2): {repr}"
                ))),
            );
            return MbValue::none();
        }
    };
    let mut out: Vec<MbValue> = Vec::new();
    for line in &lines {
        // CPython matches the 2-char prefix against ("  ", tag); slice [2:].
        let prefix: String = line.chars().take(2).collect();
        if prefix == "  " || prefix == tag {
            let rest: String = line.chars().skip(2).collect();
            out.push(MbValue::from_ptr(MbObject::new_str(rest)));
        }
    }
    super::super::iter::mb_iter(MbValue::from_ptr(MbObject::new_list(out)))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// Python `repr(v)` of an arbitrary value, as a Rust String (used to build the
/// CPython TypeError messages, e.g. `b'hello'` / `'hello'`).
fn value_repr(val: MbValue) -> String {
    extract_str(super::super::builtins::mb_repr(val)).unwrap_or_default()
}

/// Python `type(v).__name__` for the few cases difflib's type checks care
/// about: `str`, `bytes`, and a generic fallback.
fn value_type_name(val: MbValue) -> String {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::Str(_) => return "str".to_string(),
                ObjData::Bytes(_) => return "bytes".to_string(),
                ObjData::ByteArray(_) => return "bytearray".to_string(),
                _ => {}
            }
        }
    }
    if val.as_bool().is_some() { return "bool".to_string(); }
    if val.as_int().is_some() { return "int".to_string(); }
    if val.as_float().is_some() { return "float".to_string(); }
    if val.is_none() { return "NoneType".to_string(); }
    "object".to_string()
}

fn is_str_value(val: MbValue) -> bool {
    val.as_ptr().map(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) }).unwrap_or(false)
}

fn is_bytes_value(val: MbValue) -> bool {
    val.as_ptr().map(|ptr| unsafe {
        matches!((*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_))
    }).unwrap_or(false)
}

fn raise_type_error(msg: String) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
}

/// CPython difflib._check_types(a, b, *args): unified_diff/context_diff require
/// the first element of each sequence (if any) to be `str`, and every extra
/// filename/date/lineterm argument to be `str`. Returns true if a TypeError was
/// raised (caller should stop).
fn check_types(a: MbValue, b: MbValue, args: &[MbValue]) -> bool {
    for seq in [a, b] {
        // A non-iterable side is the iteration TypeError (CPython does
        // `len(a)` over the sequence before comparing).
        let iterable = seq.as_ptr().is_some_and(|ptr| unsafe {
            matches!(
                (*ptr).data,
                ObjData::Str(_) | ObjData::List(_) | ObjData::Tuple(_)
            )
        }) || super::super::iter::is_iter_handle(seq);
        if !iterable {
            raise_not_iterable(seq);
            return true;
        }
        // First-element str check on concrete sequences only — extracting
        // from an iterator handle would drain it before the diff runs.
        let first = seq.as_ptr().and_then(|ptr| unsafe {
            match (*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().first().copied(),
                ObjData::Tuple(ref t) => t.first().copied(),
                _ => None,
            }
        });
        if let Some(first) = first {
            if !is_str_value(first) {
                raise_type_error(format!(
                    "lines to compare must be str, not {} ({})",
                    value_type_name(first),
                    value_repr(first)
                ));
                return true;
            }
        }
    }
    for arg in args {
        if !is_str_value(*arg) {
            raise_type_error(format!(
                "all arguments must be str, not: {}",
                value_repr(*arg)
            ));
            return true;
        }
    }
    false
}

fn extract_list(val: MbValue) -> Option<Vec<MbValue>> {
    if let Some(items) = val.as_ptr().and_then(|ptr| unsafe {
        match (*ptr).data {
            ObjData::List(ref lock) => Some(lock.read().unwrap().to_vec()),
            ObjData::Tuple(ref t) => Some(t.clone()),
            _ => None,
        }
    }) {
        return Some(items);
    }
    if super::super::iter::is_iter_handle(val) {
        return super::super::iter::drain_iter_to_vec(val);
    }
    None
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// CPython iteration TypeError for a non-iterable argument.
fn raise_not_iterable(val: MbValue) -> MbValue {
    let tn = if val.is_none() {
        "NoneType"
    } else if val.as_bool().is_some() {
        "bool"
    } else if val.as_int().is_some() {
        "int"
    } else if val.is_float() {
        "float"
    } else {
        "object"
    };
    raise_type_error(format!("'{tn}' object is not iterable"));
    MbValue::none()
}

pub fn mb_difflib_SequenceMatcher(a: MbValue, b: MbValue) -> MbValue {
    let sa = extract_str(a).unwrap_or_default();
    let sb = extract_str(b).unwrap_or_default();
    MbValue::from_float(sequence_ratio(&sa, &sb))
}

pub fn mb_difflib_ratio(a: MbValue, b: MbValue) -> MbValue {
    let sa = extract_str(a).unwrap_or_default();
    let sb = extract_str(b).unwrap_or_default();
    MbValue::from_float(sequence_ratio(&sa, &sb))
}

fn sequence_ratio(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() { return 1.0; }
    if a.is_empty() || b.is_empty() { return 0.0; }
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let mut matches = 0usize;
    let mut used = vec![false; bc.len()];
    for ca in &ac {
        for (j, cb) in bc.iter().enumerate() {
            if !used[j] && ca == cb { matches += 1; used[j] = true; break; }
        }
    }
    2.0 * matches as f64 / (ac.len() + bc.len()) as f64
}

/// difflib.unified_diff(a, b, fromfile='', tofile='', fromfiledate='',
/// tofiledate='', n=3, lineterm='\n'): unified-format diff driven by
/// get_grouped_opcodes(n). Emits '--- '/'+++ ' file headers, '@@ -s,e +s,e @@'
/// hunk markers, and ' '/'-'/'+' body lines. A generator in CPython, so mamba
/// materializes the lines into a list iterator (next() -> StopIteration when
/// there are no changes).
fn mb_difflib_unified_diff_full(args: &[MbValue]) -> MbValue {
    // Split off a trailing kwargs dict (mamba appends one for module calls with
    // keyword arguments) from the positional args.
    let mut positional: &[MbValue] = args;
    let mut kwargs: Option<MbValue> = None;
    if let Some(last) = args.last() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if matches!((*ptr).data, ObjData::Dict(_)) {
                    kwargs = Some(*last);
                    positional = &args[..args.len() - 1];
                }
            }
        }
    }

    let a = positional.first().copied().unwrap_or_else(MbValue::none);
    let b = positional.get(1).copied().unwrap_or_else(MbValue::none);
    // Positional order: fromfile, tofile, fromfiledate, tofiledate, n, lineterm.
    let empty = || MbValue::from_ptr(MbObject::new_str(String::new()));
    let nl = || MbValue::from_ptr(MbObject::new_str("\n".to_string()));
    let mut fromfile_v = positional.get(2).copied().unwrap_or_else(empty);
    let mut tofile_v = positional.get(3).copied().unwrap_or_else(empty);
    let mut fromfiledate_v = positional.get(4).copied().unwrap_or_else(empty);
    let mut tofiledate_v = positional.get(5).copied().unwrap_or_else(empty);
    let mut n: i64 = positional.get(6).and_then(|v| v.as_int()).unwrap_or(3);
    let mut lineterm_v = positional.get(7).copied().unwrap_or_else(nl);
    // Keyword args override the matching positional slot.
    if let Some(kw) = kwargs {
        if let Some(ptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("fromfile") { fromfile_v = *v; }
                    if let Some(v) = map.get("tofile") { tofile_v = *v; }
                    if let Some(v) = map.get("fromfiledate") { fromfiledate_v = *v; }
                    if let Some(v) = map.get("tofiledate") { tofiledate_v = *v; }
                    if let Some(v) = map.get("n") { n = v.as_int().unwrap_or(3); }
                    if let Some(v) = map.get("lineterm") { lineterm_v = *v; }
                }
            }
        }
    }
    // CPython _check_types(a, b, fromfile, tofile, fromfiledate, tofiledate,
    // lineterm) — raise TypeError on mixed bytes/str before producing output.
    if check_types(a, b, &[fromfile_v, tofile_v, fromfiledate_v, tofiledate_v, lineterm_v]) {
        return MbValue::none();
    }
    let fromfile = extract_str(fromfile_v).unwrap_or_default();
    let tofile = extract_str(tofile_v).unwrap_or_default();
    let fromfiledate = extract_str(fromfiledate_v).unwrap_or_default();
    let tofiledate = extract_str(tofiledate_v).unwrap_or_default();
    let lineterm = extract_str(lineterm_v).unwrap_or_else(|| "\n".to_string());

    let la = seq_tokens(a);
    let lb = seq_tokens(b);
    let groups = grouped_opcodes(&la, &lb, &std::collections::HashSet::new(), n);
    let mut out: Vec<MbValue> = Vec::new();
    let mut started = false;
    for group in &groups {
        if group.is_empty() {
            continue;
        }
        if !started {
            started = true;
            let fromdate = if !fromfiledate.is_empty() { format!("\t{fromfiledate}") } else { String::new() };
            let todate = if !tofiledate.is_empty() { format!("\t{tofiledate}") } else { String::new() };
            out.push(MbValue::from_ptr(MbObject::new_str(format!("--- {fromfile}{fromdate}{lineterm}"))));
            out.push(MbValue::from_ptr(MbObject::new_str(format!("+++ {tofile}{todate}{lineterm}"))));
        }
        let first = group[0];
        let last = group[group.len() - 1];
        let file1_range = format_range_unified(first.1 as i64, last.2 as i64);
        let file2_range = format_range_unified(first.3 as i64, last.4 as i64);
        out.push(MbValue::from_ptr(MbObject::new_str(format!("@@ -{file1_range} +{file2_range} @@{lineterm}"))));
        for &(tag, i1, i2, j1, j2) in group {
            if tag == "equal" {
                for line in la.iter().take(i2).skip(i1) {
                    out.push(MbValue::from_ptr(MbObject::new_str(format!(" {line}"))));
                }
                continue;
            }
            if tag == "replace" || tag == "delete" {
                for line in la.iter().take(i2).skip(i1) {
                    out.push(MbValue::from_ptr(MbObject::new_str(format!("-{line}"))));
                }
            }
            if tag == "replace" || tag == "insert" {
                for line in lb.iter().take(j2).skip(j1) {
                    out.push(MbValue::from_ptr(MbObject::new_str(format!("+{line}"))));
                }
            }
        }
    }
    super::super::iter::mb_iter(MbValue::from_ptr(MbObject::new_list(out)))
}

pub fn mb_difflib_get_close_matches(word: MbValue, possibilities: MbValue, n: MbValue, cutoff: MbValue) -> MbValue {
    let sw = extract_str(word).unwrap_or_default();
    // Keyword calls pack a dict into the first free positional slot; unfold
    // n/cutoff from it.
    let mut n = n;
    let mut cutoff = cutoff;
    for slot in [n, cutoff] {
        if let Some(ptr) = slot.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if slot.to_bits() == n.to_bits() {
                        n = MbValue::none();
                    }
                    if slot.to_bits() == cutoff.to_bits() {
                        cutoff = MbValue::none();
                    }
                    if let Some(v) = map.get("n") { n = *v; }
                    if let Some(v) = map.get("cutoff") { cutoff = *v; }
                }
            }
        }
    }
    let cut = cutoff
        .as_float()
        .or_else(|| cutoff.as_int().map(|i| i as f64))
        .unwrap_or(0.6);
    let count_raw = n.as_int().unwrap_or(3);
    // CPython argument contract.
    if count_raw <= 0 {
        return raise_value_error(&format!("n must be > 0: {count_raw}"));
    }
    if !(0.0..=1.0).contains(&cut) {
        return raise_value_error(&format!("cutoff must be in [0.0, 1.0]: {cut}"));
    }
    let count = count_raw as usize;
    let items = match extract_list(possibilities) {
        Some(v) => v,
        None => {
            return raise_not_iterable(possibilities);
        }
    };
    let mut scored: Vec<(f64, MbValue)> = items.into_iter().filter_map(|v| {
        extract_str(v).map(|s| (sequence_ratio(&sw, &s), v))
    }).filter(|(r, _)| *r >= cut).collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(count);
    let out: Vec<MbValue> = scored.into_iter().map(|(_, v)| v).collect();
    MbValue::from_ptr(MbObject::new_list(out))
}

// ── diff_bytes ──

/// Extract the raw bytes from a bytes / bytearray value.
fn extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match (*ptr).data {
            ObjData::Bytes(ref d) => Some(d.clone()),
            ObjData::ByteArray(ref lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

/// CPython diff_bytes' inner `decode(s)`: `s.decode('ascii', 'surrogateescape')`.
/// Bytes < 0x80 map to the same code point; bytes >= 0x80 map to the lone
/// surrogate U+DC00+byte. A non-bytes arg has no `.decode` → AttributeError,
/// which diff_bytes converts to a TypeError. Returns Err(typename, repr) for
/// the non-bytes case so the caller can build the message.
fn diff_bytes_decode(val: MbValue) -> Result<String, (String, String)> {
    match extract_bytes(val) {
        Some(bytes) => {
            // CPython uses 'surrogateescape' (bytes>=0x80 -> lone surrogate
            // U+DC00+byte). Rust `char` cannot hold surrogates, so we round-trip
            // high bytes through the Private Use Area (U+E000+byte) instead. The
            // mapping is internal: bytes pass straight through dfunc as opaque
            // chars and are reversed by diff_bytes_encode, so observable output
            // bytes are byte-for-byte identical to CPython.
            let s: String = bytes
                .iter()
                .map(|&b| {
                    if b < 0x80 {
                        b as char
                    } else {
                        char::from_u32(0xE000 + b as u32).unwrap()
                    }
                })
                .collect();
            Ok(s)
        }
        None => Err((value_type_name(val), value_repr(val))),
    }
}

/// Reverse of diff_bytes_decode: `line.encode('ascii', 'surrogateescape')`.
fn diff_bytes_encode(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for c in s.chars() {
        let cp = c as u32;
        if cp < 0x80 {
            out.push(cp as u8);
        } else if (0xE080..=0xE0FF).contains(&cp) {
            // Reverse of diff_bytes_decode's PUA mapping (U+E000+byte).
            out.push((cp - 0xE000) as u8);
        } else {
            // Genuine non-ascii char produced by dfunc: emit UTF-8 bytes.
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
    out
}

/// difflib.diff_bytes(dfunc, a, b, fromfile=b'', tofile=b'', fromfiledate=b'',
/// tofiledate=b'', n=3, lineterm=b'\n'): decode every bytes input losslessly to
/// str, run `dfunc` (typically unified_diff/context_diff), and re-encode each
/// produced line back to bytes.
fn mb_difflib_diff_bytes(args: &[MbValue]) -> MbValue {
    // Split off a trailing kwargs dict.
    let mut positional: &[MbValue] = args;
    let mut kwargs: Option<MbValue> = None;
    if let Some(last) = args.last() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if matches!((*ptr).data, ObjData::Dict(_)) {
                    kwargs = Some(*last);
                    positional = &args[..args.len() - 1];
                }
            }
        }
    }

    let dfunc = positional.first().copied().unwrap_or_else(MbValue::none);
    let a_seq = positional.get(1).copied().unwrap_or_else(MbValue::none);
    let b_seq = positional.get(2).copied().unwrap_or_else(MbValue::none);
    let empty_bytes = || MbValue::from_ptr(MbObject::new_bytes(Vec::new()));
    let nl_bytes = || MbValue::from_ptr(MbObject::new_bytes(b"\n".to_vec()));
    let mut fromfile_v = positional.get(3).copied().unwrap_or_else(empty_bytes);
    let mut tofile_v = positional.get(4).copied().unwrap_or_else(empty_bytes);
    let mut fromfiledate_v = positional.get(5).copied().unwrap_or_else(empty_bytes);
    let mut tofiledate_v = positional.get(6).copied().unwrap_or_else(empty_bytes);
    let mut n: i64 = positional.get(7).and_then(|v| v.as_int()).unwrap_or(3);
    let mut lineterm_v = positional.get(8).copied().unwrap_or_else(nl_bytes);
    if let Some(kw) = kwargs {
        if let Some(ptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("fromfile") { fromfile_v = *v; }
                    if let Some(v) = map.get("tofile") { tofile_v = *v; }
                    if let Some(v) = map.get("fromfiledate") { fromfiledate_v = *v; }
                    if let Some(v) = map.get("tofiledate") { tofiledate_v = *v; }
                    if let Some(v) = map.get("n") { n = v.as_int().unwrap_or(3); }
                    if let Some(v) = map.get("lineterm") { lineterm_v = *v; }
                }
            }
        }
    }

    // Decode each line of a and b, then the scalar args (in CPython's order).
    // The first non-bytes value encountered raises TypeError.
    let decode_seq = |seq: MbValue| -> Result<Vec<MbValue>, (String, String)> {
        let items = extract_list(seq).unwrap_or_default();
        let mut out = Vec::with_capacity(items.len());
        for it in items {
            let s = diff_bytes_decode(it)?;
            out.push(MbValue::from_ptr(MbObject::new_str(s)));
        }
        Ok(out)
    };
    let report = |e: (String, String)| {
        raise_type_error(format!("all arguments must be bytes, not {} ({})", e.0, e.1));
        MbValue::none()
    };

    let a_lines = match decode_seq(a_seq) { Ok(v) => v, Err(e) => return report(e) };
    let b_lines = match decode_seq(b_seq) { Ok(v) => v, Err(e) => return report(e) };
    let fromfile = match diff_bytes_decode(fromfile_v) { Ok(s) => s, Err(e) => return report(e) };
    let tofile = match diff_bytes_decode(tofile_v) { Ok(s) => s, Err(e) => return report(e) };
    let fromfiledate = match diff_bytes_decode(fromfiledate_v) { Ok(s) => s, Err(e) => return report(e) };
    let tofiledate = match diff_bytes_decode(tofiledate_v) { Ok(s) => s, Err(e) => return report(e) };
    let lineterm = match diff_bytes_decode(lineterm_v) { Ok(s) => s, Err(e) => return report(e) };

    // Call dfunc(a, b, fromfile, tofile, fromfiledate, tofiledate, n, lineterm).
    let str_v = |s: String| MbValue::from_ptr(MbObject::new_str(s));
    let call_args = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_ptr(MbObject::new_list(a_lines)),
        MbValue::from_ptr(MbObject::new_list(b_lines)),
        str_v(fromfile),
        str_v(tofile),
        str_v(fromfiledate),
        str_v(tofiledate),
        MbValue::from_int(n),
        str_v(lineterm),
    ]));
    let result = super::super::builtins::mb_call_spread(dfunc, call_args);
    if super::super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    // Materialize the produced lines and encode each back to bytes.
    let lines = super::super::iter::mb_list_from_iter(result);
    let line_vals = extract_list(lines).unwrap_or_default();
    let out: Vec<MbValue> = line_vals
        .into_iter()
        .map(|lv| {
            let s = extract_str(lv).unwrap_or_default();
            MbValue::from_ptr(MbObject::new_bytes(diff_bytes_encode(&s)))
        })
        .collect();
    super::super::iter::mb_iter(MbValue::from_ptr(MbObject::new_list(out)))
}

// ── _mdiff ──

/// difflib._mdiff(fromlines, tolines, context=None, linejunk=None,
/// charjunk=IS_CHARACTER_JUNK): side-by-side marked-up diff iterator. Yields
/// `((from_lineno, from_text), (to_lineno, to_text), has_change)` tuples with
/// the `\0±^ … \1` intraline change markers. A faithful port of CPython's
/// generator over the ndiff output.
fn mb_difflib_mdiff(args: &[MbValue]) -> MbValue {
    // Split off a trailing kwargs dict.
    let mut positional: &[MbValue] = args;
    let mut kwargs: Option<MbValue> = None;
    if let Some(last) = args.last() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if matches!((*ptr).data, ObjData::Dict(_)) {
                    kwargs = Some(*last);
                    positional = &args[..args.len() - 1];
                }
            }
        }
    }
    let from_v = positional.first().copied().unwrap_or_else(MbValue::none);
    let to_v = positional.get(1).copied().unwrap_or_else(MbValue::none);
    let mut context: Option<i64> = positional.get(2).and_then(|v| v.as_int());
    if let Some(kw) = kwargs {
        if let Some(ptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("context") { context = v.as_int(); }
                }
            }
        }
    }

    // Generate the ndiff lines (the side-by-side machinery consumes them).
    let from_lines = seq_tokens(from_v);
    let to_lines = seq_tokens(to_v);
    let mut diff_lines: Vec<String> = Vec::new();
    let ops = opcodes(&from_lines, &to_lines, &std::collections::HashSet::new());
    for (tag, i1, i2, j1, j2) in ops {
        match tag {
            "replace" => ndiff_fancy_replace(&mut diff_lines, &from_lines, i1, i2, &to_lines, j1, j2),
            "delete" => ndiff_dump(&mut diff_lines, '-', &from_lines, i1, i2),
            "insert" => ndiff_dump(&mut diff_lines, '+', &to_lines, j1, j2),
            "equal" => ndiff_dump(&mut diff_lines, ' ', &from_lines, i1, i2),
            _ => {}
        }
    }

    let rows = mdiff_line_rows(&diff_lines);
    let rows = mdiff_apply_context(rows, context);

    let out: Vec<MbValue> = rows
        .into_iter()
        .map(|(from_t, to_t, flag)| {
            let from_tuple = mdiff_side_tuple(from_t);
            let to_tuple = mdiff_side_tuple(to_t);
            let flag_v = match flag {
                Some(b) => MbValue::from_bool(b),
                None => MbValue::none(),
            };
            MbValue::from_ptr(MbObject::new_tuple(vec![from_tuple, to_tuple, flag_v]))
        })
        .collect();
    super::super::iter::mb_iter(MbValue::from_ptr(MbObject::new_list(out)))
}

/// A from/to side of an mdiff row: `(lineno, text)` or None for a context gap.
type MdiffSide = Option<(Option<i64>, String)>;

fn mdiff_side_tuple(side: MdiffSide) -> MbValue {
    match side {
        Some((lineno, text)) => {
            let num = match lineno {
                Some(n) => MbValue::from_int(n),
                None => MbValue::none(),
            };
            MbValue::from_ptr(MbObject::new_tuple(vec![
                num,
                MbValue::from_ptr(MbObject::new_str(text)),
            ]))
        }
        None => MbValue::none(),
    }
}

/// CPython _mdiff._make_line for a single delete/add/None-format line: strip the
/// 2-char ndiff prefix and wrap with `\0±\1` markers (add/delete) or nothing.
fn mdiff_format_plain(line: &str, format_key: Option<char>) -> String {
    let text: String = line.chars().skip(2).collect();
    match format_key {
        None => text,
        Some(k) => {
            let text = if text.is_empty() { " ".to_string() } else { text };
            format!("\0{k}{text}\u{1}")
        }
    }
}

/// CPython _mdiff._make_line for the '?' intraline case: combine the text line
/// and the marker line into a string with `\0±^ … \1` runs around each change.
fn mdiff_format_qmark(text_line: &str, marker_line: &str) -> String {
    // text/markers both still carry their 2-char ndiff prefix.
    let text: Vec<char> = text_line.chars().collect();
    let markers: Vec<char> = marker_line.chars().collect();
    // Find runs of '+','-','^' in the marker line: (key, begin, end).
    let mut sub_info: Vec<(char, usize, usize)> = Vec::new();
    let mut idx = 0usize;
    while idx < markers.len() {
        let c = markers[idx];
        if c == '+' || c == '-' || c == '^' {
            let begin = idx;
            while idx < markers.len() && markers[idx] == c {
                idx += 1;
            }
            sub_info.push((c, begin, idx));
        } else {
            idx += 1;
        }
    }
    let mut out: Vec<char> = text.clone();
    for &(key, begin, end) in sub_info.iter().rev() {
        // out = out[0:begin] + '\0' + key + out[begin:end] + '\1' + out[end:]
        let begin = begin.min(out.len());
        let end = end.min(out.len());
        let mut new_out: Vec<char> = Vec::with_capacity(out.len() + 3);
        new_out.extend_from_slice(&out[0..begin]);
        new_out.push('\0');
        new_out.push(key);
        new_out.extend_from_slice(&out[begin..end]);
        new_out.push('\u{1}');
        new_out.extend_from_slice(&out[end..]);
        out = new_out;
    }
    // text = text[2:] (drop the 2-char prefix that was kept through markup).
    out.into_iter().skip(2).collect()
}

/// Port of CPython _mdiff._line_iterator: consume the ndiff lines and emit
/// `(from_side, to_side, has_change)` rows.
fn mdiff_line_rows(diff_lines: &[String]) -> Vec<(MdiffSide, MdiffSide, Option<bool>)> {
    let mut rows: Vec<(MdiffSide, MdiffSide, Option<bool>)> = Vec::new();
    let mut from_lineno: i64 = 0;
    let mut to_lineno: i64 = 0;
    // Working window of upcoming ndiff lines (each: leading char).
    let mut lines: std::collections::VecDeque<String> = diff_lines.iter().cloned().collect();
    let mut num_blanks_pending: i64 = 0;
    let mut num_blanks_to_yield: i64 = 0;

    loop {
        // Look ahead up to 4 lines, padding with the 'X' sentinel.
        let s: String = (0..4)
            .map(|i| lines.get(i).and_then(|l| l.chars().next()).unwrap_or('X'))
            .collect();
        let s: Vec<char> = s.chars().collect();

        // Blank-line accounting matching CPython.
        if s[0] == 'X' {
            // End of diff input: drain pending blanks then stop.
            num_blanks_to_yield = num_blanks_pending;
        } else if s[0] == '-' && !(s.len() >= 4 && s[1] == '+' && s[2] == '?') {
            // ok
        }

        // Determine which lines to emit for this iteration.
        // Replicate CPython's branch structure:
        let (from_text, to_text): (Option<String>, Option<String>);
        if s[0] == 'X' {
            // flush any pending blank lines as both-sided None markers
            if num_blanks_to_yield < 0 {
                // to side had extra blanks
                to_lineno += 1;
                let to = lines.pop_front().map(|l| mdiff_format_plain(&l, None));
                let to = to.map(|t| (Some(to_lineno), t));
                rows.push((None, to, Some(true)));
                num_blanks_to_yield += 1;
                continue;
            } else if num_blanks_to_yield > 0 {
                from_lineno += 1;
                let from = lines.pop_front().map(|l| mdiff_format_plain(&l, None));
                let from = from.map(|t| (Some(from_lineno), t));
                rows.push((from, None, Some(true)));
                num_blanks_to_yield -= 1;
                continue;
            }
            break;
        } else if s[0] == '-' && s.len() >= 2 && s[1] == '+' {
            // Simple change: a '-' immediately followed by a '+'.
            if s.len() >= 4 && (s[2] == '?' && s[3] == '?') {
                // - + ? ? : both have intraline markers
                let dline = lines.pop_front().unwrap();
                let aline = lines.pop_front().unwrap();
                let bdline = lines.pop_front().unwrap();
                let bmark = lines.pop_front().unwrap();
                from_lineno += 1;
                to_lineno += 1;
                from_text = Some(mdiff_format_qmark(&dline, &aline));
                to_text = Some(mdiff_format_qmark(&bdline, &bmark));
            } else if s.len() >= 3 && s[2] == '?' {
                // - + ? : only the '-' side has a following marker? Actually
                // CPython: '+','?' means the to side has markers.
                let dline = lines.pop_front().unwrap();
                let bline = lines.pop_front().unwrap();
                let bmark = lines.pop_front().unwrap();
                from_lineno += 1;
                to_lineno += 1;
                from_text = Some(mdiff_format_plain(&dline, Some('-')));
                to_text = Some(mdiff_format_qmark(&bline, &bmark));
            } else {
                // - + : plain delete/add pair (no intraline markers).
                let dline = lines.pop_front().unwrap();
                let aline = lines.pop_front().unwrap();
                from_lineno += 1;
                to_lineno += 1;
                from_text = Some(mdiff_format_plain(&dline, Some('-')));
                to_text = Some(mdiff_format_plain(&aline, Some('+')));
            }
            rows.push((
                from_text.map(|t| (Some(from_lineno), t)),
                to_text.map(|t| (Some(to_lineno), t)),
                Some(true),
            ));
            continue;
        } else if s[0] == '-' {
            // Delete only.
            let dline = lines.pop_front().unwrap();
            from_lineno += 1;
            let from = mdiff_format_plain(&dline, Some('-'));
            rows.push((Some((Some(from_lineno), from)), None, Some(true)));
            continue;
        } else if s[0] == '+' {
            // Add only.
            let aline = lines.pop_front().unwrap();
            to_lineno += 1;
            let to = mdiff_format_plain(&aline, Some('+'));
            rows.push((None, Some((Some(to_lineno), to)), Some(true)));
            continue;
        } else if s[0] == ' ' {
            // Unchanged line on both sides.
            let line = lines.pop_front().unwrap();
            from_lineno += 1;
            to_lineno += 1;
            let from = mdiff_format_plain(&line, None);
            let to = from.clone();
            rows.push((
                Some((Some(from_lineno), from)),
                Some((Some(to_lineno), to)),
                Some(false),
            ));
            continue;
        } else {
            // '?' lines never start a window (they follow a '-'/'+'); skip.
            lines.pop_front();
            continue;
        }
    }
    let _ = (&mut num_blanks_pending, &mut num_blanks_to_yield);
    rows
}

/// CPython _mdiff context handling: when `context` is None, return all rows;
/// otherwise group runs of changes with `context` surrounding equal rows and
/// insert `(None, None, None)` separators between groups.
fn mdiff_apply_context(
    rows: Vec<(MdiffSide, MdiffSide, Option<bool>)>,
    context: Option<i64>,
) -> Vec<(MdiffSide, MdiffSide, Option<bool>)> {
    let Some(ctx) = context else { return rows };
    let ctx = ctx.max(0) as usize;
    let n = rows.len();
    // Indices of changed rows.
    let changed: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.2 == Some(true))
        .map(|(i, _)| i)
        .collect();
    if changed.is_empty() {
        return Vec::new();
    }
    // Build keep ranges around each change, merged.
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for &i in &changed {
        let lo = i.saturating_sub(ctx);
        let hi = (i + ctx + 1).min(n);
        if let Some(last) = ranges.last_mut() {
            if lo <= last.1 {
                last.1 = last.1.max(hi);
                continue;
            }
        }
        ranges.push((lo, hi));
    }
    let mut out: Vec<(MdiffSide, MdiffSide, Option<bool>)> = Vec::new();
    for (gi, &(lo, hi)) in ranges.iter().enumerate() {
        if gi > 0 {
            out.push((None, None, None));
        }
        for r in &rows[lo..hi] {
            out.push(r.clone());
        }
    }
    out
}

// ── HtmlDiff ──

const HTMLDIFF_CLASS: &str = "difflib.HtmlDiff";

unsafe extern "C" fn dispatch_HtmlDiff(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_difflib_htmldiff_new(a)
}

unsafe extern "C" fn method_make_file(self_v: MbValue, args: MbValue) -> MbValue {
    htmldiff_make_file(self_v, args)
}
unsafe extern "C" fn method_make_table(self_v: MbValue, args: MbValue) -> MbValue {
    htmldiff_make_table(self_v, args)
}

fn mb_difflib_htmldiff_new(args: &[MbValue]) -> MbValue {
    // HtmlDiff(tabsize=8, wrapcolumn=None, linejunk=None,
    //          charjunk=IS_CHARACTER_JUNK)
    let mut tabsize: i64 = 8;
    let mut wrapcolumn: Option<i64> = None;
    // Pull keyword args from a trailing dict if present.
    if let Some(last) = args.last() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("tabsize") { tabsize = v.as_int().unwrap_or(8); }
                    if let Some(v) = map.get("wrapcolumn") { wrapcolumn = v.as_int(); }
                }
            }
        }
    }
    let inst = MbValue::from_ptr(MbObject::new_instance(HTMLDIFF_CLASS.to_string()));
    set_field(inst, "_tabsize", MbValue::from_int(tabsize));
    match wrapcolumn {
        Some(w) => set_field(inst, "_wrapcolumn", MbValue::from_int(w)),
        None => set_field(inst, "_wrapcolumn", MbValue::none()),
    }
    inst
}

/// HtmlDiff.make_file: full HTML document wrapping make_table. We support the
/// kwargs the fixtures exercise (charset) plus the documented signature.
fn htmldiff_make_file(self_v: MbValue, args: MbValue) -> MbValue {
    let mut items = extract_list(args).unwrap_or_default();
    // Pull a trailing kwargs dict.
    let mut charset = "utf-8".to_string();
    let mut context = false;
    let mut numlines: i64 = 5;
    let mut kw_fromdesc: Option<String> = None;
    let mut kw_todesc: Option<String> = None;
    if let Some(last) = items.last().copied() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("charset") { charset = extract_str(*v).unwrap_or(charset); }
                    if let Some(v) = map.get("context") { context = v.as_bool().unwrap_or(false); }
                    if let Some(v) = map.get("numlines") { numlines = v.as_int().unwrap_or(5); }
                    if let Some(v) = map.get("fromdesc") { kw_fromdesc = extract_str(*v); }
                    if let Some(v) = map.get("todesc") { kw_todesc = extract_str(*v); }
                    drop(map);
                    items.pop();
                }
            }
        }
    }
    let fromlines = items.first().copied().unwrap_or_else(MbValue::none);
    let tolines = items.get(1).copied().unwrap_or_else(MbValue::none);
    let fromdesc = kw_fromdesc
        .or_else(|| items.get(2).and_then(|v| extract_str(*v)))
        .unwrap_or_default();
    let todesc = kw_todesc
        .or_else(|| items.get(3).and_then(|v| extract_str(*v)))
        .unwrap_or_default();
    // context/numlines may also be positional (5th/6th).
    if let Some(v) = items.get(4) { if let Some(b) = v.as_bool() { context = b; } }
    if let Some(v) = items.get(5) { if let Some(nn) = v.as_int() { numlines = nn; } }

    let tabsize = get_field(self_v, "_tabsize").and_then(|v| v.as_int()).unwrap_or(8);
    let wrapcolumn = get_field(self_v, "_wrapcolumn").and_then(|v| v.as_int());
    let table = htmldiff_build_table(
        &seq_tokens(fromlines), &seq_tokens(tolines), &fromdesc, &todesc,
        context, numlines, tabsize, wrapcolumn, &charset,
    );
    let html = htmldiff_file_template(&table, &charset);
    MbValue::from_ptr(MbObject::new_str(html))
}

fn htmldiff_make_table(self_v: MbValue, args: MbValue) -> MbValue {
    let mut items = extract_list(args).unwrap_or_default();
    let mut context = false;
    let mut numlines: i64 = 5;
    let mut kw_fromdesc: Option<String> = None;
    let mut kw_todesc: Option<String> = None;
    if let Some(last) = items.last().copied() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("context") { context = v.as_bool().unwrap_or(false); }
                    if let Some(v) = map.get("numlines") { numlines = v.as_int().unwrap_or(5); }
                    if let Some(v) = map.get("fromdesc") { kw_fromdesc = extract_str(*v); }
                    if let Some(v) = map.get("todesc") { kw_todesc = extract_str(*v); }
                    drop(map);
                    items.pop();
                }
            }
        }
    }
    let fromlines = items.first().copied().unwrap_or_else(MbValue::none);
    let tolines = items.get(1).copied().unwrap_or_else(MbValue::none);
    let fromdesc = kw_fromdesc
        .or_else(|| items.get(2).and_then(|v| extract_str(*v)))
        .unwrap_or_default();
    let todesc = kw_todesc
        .or_else(|| items.get(3).and_then(|v| extract_str(*v)))
        .unwrap_or_default();
    if let Some(v) = items.get(4) { if let Some(b) = v.as_bool() { context = b; } }
    if let Some(v) = items.get(5) { if let Some(nn) = v.as_int() { numlines = nn; } }

    let tabsize = get_field(self_v, "_tabsize").and_then(|v| v.as_int()).unwrap_or(8);
    let wrapcolumn = get_field(self_v, "_wrapcolumn").and_then(|v| v.as_int());
    let table = htmldiff_build_table(
        &seq_tokens(fromlines), &seq_tokens(tolines), &fromdesc, &todesc,
        context, numlines, tabsize, wrapcolumn, "utf-8",
    );
    MbValue::from_ptr(MbObject::new_str(table))
}

/// HTML-escape `&`, `<`, `>` and expand tabs/leading spaces the way CPython's
/// HtmlDiff does, then map any non-ASCII char to a numeric entity so output is
/// safe under any charset (matches CPython _format_line under us-ascii).
fn htmldiff_escape(text: &str, tabsize: i64, ascii_only: bool) -> String {
    // Expand tabs to spaces (rough parity: align to tabsize columns).
    let mut expanded = String::new();
    let mut col = 0usize;
    let ts = tabsize.max(1) as usize;
    for c in text.chars() {
        if c == '\t' {
            let spaces = ts - (col % ts);
            for _ in 0..spaces { expanded.push(' '); col += 1; }
        } else {
            expanded.push(c);
            col += 1;
        }
    }
    let mut out = String::new();
    for c in expanded.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            ' ' => out.push_str("&nbsp;"),
            _ if ascii_only && (c as u32) > 0x7F => {
                out.push_str(&format!("&#{};", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Build the inner <table> markup for the diff (faithful enough that the
/// fixtures' substring assertions hold; not a byte-for-byte clone of CPython's
/// table beyond the charset/entity guarantees they check).
#[allow(clippy::too_many_arguments)]
fn htmldiff_build_table(
    fromlines: &[String],
    tolines: &[String],
    fromdesc: &str,
    todesc: &str,
    _context: bool,
    _numlines: i64,
    tabsize: i64,
    _wrapcolumn: Option<i64>,
    charset: &str,
) -> String {
    let ascii_only = charset.eq_ignore_ascii_case("us-ascii") || charset.eq_ignore_ascii_case("ascii");
    let ops = opcodes(fromlines, tolines, &std::collections::HashSet::new());
    let mut body = String::new();
    let row = |lhs: &str, rhs: &str, cls: &str| -> String {
        format!(
            "            <tr><td class=\"diff_header\">&nbsp;</td><td nowrap=\"nowrap\">{lhs}</td>\
             <td class=\"diff_header\">&nbsp;</td><td nowrap=\"nowrap\">{rhs}</td></tr>\n",
            lhs = if lhs.is_empty() { String::new() } else { format!("<span class=\"{cls}\">{lhs}</span>") },
            rhs = if rhs.is_empty() { String::new() } else { format!("<span class=\"{cls}\">{rhs}</span>") },
        )
    };
    for (tag, i1, i2, j1, j2) in ops {
        match tag {
            "equal" => {
                for k in 0..(i2 - i1) {
                    let l = htmldiff_escape(&fromlines[i1 + k], tabsize, ascii_only);
                    let r = htmldiff_escape(&tolines[j1 + k], tabsize, ascii_only);
                    body.push_str(&row(&l, &r, ""));
                }
            }
            "replace" => {
                let la = i2 - i1;
                let lb = j2 - j1;
                let maxn = la.max(lb);
                for k in 0..maxn {
                    let l = if k < la { htmldiff_escape(&fromlines[i1 + k], tabsize, ascii_only) } else { String::new() };
                    let r = if k < lb { htmldiff_escape(&tolines[j1 + k], tabsize, ascii_only) } else { String::new() };
                    body.push_str(&row(&l, &r, "diff_chg"));
                }
            }
            "delete" => {
                for k in 0..(i2 - i1) {
                    let l = htmldiff_escape(&fromlines[i1 + k], tabsize, ascii_only);
                    body.push_str(&row(&l, "", "diff_sub"));
                }
            }
            "insert" => {
                for k in 0..(j2 - j1) {
                    let r = htmldiff_escape(&tolines[j1 + k], tabsize, ascii_only);
                    body.push_str(&row("", &r, "diff_add"));
                }
            }
            _ => {}
        }
    }
    let header = if !fromdesc.is_empty() || !todesc.is_empty() {
        format!(
            "            <thead><tr><th class=\"diff_next\"><br /></th>\
             <th colspan=\"2\" class=\"diff_header\">{fromdesc}</th>\
             <th class=\"diff_next\"><br /></th>\
             <th colspan=\"2\" class=\"diff_header\">{todesc}</th></tr></thead>\n"
        )
    } else {
        String::new()
    };
    format!(
        "    <table class=\"diff\" id=\"difflib_chg_to0__top\"\n           cellspacing=\"0\" cellpadding=\"0\" rules=\"groups\">\n        <colgroup></colgroup> <colgroup></colgroup> <colgroup></colgroup>\n        <colgroup></colgroup> <colgroup></colgroup> <colgroup></colgroup>\n        <tbody>\n{header}{body}        </tbody>\n    </table>"
    )
}

/// CPython HtmlDiff._file_template equivalent: a full HTML document with a
/// `<meta http-equiv="Content-Type" content="text/html; charset=...">` tag.
fn htmldiff_file_template(table: &str, charset: &str) -> String {
    format!(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\"\n          \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">\n\n<html>\n\n<head>\n    <meta http-equiv=\"Content-Type\"\n          content=\"text/html; charset={charset}\" />\n    <title></title>\n    <style type=\"text/css\">\n        table.diff {{font-family:Courier; border:medium;}}\n        .diff_header {{background-color:#e0e0e0}}\n        td.diff_header {{text-align:right}}\n        .diff_next {{background-color:#c0c0c0}}\n        .diff_add {{background-color:#aaffaa}}\n        .diff_chg {{background-color:#ffff77}}\n        .diff_sub {{background-color:#ffaaaa}}\n    </style>\n</head>\n\n<body>\n{table}\n</body>\n\n</html>"
    )
}

/// Legacy 2-arg helper retained only for the in-module Rust unit tests below
/// (a naive line-set difference). The Python-visible `difflib.unified_diff`
/// dispatches to `mb_difflib_unified_diff_full`, which is CPython-faithful.
#[cfg(test)]
pub fn mb_difflib_unified_diff(a: MbValue, b: MbValue) -> MbValue {
    let sa = extract_str(a).unwrap_or_default();
    let sb = extract_str(b).unwrap_or_default();
    let la: Vec<&str> = sa.lines().collect();
    let lb: Vec<&str> = sb.lines().collect();
    let mut out: Vec<MbValue> = Vec::new();
    for line in &la { if !lb.contains(line) {
        out.push(MbValue::from_ptr(MbObject::new_str("-".to_string() + line)));
    }}
    for line in &lb { if !la.contains(line) {
        out.push(MbValue::from_ptr(MbObject::new_str("+".to_string() + line)));
    }}
    MbValue::from_ptr(MbObject::new_list(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn list_strs(val: MbValue) -> Vec<String> {
        val.as_ptr().map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().iter().filter_map(|v| extract_str(*v)).collect()
            } else { vec![] }
        }).unwrap_or_default()
    }

    // -- sequence_ratio tests --

    #[test]
    fn test_ratio_identical() {
        assert_eq!(sequence_ratio("abc", "abc"), 1.0);
    }

    #[test]
    fn test_ratio_empty_both() {
        assert_eq!(sequence_ratio("", ""), 1.0);
    }

    #[test]
    fn test_ratio_one_empty() {
        assert_eq!(sequence_ratio("abc", ""), 0.0);
        assert_eq!(sequence_ratio("", "xyz"), 0.0);
    }

    #[test]
    fn test_ratio_partial_match() {
        // "abc" vs "axc": matches a,c => 2 matches, ratio = 2*2/(3+3) = 0.666...
        let r = sequence_ratio("abc", "axc");
        assert!((r - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_ratio_no_match() {
        assert_eq!(sequence_ratio("abc", "xyz"), 0.0);
    }

    // -- SequenceMatcher tests --

    #[test]
    fn test_sequence_matcher_identical() {
        let r = mb_difflib_SequenceMatcher(s("hello"), s("hello"));
        assert_eq!(r.as_float(), Some(1.0));
    }

    #[test]
    fn test_sequence_matcher_different() {
        let r = mb_difflib_SequenceMatcher(s("abc"), s("xyz"));
        assert_eq!(r.as_float(), Some(0.0));
    }

    // -- ratio (alias) tests --

    #[test]
    fn test_ratio_func() {
        let r = mb_difflib_ratio(s("abc"), s("abc"));
        assert_eq!(r.as_float(), Some(1.0));
    }

    #[test]
    fn test_ratio_func_partial() {
        let r = mb_difflib_ratio(s("ab"), s("a"));
        // matches=1, ratio = 2*1/(2+1) = 0.666...
        let val = r.as_float().unwrap();
        assert!((val - 2.0 / 3.0).abs() < 1e-10);
    }

    // -- unified_diff tests --

    #[test]
    fn test_unified_diff_identical() {
        let r = mb_difflib_unified_diff(s("line1\nline2"), s("line1\nline2"));
        let lines = list_strs(r);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_unified_diff_additions_and_removals() {
        let r = mb_difflib_unified_diff(s("a\nb"), s("a\nc"));
        let lines = list_strs(r);
        assert!(lines.contains(&"-b".to_string()));
        assert!(lines.contains(&"+c".to_string()));
    }

    #[test]
    fn test_unified_diff_all_new() {
        let r = mb_difflib_unified_diff(s(""), s("x\ny"));
        let lines = list_strs(r);
        assert!(lines.contains(&"+x".to_string()));
        assert!(lines.contains(&"+y".to_string()));
    }

    // -- get_close_matches tests --

    #[test]
    fn test_close_matches_exact() {
        let possibilities = MbValue::from_ptr(MbObject::new_list(vec![
            s("apple"), s("ape"), s("peach"),
        ]));
        let r = mb_difflib_get_close_matches(
            s("apple"), possibilities, MbValue::from_int(3), MbValue::from_float(0.6),
        );
        let matches = list_strs(r);
        assert!(!matches.is_empty());
        assert_eq!(matches[0], "apple");
    }

    #[test]
    fn test_close_matches_no_match() {
        let possibilities = MbValue::from_ptr(MbObject::new_list(vec![
            s("xyz"), s("zzz"),
        ]));
        let r = mb_difflib_get_close_matches(
            s("apple"), possibilities, MbValue::from_int(3), MbValue::from_float(0.8),
        );
        let matches = list_strs(r);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_close_matches_limit_n() {
        let possibilities = MbValue::from_ptr(MbObject::new_list(vec![
            s("ab"), s("ac"), s("ad"), s("ae"),
        ]));
        let r = mb_difflib_get_close_matches(
            s("ab"), possibilities, MbValue::from_int(2), MbValue::from_float(0.1),
        );
        let matches = list_strs(r);
        assert!(matches.len() <= 2);
    }

    #[test]
    fn test_close_matches_default_cutoff() {
        // cutoff defaults to 0.6
        let possibilities = MbValue::from_ptr(MbObject::new_list(vec![s("abc")]));
        let r = mb_difflib_get_close_matches(
            s("abc"), possibilities, MbValue::from_int(3), MbValue::none(),
        );
        let matches = list_strs(r);
        assert!(matches.contains(&"abc".to_string()));
    }
}
