# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_membership"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string: membership probes hold: 'a'/'Z' in ascii_letters, 'a'/'F' in hexdigits, ' ' and newline in whitespace, '!' in punctuation"""
import string

assert "a" in string.ascii_letters and "Z" in string.ascii_letters, "letters membership"
assert "a" in string.hexdigits and "F" in string.hexdigits, "hexdigits membership"
assert " " in string.whitespace and "\n" in string.whitespace, "whitespace membership"
assert "!" in string.punctuation, "punctuation membership"
print("constant_membership OK")
