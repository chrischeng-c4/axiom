# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "invalid_parse_float_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: invalid_parse_float_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('f=0.1', parse_float=lambda s: {})
except ValueError:
    _raised = True
assert _raised, "invalid_parse_float_raises: expected ValueError"
print("invalid_parse_float_raises OK")
