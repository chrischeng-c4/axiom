# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_char_classes"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string: each constant contains only its own character class: ascii_lowercase all islower, ascii_uppercase all isupper, digits all isdigit"""
import string

assert all(c.islower() for c in string.ascii_lowercase), "all lowercase"
assert all(c.isupper() for c in string.ascii_uppercase), "all uppercase"
assert all(c.isdigit() for c in string.digits), "all digits"
print("constant_char_classes OK")
