# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "ref_callback_fires_on_collection"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = "mamba refcount-only: weakref collection callbacks never fire (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: a ref() callback fires on referent collection and receives the now-dead ref as its only argument"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


# The callback receives the (now-dead) ref object as its only argument.
seen = []
n = _Node(11)
r = weakref.ref(n, lambda w: seen.append(w))
del n
gc.collect()
assert seen == [r], f"callback arg is the ref = {seen!r}"
assert seen[0]() is None, "ref passed to callback is already dead"

print("ref_callback_fires_on_collection OK")
