# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "ascii_uppercase_is_str"
# subject = "string.ascii_uppercase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.ascii_uppercase: ascii_uppercase_is_str (surface)."""
import string

assert type(string.ascii_uppercase).__name__ == "str"
print("ascii_uppercase_is_str OK")
