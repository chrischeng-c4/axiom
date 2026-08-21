# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "ref_expires_after_referent_collected"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = "mamba refcount-only: ref does not expire on referent collection (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: a ref() returns None after its referent is deleted and garbage-collected"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


n = _Node(1)
r = weakref.ref(n)
assert r() is not None, "ref alive before del"
del n
gc.collect()
assert r() is None, "ref dead after del+gc"

print("ref_expires_after_referent_collected OK")
