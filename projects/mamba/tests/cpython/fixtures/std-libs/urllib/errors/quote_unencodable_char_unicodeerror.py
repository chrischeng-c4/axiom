# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "quote_unencodable_char_unicodeerror"
# subject = "urllib.parse.quote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote_unencodable_char_unicodeerror (errors)."""
from urllib.parse import quote

_raised = False
try:
    quote('\u6f22', encoding='latin-1')
except UnicodeEncodeError:
    _raised = True
assert _raised, "quote_unencodable_char_unicodeerror: expected UnicodeEncodeError"
print("quote_unencodable_char_unicodeerror OK")
