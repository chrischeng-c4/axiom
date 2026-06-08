# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wmi"
# dimension = "errors"
# case = "wmi_tests__test_wmi_query_not_select"
# subject = "cpython.test_wmi.WmiTests.test_wmi_query_not_select"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_wmi.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_wmi.py::WmiTests::test_wmi_query_not_select
"""Auto-ported test: WmiTests::test_wmi_query_not_select (CPython 3.12 oracle)."""


try:
    import _wmi
except ImportError:
    print("WmiTests::test_wmi_query_not_select: skipped, _wmi unavailable")
    raise SystemExit(0)


try:
    _wmi.exec_query("not select, just in case someone tries something")
except ValueError:
    pass
else:
    raise AssertionError("_wmi.exec_query accepted a non-SELECT query")

print("WmiTests::test_wmi_query_not_select: ok")
