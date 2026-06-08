# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "string_escape_roundtrip"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_scanstring.py"
# status = "filled"
# ///
"""json.dumps: embedded quote, backslash, and newline characters are escaped on dump and restored verbatim on load"""
import json

assert json.dumps('quote"here') == '"quote\\"here"', json.dumps('quote"here')
assert json.dumps("back\\slash") == '"back\\\\slash"', json.dumps("back\\slash")
assert json.loads('"a\\nb"') == "a\nb", json.loads('"a\\nb"')

original = 'tab\there "quote" and \\back\\'
assert json.loads(json.dumps(original)) == original, "escape round-trip"

print("string_escape_roundtrip OK")
