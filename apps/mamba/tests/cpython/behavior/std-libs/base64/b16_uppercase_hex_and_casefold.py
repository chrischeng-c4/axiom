# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b16_uppercase_hex_and_casefold"
# subject = "base64.b16encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b16encode: b16encode emits uppercase hex (b'ABCDEF'); b16decode with casefold=True accepts lowercase input"""
import base64

assert base64.b16encode(b"\xab\xcd\xef") == b"ABCDEF", "b16 uppercase"
assert base64.b16decode(b"abcdef", casefold=True) == b"\xab\xcd\xef", "b16 casefold"
print("b16_uppercase_hex_and_casefold OK")
