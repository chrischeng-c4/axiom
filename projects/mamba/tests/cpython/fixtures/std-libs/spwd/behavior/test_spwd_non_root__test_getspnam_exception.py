# /// script
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
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_spwd.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
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
