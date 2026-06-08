# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "urlsafe_differs_from_standard"
# subject = "base64.urlsafe_b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.urlsafe_b64encode: urlsafe_b64encode replaces '+'/'/' with '-'/'_' so its output differs from standard b64encode for bytes that hit those positions, and still round-trips"""
import base64

# These bytes encode to a group containing '+' and '/' under the standard
# alphabet, so the urlsafe alphabet must differ.
_payload = b"\xfb\xff"
_std = base64.b64encode(_payload)
_url = base64.urlsafe_b64encode(_payload)
assert _std != _url, (_std, _url)
assert base64.urlsafe_b64decode(_url) == _payload, "urlsafe round-trip"
print("urlsafe_differs_from_standard OK")
