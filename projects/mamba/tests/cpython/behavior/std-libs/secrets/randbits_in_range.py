# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "randbits_in_range"
# subject = "secrets.randbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.randbits: randbits(k) returns an int in [0, 2**k) for k in 1,8,16,32,64; randbits(0) is 0"""
import secrets

for _bits in [1, 8, 16, 32, 64]:
    _max = 1 << _bits
    for _draw in range(5):
        _v = secrets.randbits(_bits)
        assert isinstance(_v, int), f"randbits({_bits}) type = {type(_v)!r}"
        assert 0 <= _v < _max, f"randbits({_bits}) out of range: {_v}"

# randbits(0) yields the only value below 2**0 == 1.
assert secrets.randbits(0) == 0, "randbits(0) must be 0"

print("randbits_in_range OK")
