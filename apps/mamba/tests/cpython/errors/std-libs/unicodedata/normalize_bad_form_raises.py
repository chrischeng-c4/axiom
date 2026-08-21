# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "errors"
# case = "normalize_bad_form_raises"
# subject = "unicodedata.normalize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.normalize: normalize_bad_form_raises (errors)."""
import unicodedata

_raised = False
try:
    unicodedata.normalize("NO_SUCH_FORM", "abc")
except ValueError:
    _raised = True
assert _raised, "normalize_bad_form_raises: expected ValueError"
print("normalize_bad_form_raises OK")
