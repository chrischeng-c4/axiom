# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha3_256_known_vectors"
# subject = "hashlib.sha3_256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha3_256: sha3_256 NIST FIPS-202 known answers: sha3_256(b'abc') and sha3_256(b'') match their reference hexdigests"""
import hashlib

assert hashlib.sha3_256(b"abc").hexdigest() == \
    "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532", "sha3_256('abc')"
assert hashlib.sha3_256(b"").hexdigest() == \
    "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a", "sha3_256('')"

print("sha3_256_known_vectors OK")
