# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64_rfc4648_vectors"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b64encode: the published RFC 4648 base64 vectors for '', 'f', 'fo', 'foo', 'foob', 'fooba', 'foobar' encode to their canonical outputs"""
import base64

for _data, _expected in [
    (b"", b""),
    (b"f", b"Zg=="),
    (b"fo", b"Zm8="),
    (b"foo", b"Zm9v"),
    (b"foob", b"Zm9vYg=="),
    (b"fooba", b"Zm9vYmE="),
    (b"foobar", b"Zm9vYmFy"),
]:
    assert base64.b64encode(_data) == _expected, (_data, base64.b64encode(_data))
print("b64_rfc4648_vectors OK")
