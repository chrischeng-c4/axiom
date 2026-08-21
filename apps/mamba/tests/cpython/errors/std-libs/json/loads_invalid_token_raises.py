# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "loads_invalid_token_raises"
# subject = "json.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_fail.py"
# status = "filled"
# ///
"""json.loads: loads_invalid_token_raises (errors)."""
import json

_raised = False
try:
    json.loads("{invalid}")
except json.JSONDecodeError:
    _raised = True
assert _raised, "loads_invalid_token_raises: expected json.JSONDecodeError"
print("loads_invalid_token_raises OK")
