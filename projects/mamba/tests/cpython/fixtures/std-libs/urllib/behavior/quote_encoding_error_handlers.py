# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_encoding_error_handlers"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: errors=replace maps un-encodable chars to escaped '?' and errors=xmlcharrefreplace to escaped numeric character references under a narrow codec"""
from urllib.parse import quote

assert quote("\u6f22\u5b57", encoding="latin-1", errors="replace") == \
    "%3F%3F", "errors=replace -> '?'"
assert quote("\u6f22\u5b57", encoding="latin-1", errors="xmlcharrefreplace") \
    == "%26%2328450%3B%26%2323383%3B", "errors=xmlcharrefreplace"

print("quote_encoding_error_handlers OK")
