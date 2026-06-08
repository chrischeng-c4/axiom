# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "mirrored_flags_bracket_chars"
# subject = "unicodedata.mirrored"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.mirrored: mirrored is 0 for 'A' and 1 for the open parenthesis '(' (a bidi-mirrored bracket)"""
import unicodedata

assert unicodedata.mirrored("A") == 0, f"A not mirrored = {unicodedata.mirrored('A')!r}"
assert unicodedata.mirrored("(") == 1, f"( is mirrored = {unicodedata.mirrored('(')!r}"

print("mirrored_flags_bracket_chars OK")
