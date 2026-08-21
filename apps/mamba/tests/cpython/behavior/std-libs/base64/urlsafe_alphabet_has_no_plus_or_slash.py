# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "urlsafe_alphabet_has_no_plus_or_slash"
# subject = "base64.urlsafe_b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.urlsafe_b64encode: urlsafe_b64encode never emits '+' or '/' even when round-tripping every byte value 0..255"""
import base64

_all_bytes = bytes(range(256))
_url = base64.urlsafe_b64encode(_all_bytes)
assert b"+" not in _url, "urlsafe no +"
assert b"/" not in _url, "urlsafe no /"
assert base64.urlsafe_b64decode(_url) == _all_bytes, "urlsafe round-trip"
print("urlsafe_alphabet_has_no_plus_or_slash OK")
