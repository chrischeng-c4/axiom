# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64_padding_by_remainder"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: b64 padding follows the 3-byte group remainder: 1 byte -> 'YQ==', 2 bytes -> 'YWI=', 3 bytes -> 'YWJj' (no padding)"""
import base64

# 1 byte -> 2 base64 chars + 2 padding
assert base64.b64encode(b"a") == b"YQ==", base64.b64encode(b"a")
# 2 bytes -> 3 base64 chars + 1 padding
assert base64.b64encode(b"ab") == b"YWI=", base64.b64encode(b"ab")
# 3 bytes -> 4 base64 chars, no padding
assert base64.b64encode(b"abc") == b"YWJj", base64.b64encode(b"abc")
print("b64_padding_by_remainder OK")
