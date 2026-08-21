# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "name_default_for_unnamed_char"
# subject = "unicodedata.name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.name: name() returns the supplied default for an unnamed control character (NUL) instead of raising"""
import unicodedata

_n = unicodedata.name(chr(0), "NULL")  # NUL has no Unicode name
assert _n == "NULL", f"name NUL default = {_n!r}"

print("name_default_for_unnamed_char OK")
