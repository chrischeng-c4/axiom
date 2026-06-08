# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha512_digest_sizes"
# subject = "hashlib.sha512"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha512: sha512 has digest_size 64 bytes and a 128-char hexdigest"""
import hashlib

_h512 = hashlib.sha512(b"abc")
assert _h512.digest_size == 64, f"sha512 digest_size = {_h512.digest_size!r}"
assert len(_h512.hexdigest()) == 128, f"sha512 hexdigest len = {len(_h512.hexdigest())!r}"

print("sha512_digest_sizes OK")
