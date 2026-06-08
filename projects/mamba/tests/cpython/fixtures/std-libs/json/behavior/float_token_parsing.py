# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "float_token_parsing"
# subject = "json.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_float.py"
# status = "filled"
# ///
"""json.loads: exponent and signed-zero float tokens parse correctly: 1e2 is 100.0 and -0.0 equals 0.0"""
import json

assert json.loads("1e2") == 100.0, f"1e2 = {json.loads('1e2')!r}"
assert isinstance(json.loads("1e2"), float), json.loads("1e2")
assert json.loads("-0.0") == 0.0, f"-0.0 = {json.loads('-0.0')!r}"
assert json.loads("3.14") == 3.14, json.loads("3.14")

print("float_token_parsing OK")
