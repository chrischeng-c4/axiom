# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_zero_and_negative_return_empty"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline with lineno 0, -1, or 2**15 returns '' (only 1..len map to lines), over a tempfile"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
    fh.write("alpha\nbravo\ncharlie\ndelta\n")
    fn = fh.name
try:
    assert linecache.getline(fn, 99) == "", "getline(99)"
    assert linecache.getline(fn, 2 ** 15) == "", "getline(2**15)"
    assert linecache.getline(fn, 0) == "", "getline(0)"
    assert linecache.getline(fn, -1) == "", "getline(-1)"
finally:
    os.unlink(fn)
print("getline_zero_and_negative_return_empty OK")
