# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "dumps_bytes_raises"
# subject = "json.dumps"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_dump.py"
# status = "filled"
# ///
"""json.dumps: dumps_bytes_raises (errors)."""
import json

_raised = False
try:
    json.dumps(b"bytes are not json")
except TypeError:
    _raised = True
assert _raised, "dumps_bytes_raises: expected TypeError"
print("dumps_bytes_raises OK")
