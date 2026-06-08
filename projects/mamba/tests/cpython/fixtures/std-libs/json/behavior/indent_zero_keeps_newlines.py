# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "indent_zero_keeps_newlines"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_indent.py"
# status = "filled"
# ///
"""json.dumps: indent=0 still emits newlines between items but with no leading spaces"""
import json

assert json.dumps([1, 2], indent=0) == "[\n1,\n2\n]", repr(json.dumps([1, 2], indent=0))

print("indent_zero_keeps_newlines OK")
