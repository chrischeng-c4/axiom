# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "unterminated_string_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: unterminated_string_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('a = "unterminated\n')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "unterminated_string_raises: expected tomllib.TOMLDecodeError"
print("unterminated_string_raises OK")
