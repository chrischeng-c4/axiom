# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "asyncgen_hooks_roundtrip"
# subject = "sys.set_asyncgen_hooks"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.set_asyncgen_hooks: asyncgen hooks default to None and round-trip through set_asyncgen_hooks(firstiter=)/(finalizer=) and get_asyncgen_hooks(), then restore"""
import sys

old = sys.get_asyncgen_hooks()
assert old.firstiter is None and old.finalizer is None, "asyncgen hooks start None"
_first = lambda *a: None
_final = lambda *a: None
try:
    sys.set_asyncgen_hooks(firstiter=_first)
    h = sys.get_asyncgen_hooks()
    assert h.firstiter is _first and h[0] is _first, "firstiter set"
    assert h.finalizer is None and h[1] is None, "finalizer untouched"
    sys.set_asyncgen_hooks(finalizer=_final)
    h = sys.get_asyncgen_hooks()
    assert h.firstiter is _first and h.finalizer is _final, "both hooks set"
finally:
    sys.set_asyncgen_hooks(*old)
assert sys.get_asyncgen_hooks().firstiter is None, "asyncgen hooks restored"
print("asyncgen_hooks_roundtrip OK")
