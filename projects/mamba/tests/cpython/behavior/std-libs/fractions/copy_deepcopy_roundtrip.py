# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "copy_deepcopy_roundtrip"
# subject = "fractions.Fraction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: copy.copy and copy.deepcopy return an equal Fraction (immutable value semantics)"""
import copy
from fractions import Fraction

f = Fraction(3, 4)
assert copy.copy(f) == f, "shallow copy equals original"
assert copy.deepcopy(f) == f, "deep copy equals original"

print("copy_deepcopy_roundtrip OK")
