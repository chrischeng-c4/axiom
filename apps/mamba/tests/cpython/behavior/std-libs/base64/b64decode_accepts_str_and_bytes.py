# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64decode_accepts_str_and_bytes"
# subject = "base64.b64decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64decode: b64decode accepts both a bytes and a str payload, decoding 'aGVsbG8=' to b'hello' either way"""
import base64

assert base64.b64decode(b"aGVsbG8=") == b"hello", "bytes payload"
assert base64.b64decode("aGVsbG8=") == b"hello", "str payload"
print("b64decode_accepts_str_and_bytes OK")
