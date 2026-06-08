# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_quotes_keys_and_values"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: keys and values are quote_plus-encoded: spaces become '+', reserved '&'/'=' are %-escaped, and bytes values are escaped byte-for-byte"""
from urllib.parse import urlencode

def hexescape(ch):
    h = hex(ord(ch))[2:].upper()
    return "%" + (h if len(h) == 2 else "0" + h)

assert urlencode({"&": "="}) == hexescape("&") + "=" + hexescape("="), \
    "reserved chars escaped"
assert urlencode({"key name": "A bunch of pluses"}) == \
    "key+name=A+bunch+of+pluses", "spaces -> +"
assert urlencode(((b"\xa0$", b"\xc1$"),)) == "%A0%24=%C1%24", "bytes value"

print("urlencode_quotes_keys_and_values OK")
