# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "name_no_name_raises"
# subject = "unicodedata.name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.name: name_no_name_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.name(chr(0))
except ValueError:
    _raised = True
assert _raised, "name_no_name_raises: expected ValueError"
print("name_no_name_raises OK")
