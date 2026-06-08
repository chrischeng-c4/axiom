# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "conditional_group"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.match: a conditional group (?(1)yes|no) branches on whether group 1 matched: r'^(\\()?([^()]+)(?(1)\\))$' accepts '(a)' and 'a' but rejects 'a)' and '(a'"""
import re

pat = r"^(\()?([^()]+)(?(1)\))$"
assert re.match(pat, "(a)").groups() == ("(", "a"), "conditional with paren"
assert re.match(pat, "a").groups() == (None, "a"), "conditional no paren"
assert re.match(pat, "a)") is None, "conditional rejects stray )"
assert re.match(pat, "(a") is None, "conditional rejects missing )"

print("conditional_group OK")
