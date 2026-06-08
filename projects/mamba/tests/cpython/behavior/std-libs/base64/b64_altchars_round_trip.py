# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64_altchars_round_trip"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: b64encode/b64decode honor a custom altchars=b'-_' two-char alphabet and round-trip through it"""
import base64

_enc_alt = base64.b64encode(b"\xfb\xef", altchars=b"-_")
assert base64.b64decode(_enc_alt, altchars=b"-_") == b"\xfb\xef", "altchars round-trip"
print("b64_altchars_round_trip OK")
