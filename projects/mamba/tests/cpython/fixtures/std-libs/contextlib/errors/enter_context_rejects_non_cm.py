# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "enter_context_rejects_non_cm"
# subject = "contextlib.ExitStack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.ExitStack: enter_context_rejects_non_cm (errors)."""
import contextlib

_raised = False
try:
    contextlib.ExitStack().enter_context(object())
except TypeError:
    _raised = True
assert _raised, "enter_context_rejects_non_cm: expected TypeError"
print("enter_context_rejects_non_cm OK")
