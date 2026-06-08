# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "suppress_does_not_catch_other_type"
# subject = "contextlib.suppress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.suppress: suppress_does_not_catch_other_type (errors)."""
import contextlib

_raised = False
try:
    exec('import contextlib\nwith contextlib.suppress(ValueError):\n    raise KeyError("x")')
except KeyError:
    _raised = True
assert _raised, "suppress_does_not_catch_other_type: expected KeyError"
print("suppress_does_not_catch_other_type OK")
