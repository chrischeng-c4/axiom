# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "errors"
# case = "invalid_statement_raises"
# subject = "tomllib.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_error.py"
# status = "filled"
# ///
"""tomllib.loads: invalid_statement_raises (errors)."""
import tomllib

_raised = False
try:
    tomllib.loads('not = a = toml = file')
except tomllib.TOMLDecodeError:
    _raised = True
assert _raised, "invalid_statement_raises: expected tomllib.TOMLDecodeError"
print("invalid_statement_raises OK")
