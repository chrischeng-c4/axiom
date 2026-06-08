# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getlines_returns_list_per_line"
# subject = "linecache.getlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getlines: getlines returns a list[str] with one newline-terminated entry per source line, over a tempfile"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
    fh.write("alpha\nbravo\ncharlie\ndelta\n")
    fn = fh.name
try:
    lines = linecache.getlines(fn)
    assert type(lines) is list, "getlines is list"
    assert len(lines) == 4, "getlines len"
    assert lines[0].rstrip() == "alpha", "getlines[0]"
    assert lines[3] == "delta\n", "getlines[3] newline-terminated"
finally:
    os.unlink(fn)
print("getlines_returns_list_per_line OK")
