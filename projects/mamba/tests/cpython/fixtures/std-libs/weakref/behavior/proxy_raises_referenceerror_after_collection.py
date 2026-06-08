# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_raises_referenceerror_after_collection"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba refcount-only: dead proxy does not raise ReferenceError (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: accessing a proxy after its referent is collected raises ReferenceError"""
import gc
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


n = _Node(6)
p = weakref.proxy(n)
del n
gc.collect()
_raised = False
try:
    _ = p.val
except ReferenceError:
    _raised = True
assert _raised, "dead proxy raises ReferenceError"

print("proxy_raises_referenceerror_after_collection OK")
