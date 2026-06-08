# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64decode_ignores_whitespace_by_default"
# subject = "base64.b64decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64decode: with validate=False (default) b64decode ignores embedded whitespace, decoding b'aGVsbG8=\\n' to b'hello'"""
import base64

assert base64.b64decode(b"aGVsbG8=\n") == b"hello", "decode ignores whitespace"
print("b64decode_ignores_whitespace_by_default OK")
