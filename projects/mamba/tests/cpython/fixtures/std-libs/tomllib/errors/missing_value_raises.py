# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "missing_value_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: missing_value_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('\n\nfwfw=')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "missing_value_raises: expected tomllib.TOMLDecodeError"
print("missing_value_raises OK")
