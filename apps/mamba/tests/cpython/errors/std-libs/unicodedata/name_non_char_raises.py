# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "name_non_char_raises"
# subject = "unicodedata.name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.name: name_non_char_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.name(123)
except TypeError:
    _raised = True
assert _raised, "name_non_char_raises: expected TypeError"
print("name_non_char_raises OK")
