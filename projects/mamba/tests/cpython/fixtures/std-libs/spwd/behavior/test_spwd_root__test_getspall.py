# /// script
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
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_spwd.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
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
