# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_ascii_letters_is_present"
# subject = "string.ascii_letters"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.ascii_letters: api_ascii_letters_is_present (surface)."""
import string

assert hasattr(string, "ascii_letters")
print("api_ascii_letters_is_present OK")
