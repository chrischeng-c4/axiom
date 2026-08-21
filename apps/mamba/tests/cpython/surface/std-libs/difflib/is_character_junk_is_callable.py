# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "surface"
# case = "is_character_junk_is_callable"
# subject = "difflib.IS_CHARACTER_JUNK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.IS_CHARACTER_JUNK: is_character_junk_is_callable (surface)."""
import difflib

assert callable(difflib.IS_CHARACTER_JUNK)
print("is_character_junk_is_callable OK")
