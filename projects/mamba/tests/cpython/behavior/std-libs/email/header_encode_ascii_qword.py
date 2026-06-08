# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "header_encode_ascii_qword"
# subject = "email.header.Header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test__encoded_words.py"
# status = "filled"
# ///
"""email.header.Header: a short ascii-safe utf-8 value encodes to a q-encoded word (=?utf-8?q?foo?=); a pure-ascii header with no charset stays unencoded"""
from email.header import Header

# A short ascii-safe utf-8 value is emitted as a q-encoded word.
assert Header("foo", charset="utf-8").encode() == "=?utf-8?q?foo?=", Header(
    "foo", charset="utf-8"
).encode()

# A pure-ascii header with no charset stays unencoded.
assert Header("plain ascii").encode() == "plain ascii", Header("plain ascii").encode()

print("header_encode_ascii_qword OK")
