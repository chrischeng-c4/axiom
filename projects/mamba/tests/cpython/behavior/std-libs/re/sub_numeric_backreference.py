# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_numeric_backreference"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.sub: a numeric backreference in the replacement re-inserts a captured group: re.sub(r'(\\w+)', r'[\\1]', 'hello world') is '[hello] [world]'"""
import re

assert re.sub(r"(\w+)", r"[\1]", "hello world") == "[hello] [world]"

print("sub_numeric_backreference OK")
