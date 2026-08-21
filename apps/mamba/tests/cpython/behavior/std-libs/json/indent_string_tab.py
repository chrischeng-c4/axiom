# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "indent_string_tab"
# subject = "json.dumps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_indent.py"
# status = "filled"
# ///
"""json.dumps: a string indent of a tab indents each level with a literal tab character (json.tool --tab equivalent)"""
import json

assert json.dumps([1, 2], indent="\t") == "[\n\t1,\n\t2\n]", repr(json.dumps([1, 2], indent="\t"))

print("indent_string_tab OK")
