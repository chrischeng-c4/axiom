# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "ref_deref_returns_referent"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref() returns the exact referent object while it is alive; its attributes read through"""
import weakref


class _Obj:
    def __init__(self, v):
        self.v = v


o = _Obj(42)
r = weakref.ref(o)
assert isinstance(r, weakref.ref), f"ref type = {type(r)!r}"
assert r() is o, "deref returns the exact referent"
assert r().v == 42, f"deref.v = {r().v!r}"

print("ref_deref_returns_referent OK")
