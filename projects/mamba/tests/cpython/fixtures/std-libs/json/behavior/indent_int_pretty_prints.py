# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "indent_int_pretty_prints"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_indent.py"
# status = "filled"
# ///
"""json.dumps: integer indent emits a newline plus N spaces per nesting level and never leaves trailing whitespace on a line"""
import json

assert json.dumps([1, 2], indent=2) == "[\n  1,\n  2\n]", repr(json.dumps([1, 2], indent=2))

# When indent is set, the item separator loses its trailing space, so no line
# carries trailing whitespace.
out = json.dumps({"b": 1, "a": 2}, indent=2)
assert out.count("\n") >= 2, repr(out)
assert " \n" not in out, f"trailing space leaked: {out!r}"

print("indent_int_pretty_prints OK")
