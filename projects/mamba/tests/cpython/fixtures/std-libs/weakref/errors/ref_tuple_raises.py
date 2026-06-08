# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "ref_tuple_raises"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref_tuple_raises (errors)."""
import weakref

_raised = False
try:
    weakref.ref((1, 2, 3))
except TypeError:
    _raised = True
assert _raised, "ref_tuple_raises: expected TypeError"
print("ref_tuple_raises OK")
