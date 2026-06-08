# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "constant_compositions"
# subject = "string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string: the composed constants equal their parts: ascii_letters == lower+upper, hexdigits == digits+'abcdefABCDEF', printable == digits+lower+upper+punctuation+whitespace"""
import string

assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase, "ascii_letters composition"
assert string.hexdigits == string.digits + "abcdefABCDEF", "hexdigits composition"
assert string.printable == (
    string.digits
    + string.ascii_lowercase
    + string.ascii_uppercase
    + string.punctuation
    + string.whitespace
), "printable composition"
print("constant_compositions OK")
