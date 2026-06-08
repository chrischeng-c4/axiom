# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "behavior"
# case = "checkcache_drops_vanished_file"
# subject = "linecache.checkcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_linecache.py"
# status = "filled"
# ///
"""linecache.checkcache: checkcache drops the cache entry once its backing file is deleted, over a TemporaryDirectory"""
import linecache
import tempfile
import os

linecache.clearcache()
with tempfile.TemporaryDirectory() as d:
    gone = os.path.join(d, "gone.py")
    with open(gone, "w") as fh:
        fh.write("x = 1\ny = 2\n")
    linecache.getline(gone, 1)
    assert gone in linecache.cache, "gone cached"
    os.unlink(gone)
    linecache.checkcache(gone)
    assert gone not in linecache.cache, "vanished dropped"
print("checkcache_drops_vanished_file OK")
