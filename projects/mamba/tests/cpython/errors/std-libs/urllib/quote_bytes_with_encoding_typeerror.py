# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "quote_bytes_with_encoding_typeerror"
# subject = "urllib.parse.quote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote_bytes_with_encoding_typeerror (errors)."""
from urllib.parse import quote

_raised = False
try:
    quote(b'\xa2\xd8', encoding='latin-1')
except TypeError:
    _raised = True
assert _raised, "quote_bytes_with_encoding_typeerror: expected TypeError"
print("quote_bytes_with_encoding_typeerror OK")
