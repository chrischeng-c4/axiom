# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "errors"
# case = "unset_get_raises_lookuperror"
# subject = "contextvars.ContextVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""contextvars.ContextVar: unset_get_raises_lookuperror (errors)."""
import contextvars

_raised = False
try:
    contextvars.ContextVar('e_unset').get()
except LookupError:
    _raised = True
assert _raised, "unset_get_raises_lookuperror: expected LookupError"
print("unset_get_raises_lookuperror OK")
