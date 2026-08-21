# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "ndiff_marks_added_removed_unchanged"
# subject = "difflib.ndiff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.ndiff: ndiff prefixes lines with '- ' (removed), '+ ' (added) and '  ' (unchanged)"""
import difflib

_nd = list(difflib.ndiff(["foo\n", "bar\n"], ["foo\n", "baz\n"]))
assert any(line.startswith("- bar") for line in _nd), f"- line missing: {_nd!r}"
assert any(line.startswith("+ baz") for line in _nd), f"+ line missing: {_nd!r}"
assert any(line.startswith("  foo") for line in _nd), f"unchanged line missing: {_nd!r}"
print("ndiff_marks_added_removed_unchanged OK")
