# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "decode_error_attributes"
# subject = "json.JSONDecodeError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_fail.py"
# status = "filled"
# ///
"""json.JSONDecodeError: a JSONDecodeError raised by loads exposes msg/pos/lineno/colno locating the malformed token"""
import json

raised = None
try:
    json.loads('{"a": 1,\n"b": ?}')
except json.JSONDecodeError as e:
    raised = e
assert raised is not None, "malformed JSON must raise JSONDecodeError"
assert hasattr(raised, "msg"), "JSONDecodeError has msg"
assert isinstance(raised.pos, int), raised.pos
assert raised.lineno == 2, raised.lineno
assert isinstance(raised.colno, int) and raised.colno > 0, raised.colno

print("decode_error_attributes OK")
