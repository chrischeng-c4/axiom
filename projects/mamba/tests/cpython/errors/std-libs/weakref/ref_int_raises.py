# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "ref_int_raises"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref_int_raises (errors)."""
import weakref

_raised = False
try:
    weakref.ref(42)
except TypeError:
    _raised = True
assert _raised, "ref_int_raises: expected TypeError"
print("ref_int_raises OK")
