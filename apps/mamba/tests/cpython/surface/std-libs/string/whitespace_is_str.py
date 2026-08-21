# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "whitespace_is_str"
# subject = "string.whitespace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.whitespace: whitespace_is_str (surface)."""
import string

assert type(string.whitespace).__name__ == "str"
print("whitespace_is_str OK")
