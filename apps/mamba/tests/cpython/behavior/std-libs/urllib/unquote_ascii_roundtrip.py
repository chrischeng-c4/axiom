# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_ascii_roundtrip"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: every ASCII char survives an uppercase-hex escape -> unquote / unquote_plus round-trip back to the original char"""
from urllib.parse import unquote, unquote_plus

def hexescape(ch):
    h = hex(ord(ch))[2:].upper()
    return "%" + (h if len(h) == 2 else "0" + h)

for n in range(128):
    esc = hexescape(chr(n))
    assert unquote(esc) == chr(n), f"unquote {esc}"
    assert unquote_plus(esc) == chr(n), f"unquote_plus {esc}"

print("unquote_ascii_roundtrip OK")
