# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "category_multi_char_raises"
# subject = "unicodedata.category"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.category: category_multi_char_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.category("xx")
except TypeError:
    _raised = True
assert _raised, "category_multi_char_raises: expected TypeError"
print("category_multi_char_raises OK")
