use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// difflib module for Mamba (mamba-stdlib).
use std::collections::HashMap;

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

dispatch_binary!(dispatch_SequenceMatcher, mb_difflib_SequenceMatcher);
dispatch_binary!(dispatch_ratio, mb_difflib_ratio);
dispatch_binary!(dispatch_unified_diff, mb_difflib_unified_diff);
dispatch_quaternary!(dispatch_get_close_matches, mb_difflib_get_close_matches);

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("SequenceMatcher", dispatch_SequenceMatcher as usize),
        ("ratio", dispatch_ratio as usize),
        ("unified_diff", dispatch_unified_diff as usize),
        ("get_close_matches", dispatch_get_close_matches as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("difflib", attrs);
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

fn extract_list(val: MbValue) -> Option<Vec<MbValue>> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::List(ref lock) = (*ptr).data {
            Some(lock.read().unwrap().to_vec())
        } else {
            None
        }
    })
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
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let mut matches = 0usize;
    let mut used = vec![false; bc.len()];
    for ca in &ac {
        for (j, cb) in bc.iter().enumerate() {
            if !used[j] && ca == cb {
                matches += 1;
                used[j] = true;
                break;
            }
        }
    }
    2.0 * matches as f64 / (ac.len() + bc.len()) as f64
}

pub fn mb_difflib_unified_diff(a: MbValue, b: MbValue) -> MbValue {
    let sa = extract_str(a).unwrap_or_default();
    let sb = extract_str(b).unwrap_or_default();
    let la: Vec<&str> = sa.lines().collect();
    let lb: Vec<&str> = sb.lines().collect();
    let mut out: Vec<MbValue> = Vec::new();
    for line in &la {
        if !lb.contains(line) {
            out.push(MbValue::from_ptr(MbObject::new_str("-".to_string() + line)));
        }
    }
    for line in &lb {
        if !la.contains(line) {
            out.push(MbValue::from_ptr(MbObject::new_str("+".to_string() + line)));
        }
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

pub fn mb_difflib_get_close_matches(
    word: MbValue,
    possibilities: MbValue,
    n: MbValue,
    cutoff: MbValue,
) -> MbValue {
    let sw = extract_str(word).unwrap_or_default();
    let cut = cutoff.as_float().unwrap_or(0.6);
    let count = n.as_int().unwrap_or(3) as usize;
    let items = extract_list(possibilities).unwrap_or_default();
    let mut scored: Vec<(f64, MbValue)> = items
        .into_iter()
        .filter_map(|v| extract_str(v).map(|s| (sequence_ratio(&sw, &s), v)))
        .filter(|(r, _)| *r >= cut)
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(count);
    let out: Vec<MbValue> = scored.into_iter().map(|(_, v)| v).collect();
    MbValue::from_ptr(MbObject::new_list(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
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
        let possibilities =
            MbValue::from_ptr(MbObject::new_list(vec![s("apple"), s("ape"), s("peach")]));
        let r = mb_difflib_get_close_matches(
            s("apple"),
            possibilities,
            MbValue::from_int(3),
            MbValue::from_float(0.6),
        );
        let matches = list_strs(r);
        assert!(!matches.is_empty());
        assert_eq!(matches[0], "apple");
    }

    #[test]
    fn test_close_matches_no_match() {
        let possibilities = MbValue::from_ptr(MbObject::new_list(vec![s("xyz"), s("zzz")]));
        let r = mb_difflib_get_close_matches(
            s("apple"),
            possibilities,
            MbValue::from_int(3),
            MbValue::from_float(0.8),
        );
        let matches = list_strs(r);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_close_matches_limit_n() {
        let possibilities =
            MbValue::from_ptr(MbObject::new_list(vec![s("ab"), s("ac"), s("ad"), s("ae")]));
        let r = mb_difflib_get_close_matches(
            s("ab"),
            possibilities,
            MbValue::from_int(2),
            MbValue::from_float(0.1),
        );
        let matches = list_strs(r);
        assert!(matches.len() <= 2);
    }

    #[test]
    fn test_close_matches_default_cutoff() {
        // cutoff defaults to 0.6
        let possibilities = MbValue::from_ptr(MbObject::new_list(vec![s("abc")]));
        let r = mb_difflib_get_close_matches(
            s("abc"),
            possibilities,
            MbValue::from_int(3),
            MbValue::none(),
        );
        let matches = list_strs(r);
        assert!(matches.contains(&"abc".to_string()));
    }
}
