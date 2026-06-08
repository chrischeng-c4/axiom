# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ttk_textonly"
# dimension = "behavior"
# case = "internal_functions_test__test_format_mapdict"
# subject = "cpython.test_ttk_textonly.InternalFunctionsTest.test_format_mapdict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ttk_textonly.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ttk_textonly.py::InternalFunctionsTest::test_format_mapdict
"""Auto-ported test: InternalFunctionsTest::test_format_mapdict (CPython oracle)."""


try:
    import _tkinter  # noqa: F401
    from tkinter import ttk
except ImportError:
    print("InternalFunctionsTest::test_format_mapdict: skipped, tkinter unavailable")
    raise SystemExit(0)


opts = {"a": [("b", "c", "val"), ("d", "otherval"), ("", "single")]}
result = ttk._format_mapdict(opts)
assert len(result) == len(list(opts.keys())) * 2, result
assert result == ("-a", "{b c} val d otherval {} single"), result
assert ttk._format_mapdict(opts, script=True) == (
    "-a",
    "{{b c} val d otherval {} single}",
)

assert ttk._format_mapdict({2: []}) == ("-2", "")

opts = {"üñíćódè": [("á", "vãl")]}
assert ttk._format_mapdict(opts) == ("-üñíćódè", "á vãl")

assert ttk._format_mapdict({"opt": [("value",)]}) == ("-opt", "{} value")
assert ttk._format_mapdict({"opt": [("", "", "hi")]}) == ("-opt", "{ } hi")

for invalid in ({"opt": [(1, 2, "valid val")]}, {"opt": [([1], "2", "valid val")]}):
    try:
        ttk._format_mapdict(invalid)
    except TypeError:
        pass
    else:
        raise AssertionError(f"ttk._format_mapdict({invalid!r}) did not raise TypeError")

assert ttk._format_mapdict({"opt": [[1, "value"]]}) == ("-opt", "1 value")
for stateval in (None, 0, False, "", set()):
    assert ttk._format_mapdict({"opt": [(stateval, "value")]}) == ("-opt", "{} value")

try:
    ttk._format_mapdict({"a": None})
except TypeError:
    pass
else:
    raise AssertionError("ttk._format_mapdict({'a': None}) did not raise TypeError")

print("InternalFunctionsTest::test_format_mapdict: ok")
