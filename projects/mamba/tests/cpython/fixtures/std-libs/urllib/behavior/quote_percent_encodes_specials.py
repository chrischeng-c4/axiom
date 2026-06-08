# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_percent_encodes_specials"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote percent-encodes space and reserved chars, leaves the unreserved set and the default safe '/' alone, and honors safe=''"""
from urllib.parse import quote

assert quote("hello world") == "hello%20world", repr(quote("hello world"))
assert quote("a/b?c=d") == "a/b%3Fc%3Dd", repr(quote("a/b?c=d"))
assert quote("a/b?c=d", safe="") == "a%2Fb%3Fc%3Dd", repr(quote("a/b?c=d", safe=""))
assert quote("safe-._~") == "safe-._~", repr(quote("safe-._~"))

print("quote_percent_encodes_specials OK")
