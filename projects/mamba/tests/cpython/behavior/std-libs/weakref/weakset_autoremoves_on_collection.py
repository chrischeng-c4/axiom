# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "weakset_autoremoves_on_collection"
# subject = "weakref.WeakSet"
# kind = "semantic"
# xfail = "mamba refcount-only: WeakSet does not auto-remove collected members (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakSet: WeakSet drops a member once it is collected"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


ws = weakref.WeakSet()
n = _Node(9)
ws.add(n)
assert n in ws, "WeakSet contains node"
del n
gc.collect()
assert len(ws) == 0, f"WeakSet cleared after GC = {len(ws)!r}"

print("weakset_autoremoves_on_collection OK")
