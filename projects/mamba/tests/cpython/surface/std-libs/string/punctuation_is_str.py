# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "punctuation_is_str"
# subject = "string.punctuation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.punctuation: punctuation_is_str (surface)."""
import string

assert type(string.punctuation).__name__ == "str"
print("punctuation_is_str OK")
