# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_hash_uses_firstlineno"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: code hashing follows equality: an equal replace() hashes the same, while changing co_firstlineno changes the hash"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert hash(co.replace()) == hash(co), "equal code objects hash the same"
shifted = co.replace(co_firstlineno=co.co_firstlineno + 1)
assert hash(shifted) != hash(co), "co_firstlineno is part of the hash"

print("code_hash_uses_firstlineno OK")
