# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_values_exact"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string: the module constants have their exact documented byte values: whitespace, ascii_lowercase/uppercase, digits, hexdigits, octdigits, punctuation"""
import string

assert string.whitespace == " \t\n\r\x0b\x0c", f"whitespace = {string.whitespace!r}"
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz", "ascii_lowercase"
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "ascii_uppercase"
assert string.digits == "0123456789", f"digits = {string.digits!r}"
assert string.octdigits == "01234567", f"octdigits = {string.octdigits!r}"
assert string.punctuation == "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~", "punctuation value"
print("constant_values_exact OK")
