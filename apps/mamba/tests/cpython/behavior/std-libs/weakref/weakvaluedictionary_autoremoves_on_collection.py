# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "weakvaluedictionary_autoremoves_on_collection"
# subject = "weakref.WeakValueDictionary"
# kind = "semantic"
# xfail = "mamba refcount-only: WeakValueDictionary does not auto-remove collected values (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: WeakValueDictionary drops an entry once its value is collected"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


wvd = weakref.WeakValueDictionary()
n = _Node(3)
wvd["x"] = n
assert "x" in wvd, "entry present"
del n
gc.collect()
assert "x" not in wvd, "entry auto-removed after GC"

print("weakvaluedictionary_autoremoves_on_collection OK")
