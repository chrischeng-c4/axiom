# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "parameter_bad_kind_raises_valueerror"
# subject = "inspect.Parameter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: parameter_bad_kind_raises_valueerror (errors)."""
import inspect

_raised = False
try:
    inspect.Parameter('x', kind=999)
except ValueError:
    _raised = True
assert _raised, "parameter_bad_kind_raises_valueerror: expected ValueError"
print("parameter_bad_kind_raises_valueerror OK")
