# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_one_based_indexing"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: getline is 1-based: line 1 is the first source line, line 3 the third, over a tempfile"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
    fh.write("alpha\nbravo\ncharlie\ndelta\n")
    fn = fh.name
try:
    assert linecache.getline(fn, 1).rstrip() == "alpha", "getline(1)"
    assert linecache.getline(fn, 3).rstrip() == "charlie", "getline(3)"
finally:
    os.unlink(fn)
print("getline_one_based_indexing OK")
