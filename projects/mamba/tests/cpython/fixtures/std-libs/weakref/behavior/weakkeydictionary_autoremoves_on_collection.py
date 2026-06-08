# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "weakkeydictionary_autoremoves_on_collection"
# subject = "weakref.WeakKeyDictionary"
# kind = "semantic"
# xfail = "mamba refcount-only: WeakKeyDictionary does not auto-remove collected keys (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakKeyDictionary: WeakKeyDictionary drops an entry once its key is collected"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


wkd = weakref.WeakKeyDictionary()
n = _Node(4)
wkd[n] = "data"
assert n in wkd, "key present"
del n
gc.collect()
assert len(wkd) == 0, f"WeakKeyDictionary auto-removed = {len(wkd)!r}"

print("weakkeydictionary_autoremoves_on_collection OK")
