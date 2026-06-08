# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_pos_endpos_string_attrs"
# subject = "re.Match.string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.string: a Match exposes pos/endpos describing the search window and .string the original subject: re.match(r'(a)','a') has pos 0, endpos 1, string 'a'"""
import re

m = re.match(r"(a)", "a")
assert m.pos == 0 and m.endpos == 1, "pos/endpos describe the window"
assert m.string == "a", "match.string is the subject"
assert m.re is not None, "match.re present"

print("match_pos_endpos_string_attrs OK")
