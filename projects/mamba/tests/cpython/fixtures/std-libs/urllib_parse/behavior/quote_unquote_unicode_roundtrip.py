# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "quote_unquote_unicode_roundtrip"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote(s, safe='') then unquote() round-trips a non-ASCII string containing accents and reserved chars back to the original"""
from urllib.parse import quote, unquote

special = "héllo wörld! <>&\""
quoted = quote(special, safe="")
unquoted = unquote(quoted)
assert unquoted == special, f"quote/unquote round-trip = {unquoted!r}"
assert "%" in quoted, f"non-ASCII was escaped = {quoted!r}"

print("quote_unquote_unicode_roundtrip OK")
