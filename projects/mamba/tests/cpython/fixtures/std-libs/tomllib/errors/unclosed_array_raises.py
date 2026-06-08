# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "unclosed_array_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: unclosed_array_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('key = [unclosed')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "unclosed_array_raises: expected tomllib.TOMLDecodeError"
print("unclosed_array_raises OK")
