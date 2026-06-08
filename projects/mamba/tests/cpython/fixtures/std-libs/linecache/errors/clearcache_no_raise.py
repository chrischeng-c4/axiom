# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "errors"
# case = "clearcache_no_raise"
# subject = "linecache.clearcache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.clearcache: clearcache always succeeds and returns None"""
import linecache

assert linecache.clearcache() is None, "clearcache returns None"
print("clearcache_no_raise OK")
