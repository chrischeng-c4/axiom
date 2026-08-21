# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "ascii_lowercase_is_str"
# subject = "string.ascii_lowercase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.ascii_lowercase: ascii_lowercase_is_str (surface)."""
import string

assert type(string.ascii_lowercase).__name__ == "str"
print("ascii_lowercase_is_str OK")
