# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_control_chars_escaped"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: control chars 0..31, 127 and the gen-delim set are always %-escaped to their uppercase 2-digit hex by both quote and quote_plus"""
from urllib.parse import quote, quote_plus

def hexescape(ch):
    h = hex(ord(ch))[2:].upper()
    return "%" + (h if len(h) == 2 else "0" + h)

should_quote = "".join(chr(n) for n in range(32)) + '<>#%"{}|\\^[]`' + chr(127)
for ch in should_quote:
    assert quote(ch) == hexescape(ch), f"quote control {ch!r}"
    assert quote_plus(ch) == hexescape(ch), f"quote_plus control {ch!r}"

print("quote_control_chars_escaped OK")
