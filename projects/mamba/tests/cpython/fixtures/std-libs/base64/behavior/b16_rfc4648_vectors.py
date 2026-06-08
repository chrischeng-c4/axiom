# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b16_rfc4648_vectors"
# subject = "base64.b16encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b16encode: the published RFC 4648 base16 vectors for '', 'f', 'fo', 'foo', 'foob', 'fooba', 'foobar' encode to their canonical uppercase-hex outputs"""
import base64

for _data, _expected in [
    (b"", b""),
    (b"f", b"66"),
    (b"fo", b"666F"),
    (b"foo", b"666F6F"),
    (b"foob", b"666F6F62"),
    (b"fooba", b"666F6F6261"),
    (b"foobar", b"666F6F626172"),
]:
    assert base64.b16encode(_data) == _expected, (_data, base64.b16encode(_data))
print("b16_rfc4648_vectors OK")
