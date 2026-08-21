# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "key_and_message_sensitivity"
# subject = "hmac.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: changing the key, or the message, changes the MAC; an empty message and an empty key are both valid and produce a full-length digest"""
import hmac
import hashlib

key = b"secret_key"
msg = b"hello world"
base = hmac.new(key, msg, digestmod=hashlib.sha256).digest()

# A different key changes the MAC.
assert hmac.new(b"other_key", msg, digestmod=hashlib.sha256).digest() != base, \
    "different key = different MAC"

# A different message changes the MAC.
assert hmac.new(key, b"other message", digestmod=hashlib.sha256).digest() != base, \
    "different msg = different MAC"

# An empty message is valid and produces a full-length digest.
empty_msg = hmac.new(b"key", b"", digestmod=hashlib.sha256).hexdigest()
assert len(empty_msg) == 64, f"empty msg HMAC len = {len(empty_msg)!r}"
assert empty_msg != hmac.new(b"key", b"x", digestmod=hashlib.sha256).hexdigest(), \
    "empty vs non-empty differ"

# An empty key is valid and produces a full-length digest.
empty_key = hmac.new(b"", b"message", digestmod=hashlib.sha256).digest()
assert isinstance(empty_key, bytes), f"empty key HMAC = {type(empty_key)!r}"
assert len(empty_key) == 32, "empty key mac len"

print("key_and_message_sensitivity OK")
