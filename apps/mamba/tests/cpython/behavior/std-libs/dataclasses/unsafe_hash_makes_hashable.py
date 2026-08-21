# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "unsafe_hash_makes_hashable"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass(unsafe_hash=True) synthesizes a field-based __hash__ so equal instances hash equal and can be used as set members"""
import dataclasses


@dataclasses.dataclass(unsafe_hash=True)
class Hashable:
    key: str


hb = Hashable("abc")
assert hash(hb) == hash(Hashable("abc")), "unsafe_hash gives consistent hash"
s = {hb}
assert Hashable("abc") in s, "hashable can be in set"

print("unsafe_hash_makes_hashable OK")
