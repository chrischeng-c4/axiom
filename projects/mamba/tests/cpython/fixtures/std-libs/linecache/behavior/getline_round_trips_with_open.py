# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "getline_round_trips_with_open"
# subject = "linecache.getline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.getline: every line read directly via open() equals getline of that 1-based lineno, over a tempfile"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as fh:
    fh.write("alpha\nbravo\ncharlie\ndelta\n")
    fn = fh.name
try:
    with open(fn, encoding="utf-8") as f:
        for index, line in enumerate(f):
            assert line == linecache.getline(fn, index + 1), "round-trip"
finally:
    os.unlink(fn)
print("getline_round_trips_with_open OK")
