# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "fullmatch_requires_whole_string"
# subject = "re.fullmatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.fullmatch: re.fullmatch requires the whole string to match: r'\\d{3}' matches '123' but returns None for '1234'"""
import re

assert re.fullmatch(r"\d{3}", "123") is not None, "fullmatch exact"
assert re.fullmatch(r"\d{3}", "1234") is None, "fullmatch too long -> None"

print("fullmatch_requires_whole_string OK")
