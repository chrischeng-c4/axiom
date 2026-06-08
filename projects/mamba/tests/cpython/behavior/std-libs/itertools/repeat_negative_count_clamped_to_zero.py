# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "repeat_negative_count_clamped_to_zero"
# subject = "itertools.repeat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.repeat: repeat clamps a negative times to zero: empty output and repr shows times=0 (positional and keyword)"""
import itertools

assert repr(itertools.repeat("a", -1)) == "repeat('a', 0)", repr(itertools.repeat("a", -1))
assert repr(itertools.repeat("a", times=-2)) == "repeat('a', 0)", "repeat kw repr"
assert list(itertools.repeat("a", -1)) == [], "repeat negative is empty"

print("repeat_negative_count_clamped_to_zero OK")
