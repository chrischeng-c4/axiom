# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "combinations_with_replacement_multisets"
# subject = "itertools.combinations_with_replacement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.combinations_with_replacement: combinations_with_replacement allows repeats; r may exceed n; counts follow C(n+r-1, r); r==0 is [()]"""
import itertools

cwr = list(itertools.combinations_with_replacement([1, 2], 2))
assert cwr == [(1, 1), (1, 2), (2, 2)], f"cwr = {cwr!r}"
assert list(itertools.combinations_with_replacement([1], 3)) == [(1, 1, 1)], "r may exceed n"
assert list(itertools.combinations_with_replacement([1, 2], 0)) == [()], "r==0 is [()]"
assert list(itertools.combinations_with_replacement([], 2)) == [], "empty input r>=1"
assert len(list(itertools.combinations_with_replacement([1, 2, 3, 4], 3))) == 20, "C(n+r-1, r) count"

print("combinations_with_replacement_multisets OK")
