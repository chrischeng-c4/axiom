# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "reset_wrong_var_raises_valueerror"
# subject = "contextvars.ContextVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: reset_wrong_var_raises_valueerror (errors)."""
import contextvars

_raised = False
try:
    contextvars.ContextVar('e_a').reset(contextvars.ContextVar('e_b').set(1))
except ValueError:
    _raised = True
assert _raised, "reset_wrong_var_raises_valueerror: expected ValueError"
print("reset_wrong_var_raises_valueerror OK")
