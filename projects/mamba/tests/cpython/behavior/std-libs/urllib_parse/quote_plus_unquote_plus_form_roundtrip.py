# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "quote_plus_unquote_plus_form_roundtrip"
# subject = "urllib.parse.quote_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.quote_plus: quote_plus encodes spaces as '+' (no literal space survives) and unquote_plus restores the original form string, round-tripping 'key=value with spaces & special+chars'"""
from urllib.parse import quote_plus, unquote_plus

form = "key=value with spaces & special+chars"
encoded = quote_plus(form)
decoded = unquote_plus(encoded)
assert decoded == form, f"quote_plus round-trip = {decoded!r}"
assert " " not in encoded, f"no literal space in encoded = {encoded!r}"

print("quote_plus_unquote_plus_form_roundtrip OK")
