# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "octdigits_is_str"
# subject = "string.octdigits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.octdigits: octdigits_is_str (surface)."""
import string

assert type(string.octdigits).__name__ == "str"
print("octdigits_is_str OK")
