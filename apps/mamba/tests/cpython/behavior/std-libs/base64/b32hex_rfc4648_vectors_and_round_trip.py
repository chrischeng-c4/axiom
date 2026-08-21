# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b32hex_rfc4648_vectors_and_round_trip"
# subject = "base64.b32hexencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_base64.py"
# status = "filled"
# ///
"""base64.b32hexencode: the RFC 4648 base32hex (extended-hex alphabet) vectors encode canonically and b32hexdecode reverses b32hexencode"""
import base64

for _data, _expected in [
    (b"", b""),
    (b"f", b"CO======"),
    (b"fo", b"CPNG===="),
    (b"foo", b"CPNMU==="),
    (b"foob", b"CPNMUOG="),
    (b"fooba", b"CPNMUOJ1"),
    (b"foobar", b"CPNMUOJ1E8======"),
]:
    assert base64.b32hexencode(_data) == _expected, (_data, base64.b32hexencode(_data))
assert base64.b32hexdecode(b"CPNMUOJ1") == b"fooba", "b32hex decode round-trip"
print("b32hex_rfc4648_vectors_and_round_trip OK")
