# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "lookup_unknown_name_raises"
# subject = "unicodedata.lookup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.lookup: lookup_unknown_name_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.lookup("NO_SUCH_CHARACTER_NAME_XYZZY")
except KeyError:
    _raised = True
assert _raised, "lookup_unknown_name_raises: expected KeyError"
print("lookup_unknown_name_raises OK")
