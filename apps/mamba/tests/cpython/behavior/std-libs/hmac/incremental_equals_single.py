# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "incremental_equals_single"
# subject = "hmac.HMAC.update"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC.update: feeding the message in chunks via update() yields the same digest as supplying it whole at construction"""
import hmac
import hashlib

key = b"secret_key"

# Whole message at construction.
single = hmac.new(key, b"hello world", digestmod=hashlib.sha256).digest()

# Same message fed in chunks via update().
inc = hmac.new(key, digestmod=hashlib.sha256)
inc.update(b"hello ")
inc.update(b"world")
assert inc.digest() == single, "incremental update == single-shot digest"

print("incremental_equals_single OK")
