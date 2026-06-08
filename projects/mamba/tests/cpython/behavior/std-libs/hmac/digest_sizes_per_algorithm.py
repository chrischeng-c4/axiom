# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "digest_sizes_per_algorithm"
# subject = "hmac.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: HMAC digest length tracks the underlying hash: md5 -> 16 bytes, sha1 -> 20 bytes, sha256 -> 32 bytes, and differing algorithms over the same key/msg yield differing MACs"""
import hmac
import hashlib

key = b"key"
msg = b"message"

# Digest length tracks the underlying hash function.
md5_mac = hmac.new(b"key", b"data", digestmod=hashlib.md5).digest()
assert len(md5_mac) == 16, f"HMAC-MD5 len = {len(md5_mac)!r}"

sha1_mac = hmac.new(key, msg, digestmod=hashlib.sha1).digest()
assert len(sha1_mac) == 20, f"HMAC-SHA1 len = {len(sha1_mac)!r}"

sha256_mac = hmac.new(key, msg, digestmod=hashlib.sha256).digest()
assert len(sha256_mac) == 32, f"HMAC-SHA256 len = {len(sha256_mac)!r}"

# Different digestmod over the same key/msg yields different MACs.
assert sha256_mac != sha1_mac, "different digestmod = different MAC"

print("digest_sizes_per_algorithm OK")
