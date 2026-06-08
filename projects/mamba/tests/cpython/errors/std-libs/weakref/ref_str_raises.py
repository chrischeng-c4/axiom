# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "ref_str_raises"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref_str_raises (errors)."""
import weakref

_raised = False
try:
    weakref.ref('hello')
except TypeError:
    _raised = True
assert _raised, "ref_str_raises: expected TypeError"
print("ref_str_raises OK")
