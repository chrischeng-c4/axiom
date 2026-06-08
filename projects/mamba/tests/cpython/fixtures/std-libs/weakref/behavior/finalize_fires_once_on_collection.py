# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "finalize_fires_once_on_collection"
# subject = "weakref.finalize"
# kind = "semantic"
# xfail = "mamba refcount-only: finalize does not fire on referent collection (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.finalize: finalize fires exactly once when its object is collected and alive flips to False"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


fired = []
n = _Node(7)
fin = weakref.finalize(n, lambda: fired.append(1))
del n
gc.collect()
assert fired == [1], f"finalize fired once = {fired!r}"
assert not fin.alive, "finalize.alive is False after firing"

print("finalize_fires_once_on_collection OK")
