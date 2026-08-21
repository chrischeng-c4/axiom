# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "class_constructor_digestmod_spellings"
# subject = "hmac.HMAC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC: hmac.HMAC accepts a string name, a hashlib-module constructor, or a keyword-only digestmod, all equivalent to hmac.new; the object reports digest_size=32, block_size=64, name='hmac-sha256'"""
import hmac
import hashlib

key = b"my secret key"
msg = b"compute the hash of this text!"

# Positional hashlib-module digestmod.
h_mod = hmac.HMAC(key, msg, hashlib.sha256)
assert isinstance(h_mod, hmac.HMAC), f"type = {type(h_mod)!r}"
assert len(h_mod.digest()) == 32, "module digestmod -> sha256 digest"

# A string digest name resolves to the same algorithm.
h_str = hmac.HMAC(key, msg, digestmod="sha256")
assert h_str.digest() == h_mod.digest(), "string name == module digestmod"

# hmac.new is equivalent to the class constructor.
h_new = hmac.new(key, msg, digestmod=hashlib.sha256)
assert h_new.digest() == h_mod.digest(), "hmac.new == HMAC(...)"

# Reported metadata for an HMAC object.
h = hmac.new(key, digestmod="sha256")
assert h.digest_size == 32, f"digest_size = {h.digest_size!r}"
assert h.block_size == 64, f"block_size = {h.block_size!r}"
assert h.name == "hmac-sha256", f"name = {h.name!r}"

# msg may be omitted at construction and supplied via update() later.
deferred = hmac.HMAC(key, digestmod="sha256")
deferred.update(msg)
assert deferred.digest() == h_mod.digest(), "deferred update == construct-time msg"

print("class_constructor_digestmod_spellings OK")
