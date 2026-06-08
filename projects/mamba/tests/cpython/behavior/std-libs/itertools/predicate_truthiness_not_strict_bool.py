# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "predicate_truthiness_not_strict_bool"
# subject = "itertools.takewhile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.takewhile: takewhile/dropwhile/filterfalse use Python truthiness (int/str/object), not strict bool, for the predicate result"""
import itertools

def isodd(x):
    return x % 2

def echo(x):
    return x

# int-returning predicate uses Python truthiness, not strict bool
assert list(itertools.filterfalse(isodd, [1, 2, 3, 4, 5])) == [2, 4], "filterfalse int pred"
assert list(itertools.takewhile(isodd, [1, 3, 5, 4, 7])) == [1, 3, 5], "takewhile int pred"
assert list(itertools.dropwhile(isodd, [1, 3, 5, 4, 7])) == [4, 7], "dropwhile int pred"

# str-returning predicate: truthy iff non-empty
assert list(itertools.takewhile(echo, ["a", "b", "", "c"])) == ["a", "b"], "takewhile str pred"
assert list(itertools.filterfalse(echo, ["a", "", "b", ""])) == ["", ""], "filterfalse str pred"

# bare-value predicate: truthy iff non-zero
assert list(itertools.takewhile(echo, [1, 2, 3, 0, 4])) == [1, 2, 3], "takewhile value pred"
assert list(itertools.dropwhile(echo, [0, 0, 1, 0, 2])) == [0, 0, 1, 0, 2], "dropwhile value pred"

print("predicate_truthiness_not_strict_bool OK")
