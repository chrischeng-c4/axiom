# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_unreserved_survives"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: the RFC 3986 unreserved set (alnum plus _.-~) is never escaped by either quote or quote_plus"""
from urllib.parse import quote, quote_plus

unreserved = (
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    "abcdefghijklmnopqrstuvwxyz"
    "0123456789_.-~"
)
assert quote(unreserved) == unreserved, "unreserved must survive quote"
assert quote_plus(unreserved) == unreserved, "unreserved must survive quote_plus"

print("quote_unreserved_survives OK")
