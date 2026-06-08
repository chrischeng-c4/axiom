# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "decode_header_splits_chunks"
# subject = "email.header.decode_header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test__encoded_words.py"
# status = "filled"
# ///
"""email.header.decode_header: decode_header splits an encoded word into (bytes, charset) chunks; a word mixed with trailing text reports both, the unencoded run carrying a None charset"""
from email.header import decode_header

# decode_header splits an encoded word into (bytes, charset) chunks.
dec = decode_header("=?utf-8?q?foo?=")
assert dec == [(b"foo", "utf-8")], dec

# A header mixing an encoded word and trailing text reports both chunks,
# with a None charset for the unencoded run.
mixed = decode_header("=?utf-8?b?Zm9v?= plain")
assert mixed == [(b"foo", "utf-8"), (b" plain", None)], mixed

print("decode_header_splits_chunks OK")
