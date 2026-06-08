# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "ref_reinit_bogus_args_raises"
# subject = "weakref.ref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref_reinit_bogus_args_raises (errors)."""
import weakref

_r = weakref.ref(Exception)

_raised = False
try:
    _r.__init__(0, 0, 0, 0, 0)
except TypeError:
    _raised = True
assert _raised, "ref_reinit_bogus_args_raises: expected TypeError"
print("ref_reinit_bogus_args_raises OK")
