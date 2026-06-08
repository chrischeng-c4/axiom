# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "invalid_value_char_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: invalid_value_char_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('key = @invalid')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "invalid_value_char_raises: expected tomllib.TOMLDecodeError"
print("invalid_value_char_raises OK")
