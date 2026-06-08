# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "clearcache_empties_cache"
# subject = "linecache.clearcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.clearcache: clearcache removes every populated entry from linecache.cache, over a TemporaryDirectory"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.TemporaryDirectory() as d:
    paths = []
    for name in ("one", "two"):
        p = os.path.join(d, name + ".py")
        with open(p, "w") as fh:
            fh.write("x = 1\ny = 2\n")
        paths.append(p)
        linecache.getline(p, 1)  # populate the cache
    assert all(p in linecache.cache for p in paths), "populated"
    linecache.clearcache()
    assert all(p not in linecache.cache for p in paths), "cleared"
print("clearcache_empties_cache OK")
