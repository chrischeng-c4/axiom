# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "bad_date_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: bad_date_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('d = 2024-13-01\n')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "bad_date_raises: expected tomllib.TOMLDecodeError"
print("bad_date_raises OK")
