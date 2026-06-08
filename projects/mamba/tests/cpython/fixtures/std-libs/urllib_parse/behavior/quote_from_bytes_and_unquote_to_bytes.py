# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "quote_from_bytes_and_unquote_to_bytes"
# subject = "urllib.parse.quote_from_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.quote_from_bytes: quote_from_bytes percent-encodes raw bytes (space->%20, control->%01) and unquote_to_bytes is its inverse, returning raw bytes without any text decode"""
from urllib.parse import quote_from_bytes, unquote_to_bytes

assert quote_from_bytes(b"archaeological arcana") == "archaeological%20arcana"
assert quote_from_bytes(b"") == ""
assert quote_from_bytes(b"z\x01/ ") == "z%01/%20"

assert unquote_to_bytes("abc%20def") == b"abc def"
assert unquote_to_bytes("") == b""
assert unquote_to_bytes(quote_from_bytes(b"\xff\x00\x7f")) == b"\xff\x00\x7f"

print("quote_from_bytes_and_unquote_to_bytes OK")
