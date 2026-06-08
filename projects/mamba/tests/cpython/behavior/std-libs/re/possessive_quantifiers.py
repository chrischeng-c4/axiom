# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "possessive_quantifiers"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.match: possessive quantifiers (*+, ++, ?+, {m,n}+) never give back: r'e*+e' fails on 'eeee' while r'e++a' matches 'eeea'"""
import re

assert re.match(r"e*+e", "eeee") is None, "e*+ eats all, no e left"
assert re.match(r"e++a", "eeea").group(0) == "eeea", "e++a"
assert re.match(r"e?+a", "ea").group(0) == "ea", "e?+a"
assert re.match(r"e{2,4}+a", "eeea").group(0) == "eeea", "e{2,4}+a"
assert re.search(r"x++", "axx").span() == (1, 3), "x++ span"

print("possessive_quantifiers OK")
