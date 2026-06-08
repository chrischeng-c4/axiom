# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "digits_is_str"
# subject = "string.digits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.digits: digits_is_str (surface)."""
import string

assert type(string.digits).__name__ == "str"
print("digits_is_str OK")
