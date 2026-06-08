# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b32_rfc4648_vectors"
# subject = "base64.b32encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b32encode: the published RFC 4648 base32 vectors for '', 'f', 'fo', 'foo', 'foob', 'fooba', 'foobar' encode to their canonical outputs"""
import base64

for _data, _expected in [
    (b"", b""),
    (b"f", b"MY======"),
    (b"fo", b"MZXQ===="),
    (b"foo", b"MZXW6==="),
    (b"foob", b"MZXW6YQ="),
    (b"fooba", b"MZXW6YTB"),
    (b"foobar", b"MZXW6YTBOI======"),
]:
    assert base64.b32encode(_data) == _expected, (_data, base64.b32encode(_data))
print("b32_rfc4648_vectors OK")
