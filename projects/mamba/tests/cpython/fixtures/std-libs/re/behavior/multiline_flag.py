# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "multiline_flag"
# subject = "re.MULTILINE"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.MULTILINE: re.MULTILINE makes ^ and $ per-line: r'^\\w+' over 'line1\\nline2\\nline3' findall is ['line1','line2','line3']"""
import re

text = "line1\nline2\nline3"
assert re.findall(r"^\w+", text, re.MULTILINE) == ["line1", "line2", "line3"]

print("multiline_flag OK")
