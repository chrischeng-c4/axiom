# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha3_256_block_size_sponge_rate"
# subject = "hashlib.sha3_256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha3_256: the sponge rate gives sha3_256 a non-power-of-two block_size of 136, unlike SHA-2; .name is the canonical 'sha3_512'"""
import hashlib

assert hashlib.sha3_256().block_size == 136, \
    f"sha3_256 block_size = {hashlib.sha3_256().block_size!r}"
assert hashlib.sha3_512().name == "sha3_512", "sha3_512 .name"

print("sha3_256_block_size_sponge_rate OK")
