# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "header_encode_non_ascii_base64"
# subject = "email.header.Header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test__encoded_words.py"
# status = "filled"
# ///
"""email.header.Header: non-ascii content (caña) encodes to a base64 encoded word (=?utf-8?b?Y2HDsWE=?=)"""
from email.header import Header

# Non-ascii content is base64-encoded as an encoded word.
ena = Header("ca\xf1a", charset="utf-8").encode()
assert ena == "=?utf-8?b?Y2HDsWE=?=", ena

print("header_encode_non_ascii_base64 OK")
