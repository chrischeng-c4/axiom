# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "hexdigits_is_str"
# subject = "string.hexdigits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.hexdigits: hexdigits_is_str (surface)."""
import string

assert type(string.hexdigits).__name__ == "str"
print("hexdigits_is_str OK")
