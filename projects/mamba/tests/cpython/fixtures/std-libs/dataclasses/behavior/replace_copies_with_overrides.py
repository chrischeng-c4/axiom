# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "replace_copies_with_overrides"
# subject = "dataclasses.replace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.replace: replace() returns a new instance with only the named fields overridden, leaving the original and unspecified fields untouched"""
import dataclasses


@dataclasses.dataclass
class Record:
    a: int
    b: str
    c: float


orig = Record(1, "x", 3.14)
new = dataclasses.replace(orig, a=99)
assert orig.a == 1, "original a unchanged"
assert new.a == 99, f"replaced a = {new.a!r}"
assert new.b == "x", f"unchanged b = {new.b!r}"
assert new.c == 3.14, f"unchanged c = {new.c!r}"

print("replace_copies_with_overrides OK")
