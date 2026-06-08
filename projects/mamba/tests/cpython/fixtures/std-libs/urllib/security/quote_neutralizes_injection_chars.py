# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "security"
# case = "quote_neutralizes_injection_chars"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: CRLF / control / delimiter chars in untrusted values are percent-escaped by quote/quote_plus/urlencode so they cannot break out of a URL field or inject a header line"""
from urllib.parse import quote, quote_plus, urlencode

hostile = "value\r\nSet-Cookie: evil=1"
escaped = quote(hostile, safe="")
assert "\r" not in escaped and "\n" not in escaped, escaped
assert "%0D%0A" in escaped, escaped
assert quote_plus("a&b=c") == "a%26b%3Dc", quote_plus("a&b=c")
qs = urlencode({"redirect": "http://evil/?x=1&y=2"})
assert "&" not in qs.split("=", 1)[1], qs

print("quote_neutralizes_injection_chars OK")
