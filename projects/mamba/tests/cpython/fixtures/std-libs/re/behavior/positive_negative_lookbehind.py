# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "positive_negative_lookbehind"
# subject = "re.search"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.search: positive lookbehind (?<=...) requires the preceding text (r'(?<=b)c' matches 'abc'); negative lookbehind (?<!...) forbids it"""
import re

assert re.search(r"(?<=b)c", "abc") is not None, "lookbehind ok"
assert re.search(r"(?<=x)c", "abc") is None, "lookbehind wrong prefix"
assert re.match(r"ab(?<!c)c", "abc") is not None, "neg lookbehind ok"
assert re.match(r"ab(?<=c)c", "abc") is None, "lookbehind needs c before"

print("positive_negative_lookbehind OK")
