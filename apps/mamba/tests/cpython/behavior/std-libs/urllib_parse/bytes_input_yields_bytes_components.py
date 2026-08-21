# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "bytes_input_yields_bytes_components"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: bytes input produces a ParseResult whose every component is bytes: urlparse(b'x-newscheme://foo.com/stuff?q#f') equals the all-bytes 6-tuple"""
from urllib.parse import urlparse

b = urlparse(b"x-newscheme://foo.com/stuff?q#f")
assert b == (b"x-newscheme", b"foo.com", b"/stuff", b"", b"q", b"f"), f"bytes = {b!r}"
assert b.scheme == b"x-newscheme", f"bytes scheme = {b.scheme!r}"
assert b.netloc == b"foo.com", f"bytes netloc = {b.netloc!r}"

print("bytes_input_yields_bytes_components OK")
