# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winapi"
# dimension = "behavior"
# case = "win_api_tests__test_getlongpathname"
# subject = "cpython.test_winapi.WinAPITests.test_getlongpathname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_winapi.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_winapi.py::WinAPITests::test_getlongpathname
"""Auto-ported test: WinAPITests::test_getlongpathname."""


import os
import pathlib
import sys


if sys.platform != "win32":
    print("WinAPITests::test_getlongpathname: skipped Windows only")
else:
    import _winapi

    testfn = pathlib.Path(os.getenv("ProgramFiles")).parents[-1] / "PROGRA~1"
    if not os.path.isdir(testfn):
        print("WinAPITests::test_getlongpathname: skipped require x:\\PROGRA~1")
    else:
        try:
            _winapi.GetLongPathName(testfn)
        except TypeError:
            pass
        else:
            raise AssertionError("pathlib.Path should be rejected")

        actual = _winapi.GetLongPathName(os.fsdecode(testfn))
        candidates = set(testfn.parent.glob("Progra*"))
        assert pathlib.Path(actual) in candidates
        print("WinAPITests::test_getlongpathname: ok")
