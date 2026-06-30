# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "errors"
# case = "weakset_rejects_non_iterable_data"
# subject = "_weakrefset.WeakSet"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/_weakrefset.py"
# status = "filled"
# ///
"""_weakrefset.WeakSet: constructor rejects non-iterable data."""
from _weakrefset import WeakSet

_raised = False
try:
    WeakSet(12345)
except TypeError as exc:
    _raised = True
    assert "object is not iterable" in str(exc), str(exc)
assert _raised, "WeakSet(12345) must raise TypeError"
print("weakset_rejects_non_iterable_data OK")
