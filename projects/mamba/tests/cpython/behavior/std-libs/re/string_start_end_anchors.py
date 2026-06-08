# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "string_start_end_anchors"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.search: \\A anchors at string start and \\Z at string end even under MULTILINE: r'^\\Aabc\\Z$' matches 'abc' but rejects '\\nabc\\n'"""
import re

assert re.search(r"^\Aabc\Z$", "abc", re.M).group(0) == "abc", "\\A..\\Z single line"
assert re.search(r"^\Aabc\Z$", "\nabc\n", re.M) is None, "\\A..\\Z reject newlines under MULTILINE"

print("string_start_end_anchors OK")
