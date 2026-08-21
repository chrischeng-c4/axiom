# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "checkcache_keeps_unchanged_file"
# subject = "linecache.checkcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.checkcache: checkcache keeps the cache entry of an unchanged on-disk file, over a TemporaryDirectory"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.TemporaryDirectory() as d:
    keep = os.path.join(d, "keep.py")
    with open(keep, "w") as fh:
        fh.write("x = 1\ny = 2\n")
    linecache.getline(keep, 1)
    assert keep in linecache.cache, "keep cached"
    linecache.checkcache(keep)
    assert keep in linecache.cache, "unchanged kept"
print("checkcache_keeps_unchanged_file OK")
