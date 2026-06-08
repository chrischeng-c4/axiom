# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "getweakrefcount_tracks_live_refs"
# subject = "weakref.getweakrefcount"
# kind = "semantic"
# xfail = "mamba getweakrefcount returns 0 (no weak-ref registry under refcount-only runtime, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.getweakrefcount: getweakrefcount counts distinct live refs (two callback refs = 2) and drops as refs are deleted"""
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


# Refs created with callbacks are always distinct objects, so two of them
# count as two live weak references to the same referent.
n = _Node(10)
r1 = weakref.ref(n, lambda _: None)
r2 = weakref.ref(n, lambda _: None)
assert weakref.getweakrefcount(n) == 2, f"refcount with 2 cbs = {weakref.getweakrefcount(n)!r}"
del r1
assert weakref.getweakrefcount(n) == 1, f"refcount after del r1 = {weakref.getweakrefcount(n)!r}"

print("getweakrefcount_tracks_live_refs OK")
