# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "ascii_letters_is_str"
# subject = "string.ascii_letters"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.ascii_letters: ascii_letters_is_str (surface)."""
import string

assert type(string.ascii_letters).__name__ == "str"
print("ascii_letters_is_str OK")
