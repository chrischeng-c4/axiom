# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tcl"
# dimension = "behavior"
# case = "tcl_test__test_eval"
# subject = "cpython.test_tcl.TclTest.testEval"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tcl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tcl.py::TclTest::testEval
"""Auto-ported test: TclTest::testEval (CPython 3.12 oracle)."""


try:
    import _tkinter  # noqa: F401
    from tkinter import Tcl
except ImportError:
    print("TclTest::testEval: skipped, _tkinter unavailable")
    raise SystemExit(0)


tcl = Tcl()
tcl.eval("set a 1")
assert tcl.eval("set a") == "1"

print("TclTest::testEval: ok")
