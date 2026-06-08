# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "randbelow_in_range"
# subject = "secrets.randbelow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.randbelow: randbelow(n) returns an int in range(n) for n>=2 across repeated draws; randbelow(1) is always 0"""
import secrets

for _hi in range(2, 10):
    for _draw in range(6):
        _rb = secrets.randbelow(_hi)
        assert isinstance(_rb, int), f"randbelow type = {type(_rb)!r}"
        assert _rb in range(_hi), f"randbelow({_hi}) out of range: {_rb}"

# randbelow(1) has exactly one valid value below 1.
for _draw in range(5):
    assert secrets.randbelow(1) == 0, "randbelow(1) must be 0"

print("randbelow_in_range OK")
