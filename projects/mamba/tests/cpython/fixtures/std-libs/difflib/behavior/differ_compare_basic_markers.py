# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "differ_compare_basic_markers"
# subject = "difflib.Differ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.Differ: Differ().compare marks each line: '  ' unchanged, '- ' removed, '+ ' added"""
import difflib

_diff = list(difflib.Differ().compare(["foo\n", "bar\n"], ["foo\n", "baz\n"]))
assert "  foo\n" in _diff, f"unchanged marker missing: {_diff!r}"
assert "- bar\n" in _diff, f"removed marker missing: {_diff!r}"
assert "+ baz\n" in _diff, f"added marker missing: {_diff!r}"
print("differ_compare_basic_markers OK")
