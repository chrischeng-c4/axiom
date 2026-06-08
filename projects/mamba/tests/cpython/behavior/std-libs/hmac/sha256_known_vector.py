# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "sha256_known_vector"
# subject = "hmac.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: HMAC-SHA256 of key='key' over the pangram matches the published RFC/NIST hexdigest f7bc83f4...2d1a3cd8"""
import hmac
import hashlib

# Published HMAC-SHA256 test vector: key="key", msg=the pangram.
key = b"key"
msg = b"The quick brown fox jumps over the lazy dog"
expected = "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
got = hmac.new(key, msg, digestmod=hashlib.sha256).hexdigest()
assert got == expected, f"HMAC-SHA256 vector = {got!r}"

print("sha256_known_vector OK")
