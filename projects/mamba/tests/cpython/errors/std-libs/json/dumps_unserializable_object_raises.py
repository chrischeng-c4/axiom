# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "errors"
# case = "dumps_unserializable_object_raises"
# subject = "json.dumps"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_dump.py"
# status = "filled"
# ///
"""json.dumps: dumps_unserializable_object_raises (errors)."""
import json

_raised = False
try:
    json.dumps(object())
except TypeError:
    _raised = True
assert _raised, "dumps_unserializable_object_raises: expected TypeError"
print("dumps_unserializable_object_raises OK")
