# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "one_shot_equals_object_path"
# subject = "hmac.digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.digest: the hmac.digest(...) one-shot fast path returns the same bytes as hmac.new(...).digest(), for both a string and a hashlib-constructor digestmod"""
import hmac
import hashlib

key = b"test_key_123"
msg = b"test message"

# String digest name.
obj_path = hmac.new(key, msg, digestmod="sha256").digest()
one_shot = hmac.digest(key, msg, digest="sha256")
assert one_shot == obj_path, "hmac.digest(str) == hmac.new().digest()"

# hashlib-constructor digestmod.
obj_path2 = hmac.new(key, msg, digestmod=hashlib.sha256).digest()
one_shot2 = hmac.digest(key, msg, digest=hashlib.sha256)
assert one_shot2 == obj_path2, "hmac.digest(ctor) == hmac.new().digest()"

print("one_shot_equals_object_path OK")
