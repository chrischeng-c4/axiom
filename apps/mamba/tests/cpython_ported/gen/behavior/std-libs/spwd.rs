use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/spwd/test_spwd_non_root__test_getspnam_exception.py`.
#[test]
fn test_gen_behavior_std_libs_spwd_test_spwd_non_root__test_getspnam_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "spwd"
# dimension = "behavior"
# case = "test_spwd_non_root__test_getspnam_exception"
# subject = "cpython.test_spwd.TestSpwdNonRoot.test_getspnam_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_spwd.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_spwd.py::TestSpwdNonRoot::test_getspnam_exception
"""Auto-ported test: TestSpwdNonRoot::test_getspnam_exception."""


import os
import warnings


try:
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", DeprecationWarning)
        import spwd
except ImportError:
    print("TestSpwdNonRoot::test_getspnam_exception: skipped spwd unavailable")
else:
    if not (hasattr(os, "geteuid") and os.geteuid() != 0):
        print("TestSpwdNonRoot::test_getspnam_exception: skipped non-root user required")
    else:
        try:
            spwd.getspnam("bin")
        except PermissionError:
            print("TestSpwdNonRoot::test_getspnam_exception: ok")
        except KeyError as exc:
            print(f"TestSpwdNonRoot::test_getspnam_exception: skipped bin entry missing: {exc}")
        else:
            raise AssertionError("expected PermissionError for non-root spwd.getspnam('bin')")
"###);
    assert_output(&out, r###"TestSpwdNonRoot::test_getspnam_exception: skipped spwd unavailable
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/spwd/test_spwd_root__test_getspall.py`.
#[test]
fn test_gen_behavior_std_libs_spwd_test_spwd_root__test_getspall() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "spwd"
# dimension = "behavior"
# case = "test_spwd_root__test_getspall"
# subject = "cpython.test_spwd.TestSpwdRoot.test_getspall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_spwd.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_spwd.py::TestSpwdRoot::test_getspall
"""Auto-ported test: TestSpwdRoot::test_getspall."""


import os
import warnings


try:
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", DeprecationWarning)
        import spwd
except ImportError:
    print("TestSpwdRoot::test_getspall: skipped spwd unavailable")
else:
    if not (hasattr(os, "geteuid") and os.geteuid() == 0):
        print("TestSpwdRoot::test_getspall: skipped root privileges required")
    else:
        entries = spwd.getspall()
        assert isinstance(entries, list)
        for entry in entries:
            assert isinstance(entry, spwd.struct_spwd)
        print("TestSpwdRoot::test_getspall: ok")
"###);
    assert_output(&out, r###"TestSpwdRoot::test_getspall: skipped spwd unavailable
"###);
}
