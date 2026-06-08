# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "behavior"
# case = "iskeyword_non_string_returns_false"
# subject = "keyword.iskeyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_keyword.py"
# status = "filled"
# ///
"""keyword.iskeyword: iskeyword(int) and iskeyword(None) return False without raising"""
import keyword

# A non-string argument compares unequal to every kwlist entry: False, no raise.
assert keyword.iskeyword(123) is False, "iskeyword(123) should be False"
assert keyword.iskeyword(None) is False, "iskeyword(None) should be False"

print("iskeyword_non_string_returns_false OK")
