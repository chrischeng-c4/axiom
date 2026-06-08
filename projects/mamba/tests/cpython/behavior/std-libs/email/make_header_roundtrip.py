# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "make_header_roundtrip"
# subject = "email.header.make_header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test__encoded_words.py"
# status = "filled"
# ///
"""email.header.make_header: make_header(decode_header(...)) reassembles decoded chunks back to text; a full encode->decode->make_header round-trip recovers the source string (Grüße)"""
from email.header import Header, decode_header, make_header

# make_header reassembles decoded chunks back into the original text.
assert str(make_header(decode_header("=?utf-8?b?Y2HDsWE=?="))) == "ca\xf1a", str(
    make_header(decode_header("=?utf-8?b?Y2HDsWE=?="))
)

# Round-trip: encode then decode recovers the source string.
src = "Gr\xfc\xdfe"
recovered = str(make_header(decode_header(Header(src, charset="utf-8").encode())))
assert recovered == src, recovered

print("make_header_roundtrip OK")
