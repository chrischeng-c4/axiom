# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "unified_diff_headers_and_lines"
# subject = "difflib.unified_diff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.unified_diff: unified_diff emits ---/+++ file headers and the -<removed>/+<added> body lines for a single changed line"""
import difflib

_old = "line1\nline2\nline3\n".splitlines(keepends=True)
_new = "line1\nchanged\nline3\n".splitlines(keepends=True)
_ud = list(difflib.unified_diff(
    _old, _new, fromfile="old", tofile="new", lineterm=""))
assert any(line.startswith("---") for line in _ud), f"--- header missing: {_ud!r}"
assert any(line.startswith("+++") for line in _ud), f"+++ header missing: {_ud!r}"
assert any(line.startswith("-line2") for line in _ud), f"-line2 missing: {_ud!r}"
assert any(line.startswith("+changed") for line in _ud), f"+changed missing: {_ud!r}"
print("unified_diff_headers_and_lines OK")
