# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "security"
# case = "unquote_no_decode_explosion"
# subject = "urllib.parse.unquote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote: untrusted input full of malformed and partial percent escapes (%, %x, %zz, trailing %) is decoded without raising and without consuming valid trailing data, so a hostile query string cannot crash the parser"""
from urllib.parse import unquote

for hostile in ("%", "%x", "%zz", "abc%", "%%%", "a%2", "%2"):
    out = unquote(hostile)
    assert isinstance(out, str), (hostile, out)
# A valid escape is decoded; an adjacent malformed one is left verbatim.
assert unquote("%41%zz") == "A%zz", unquote("%41%zz")
assert unquote("ok%20done%") == "ok done%", unquote("ok%20done%")

print("unquote_no_decode_explosion OK")
