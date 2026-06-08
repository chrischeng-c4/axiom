# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winapi"
# dimension = "behavior"
# case = "win_api_tests__test_getshortpathname"
# subject = "cpython.test_winapi.WinAPITests.test_getshortpathname"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_winapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_winapi.py::WinAPITests::test_getshortpathname
"""Auto-ported test: WinAPITests::test_getshortpathname."""


import os
import pathlib
import re
import sys


if sys.platform != "win32":
    print("WinAPITests::test_getshortpathname: skipped Windows only")
else:
    import _winapi

    testfn = pathlib.Path(os.getenv("ProgramFiles"))
    if not os.path.isdir(testfn):
        print("WinAPITests::test_getshortpathname: skipped require %ProgramFiles%")
    else:
        try:
            _winapi.GetShortPathName(testfn)
        except TypeError:
            pass
        else:
            raise AssertionError("pathlib.Path should be rejected")

        actual = _winapi.GetShortPathName(os.fsdecode(testfn))
        assert re.match(r".\:\\PROGRA~\d", actual.upper()) is not None, actual
        print("WinAPITests::test_getshortpathname: ok")
