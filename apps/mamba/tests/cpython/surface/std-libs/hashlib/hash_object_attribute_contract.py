# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "hash_object_attribute_contract"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: a sha256 hash object exposes digest_size==32, block_size==64, name=='sha256', a 64-char str hexdigest, and a 32-byte digest"""
import hashlib

_h256 = hashlib.sha256(b"hello")
assert hasattr(_h256, "digest_size"), "has digest_size"
assert hasattr(_h256, "block_size"), "has block_size"
assert hasattr(_h256, "name"), "has name"
assert _h256.digest_size == 32, f"sha256 digest_size = {_h256.digest_size!r}"
assert _h256.block_size == 64, f"sha256 block_size = {_h256.block_size!r}"
assert _h256.name == "sha256", f"sha256 .name = {_h256.name!r}"

_hex = _h256.hexdigest()
assert isinstance(_hex, str), f"hexdigest type = {type(_hex)!r}"
assert len(_hex) == 64, f"sha256 hexdigest length = {len(_hex)!r}"

_dig = hashlib.sha256(b"hello").digest()
assert isinstance(_dig, bytes), f"digest type = {type(_dig)!r}"
assert len(_dig) == 32, f"sha256 digest length = {len(_dig)!r}"

print("hash_object_attribute_contract OK")
