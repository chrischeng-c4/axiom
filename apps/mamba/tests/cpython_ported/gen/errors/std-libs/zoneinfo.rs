use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/zoneinfo/from_file_missing_raises.py`.
#[test]
fn test_gen_errors_std_libs_zoneinfo_from_file_missing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "errors"
# case = "from_file_missing_raises"
# subject = "zoneinfo.ZoneInfo.from_file"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo.from_file: from_file_missing_raises (errors)."""
import zoneinfo

_raised = False
try:
    zoneinfo.ZoneInfo.from_file(open("/no/such/tz/file", "rb"), key="X")
except FileNotFoundError:
    _raised = True
assert _raised, "from_file_missing_raises: expected FileNotFoundError"
print("from_file_missing_raises OK")
"###);
    assert_output(&out, r###"from_file_missing_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zoneinfo/not_found_caught_as_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_zoneinfo_not_found_caught_as_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "errors"
# case = "not_found_caught_as_keyerror"
# subject = "zoneinfo.ZoneInfoNotFoundError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfoNotFoundError: not_found_caught_as_keyerror (errors)."""
import zoneinfo

_raised = False
try:
    zoneinfo.ZoneInfo("No/Such/Timezone")
except KeyError:
    _raised = True
assert _raised, "not_found_caught_as_keyerror: expected KeyError"
print("not_found_caught_as_keyerror OK")
"###);
    assert_output(&out, r###"not_found_caught_as_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zoneinfo/path_traversal_key_raises_value_error.py`.
#[test]
fn test_gen_errors_std_libs_zoneinfo_path_traversal_key_raises_value_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "errors"
# case = "path_traversal_key_raises_value_error"
# subject = "zoneinfo.ZoneInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: path_traversal_key_raises_value_error (errors)."""
import zoneinfo

_raised = False
try:
    zoneinfo.ZoneInfo("../etc/passwd")
except ValueError:
    _raised = True
assert _raised, "path_traversal_key_raises_value_error: expected ValueError"
print("path_traversal_key_raises_value_error OK")
"###);
    assert_output(&out, r###"path_traversal_key_raises_value_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/zoneinfo/unknown_zone_raises_not_found.py`.
#[test]
fn test_gen_errors_std_libs_zoneinfo_unknown_zone_raises_not_found() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "errors"
# case = "unknown_zone_raises_not_found"
# subject = "zoneinfo.ZoneInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
"""zoneinfo.ZoneInfo: unknown_zone_raises_not_found (errors)."""
import zoneinfo

_raised = False
try:
    zoneinfo.ZoneInfo("No/Such/Timezone")
except zoneinfo.ZoneInfoNotFoundError:
    _raised = True
assert _raised, "unknown_zone_raises_not_found: expected zoneinfo.ZoneInfoNotFoundError"
print("unknown_zone_raises_not_found OK")
"###);
    assert_output(&out, r###"unknown_zone_raises_not_found OK
"###);
}
