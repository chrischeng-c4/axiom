# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "finalization"
# dimension = "behavior"
# case = "non_gc_simple_base__test"
# subject = "cpython.test_finalization.NonGCSimpleBase.test"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_finalization.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: NonGCSimpleBase::test helper boundary (CPython 3.12 oracle)."""

from test.test_finalization import NonGCSimpleBase


NonGCSimpleBase.del_calls.append("stale-del")
NonGCSimpleBase.tp_del_calls.append("stale-tp-del")
with NonGCSimpleBase.test():
    assert NonGCSimpleBase.del_calls == []
    assert NonGCSimpleBase.tp_del_calls == []
    assert NonGCSimpleBase._cleaning is False

assert NonGCSimpleBase._cleaning is True
assert NonGCSimpleBase.del_calls == []
assert NonGCSimpleBase.tp_del_calls == []
assert NonGCSimpleBase.errors == []

try:
    with NonGCSimpleBase.test():
        NonGCSimpleBase.errors.append(RuntimeError("recorded finalizer error"))
except RuntimeError as exc:
    assert str(exc) == "recorded finalizer error", str(exc)
else:
    raise AssertionError("NonGCSimpleBase.test should re-raise recorded errors")

assert NonGCSimpleBase._cleaning is True
assert NonGCSimpleBase.errors == []

print("NonGCSimpleBase::test helper boundary: ok")
