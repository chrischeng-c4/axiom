# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "printable_is_str"
# subject = "string.printable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.printable: printable_is_str (surface)."""
import string

assert type(string.printable).__name__ == "str"
print("printable_is_str OK")
