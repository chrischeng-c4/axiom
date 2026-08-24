use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/startfile/test_case__test_empty.py`.
#[test]
fn test_gen_behavior_std_libs_startfile_test_case__test_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "startfile"
# dimension = "behavior"
# case = "test_case__test_empty"
# subject = "cpython.test_startfile.TestCase.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_startfile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_startfile.py::TestCase::test_empty
"""Auto-ported test: TestCase::test_empty."""


import os
import platform
import sys
import tempfile
from os import path


if not hasattr(os, "startfile"):
    print("TestCase::test_empty: skipped os.startfile unavailable")
elif hasattr(platform, "win32_is_iot") and platform.win32_is_iot():
    print("TestCase::test_empty: skipped Windows IoT Core or nanoserver")
else:
    with tempfile.TemporaryDirectory() as tmp:
        empty = path.join(tmp, "empty.vbs")
        with open(empty, "w", encoding="utf-8") as handle:
            handle.write("' empty script\n")
        cwd = path.dirname(sys.executable)
        os.startfile(empty)
        os.startfile(empty, "open")
        os.startfile(empty, cwd=cwd)
    print("TestCase::test_empty: ok")
"###);
    assert_output(&out, r###"TestCase::test_empty: skipped os.startfile unavailable
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/startfile/test_case__test_nonexisting.py`.
#[test]
fn test_gen_behavior_std_libs_startfile_test_case__test_nonexisting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "startfile"
# dimension = "behavior"
# case = "test_case__test_nonexisting"
# subject = "cpython.test_startfile.TestCase.test_nonexisting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_startfile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_startfile.py::TestCase::test_nonexisting
"""Auto-ported test: TestCase::test_nonexisting."""


import os


if not hasattr(os, "startfile"):
    print("TestCase::test_nonexisting: skipped os.startfile unavailable")
else:
    try:
        os.startfile("nonexisting.vbs")
    except OSError:
        print("TestCase::test_nonexisting: ok")
    else:
        raise AssertionError("expected OSError for nonexisting.vbs")
"###);
    assert_output(&out, r###"TestCase::test_nonexisting: skipped os.startfile unavailable
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/startfile/test_case__test_python.py`.
#[test]
fn test_gen_behavior_std_libs_startfile_test_case__test_python() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "startfile"
# dimension = "behavior"
# case = "test_case__test_python"
# subject = "cpython.test_startfile.TestCase.test_python"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_startfile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_startfile.py::TestCase::test_python
"""Auto-ported test: TestCase::test_python."""


import os
import platform
import sys
from os import path


if not hasattr(os, "startfile"):
    print("TestCase::test_python: skipped os.startfile unavailable")
elif hasattr(platform, "win32_is_iot") and platform.win32_is_iot():
    print("TestCase::test_python: skipped Windows IoT Core or nanoserver")
else:
    cwd, name = path.split(sys.executable)
    os.startfile(name, arguments="-V", cwd=cwd)
    os.startfile(name, arguments="-V", cwd=cwd, show_cmd=0)
    print("TestCase::test_python: ok")
"###);
    assert_output(&out, r###"TestCase::test_python: skipped os.startfile unavailable
"###);
}
