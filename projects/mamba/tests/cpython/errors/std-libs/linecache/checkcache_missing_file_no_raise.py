# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "checkcache_missing_file_no_raise"
# subject = "linecache.checkcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.checkcache: checkcache on a missing/unknown filename does not raise and returns None"""
import linecache

linecache.clearcache()
assert linecache.checkcache("/no/such/file.py") is None, "checkcache missing returns None"
print("checkcache_missing_file_no_raise OK")
