# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_bytes_not_all_zero"
# subject = "secrets.token_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_bytes: a 32-byte token_bytes draw is not all-zero (cryptographically random, not a stub)"""
import secrets

# A 32-byte cryptographic draw being all-zero has probability 2**-256;
# any all-zero result signals a broken/stubbed RNG rather than a flake.
_b = secrets.token_bytes(32)
assert any(_byte != 0 for _byte in _b), f"token_bytes(32) is all zero: {_b!r}"

print("token_bytes_not_all_zero OK")
