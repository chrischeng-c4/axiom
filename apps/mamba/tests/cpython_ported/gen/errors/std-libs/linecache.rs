use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/linecache/checkcache_missing_file_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_linecache_checkcache_missing_file_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "checkcache_missing_file_no_raise"
# subject = "linecache.checkcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.checkcache: checkcache on a missing/unknown filename does not raise and returns None"""
import linecache

linecache.clearcache()
assert linecache.checkcache("/no/such/file.py") is None, "checkcache missing returns None"
print("checkcache_missing_file_no_raise OK")
"###);
    assert_output(&out, r###"checkcache_missing_file_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/linecache/clearcache_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_linecache_clearcache_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "clearcache_no_raise"
# subject = "linecache.clearcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.clearcache: clearcache always succeeds and returns None"""
import linecache

assert linecache.clearcache() is None, "clearcache returns None"
print("clearcache_no_raise OK")
"###);
    assert_output(&out, r###"clearcache_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/linecache/getline_missing_file_returns_empty.py`.
#[test]
fn test_gen_errors_std_libs_linecache_getline_missing_file_returns_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "getline_missing_file_returns_empty"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline of a non-existent file returns '' without raising"""
import linecache

linecache.clearcache()
assert linecache.getline("/no/such/file.py", 1) == "", "missing file getline"
print("getline_missing_file_returns_empty OK")
"###);
    assert_output(&out, r###"getline_missing_file_returns_empty OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/linecache/getline_out_of_range_returns_empty.py`.
#[test]
fn test_gen_errors_std_libs_linecache_getline_out_of_range_returns_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "getline_out_of_range_returns_empty"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline with a line number past EOF returns '' without raising"""
import linecache

linecache.clearcache()
# This very file exists but has far fewer than 999999 lines.
assert linecache.getline(__file__, 999999) == "", "out-of-range getline"
print("getline_out_of_range_returns_empty OK")
"###);
    assert_output(&out, r###"getline_out_of_range_returns_empty OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/linecache/getlines_missing_file_returns_empty_list.py`.
#[test]
fn test_gen_errors_std_libs_linecache_getlines_missing_file_returns_empty_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "getlines_missing_file_returns_empty_list"
# subject = "linecache.getlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getlines: getlines of a non-existent file returns [] without raising"""
import linecache

linecache.clearcache()
assert linecache.getlines("/no/such/file.py") == [], "missing file getlines"
print("getlines_missing_file_returns_empty_list OK")
"###);
    assert_output(&out, r###"getlines_missing_file_returns_empty_list OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/linecache/updatecache_missing_file_returns_empty_list.py`.
#[test]
fn test_gen_errors_std_libs_linecache_updatecache_missing_file_returns_empty_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "updatecache_missing_file_returns_empty_list"
# subject = "linecache.updatecache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.updatecache: updatecache of a non-existent file returns [] without raising"""
import linecache

linecache.clearcache()
assert linecache.updatecache("/no/such/file.py") == [], "missing file updatecache"
print("updatecache_missing_file_returns_empty_list OK")
"###);
    assert_output(&out, r###"updatecache_missing_file_returns_empty_list OK
"###);
}
