# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_letters_digits_disjoint"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string: the ascii_letters and digits character sets are disjoint (no char is both a letter and a digit)"""
import string

_letter_set = set(string.ascii_letters)
_digit_set = set(string.digits)
assert not (_letter_set & _digit_set), "letters and digits disjoint"
print("constant_letters_digits_disjoint OK")
