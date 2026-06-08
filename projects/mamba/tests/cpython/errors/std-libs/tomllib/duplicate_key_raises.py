# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "duplicate_key_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: duplicate_key_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('a = 1\na = 2\n')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "duplicate_key_raises: expected tomllib.TOMLDecodeError"
print("duplicate_key_raises OK")
