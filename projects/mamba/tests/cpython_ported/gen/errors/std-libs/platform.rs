use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/platform/libc_ver_missing_executable_raises.py`.
#[test]
fn test_gen_errors_std_libs_platform_libc_ver_missing_executable_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "errors"
# case = "libc_ver_missing_executable_raises"
# subject = "platform.libc_ver"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.libc_ver: libc_ver_missing_executable_raises (errors)."""
import platform

_raised = False
try:
    platform.libc_ver("/no/such/exe")
except FileNotFoundError:
    _raised = True
assert _raised, "libc_ver_missing_executable_raises: expected FileNotFoundError"
print("libc_ver_missing_executable_raises OK")
"###);
    assert_output(&out, r###"libc_ver_missing_executable_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/platform/sys_version_malformed_banner_raises.py`.
#[test]
fn test_gen_errors_std_libs_platform_sys_version_malformed_banner_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "errors"
# case = "sys_version_malformed_banner_raises"
# subject = "platform._sys_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform._sys_version: sys_version_malformed_banner_raises (errors)."""
import platform

_raised = False
try:
    platform._sys_version("2. 4.3 (truncation) \\n[GCC]")
except ValueError:
    _raised = True
assert _raised, "sys_version_malformed_banner_raises: expected ValueError"
print("sys_version_malformed_banner_raises OK")
"###);
    assert_output(&out, r###"sys_version_malformed_banner_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/platform/uname_index_out_of_range_raises.py`.
#[test]
fn test_gen_errors_std_libs_platform_uname_index_out_of_range_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "errors"
# case = "uname_index_out_of_range_raises"
# subject = "platform.uname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform.uname: uname_index_out_of_range_raises (errors)."""
import platform

_raised = False
try:
    platform.uname()[6]
except IndexError:
    _raised = True
assert _raised, "uname_index_out_of_range_raises: expected IndexError"
print("uname_index_out_of_range_raises OK")
"###);
    assert_output(&out, r###"uname_index_out_of_range_raises OK
"###);
}
