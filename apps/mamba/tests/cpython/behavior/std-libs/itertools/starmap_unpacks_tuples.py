# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "starmap_unpacks_tuples"
# subject = "itertools.starmap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.starmap: starmap applies the function to each argument tuple (unpacked), for user lambdas and builtins (pow, divmod)"""
import itertools

result = list(itertools.starmap(lambda a, b: a + b, [(1, 2), (3, 4), (5, 6)]))
assert result == [3, 7, 11], f"starmap = {result!r}"
assert list(itertools.starmap(pow, [(2, 3), (3, 2), (10, 2)])) == [8, 9, 100], "starmap pow"
assert list(itertools.starmap(divmod, [(10, 3), (15, 4)])) == [(3, 1), (3, 3)], "starmap divmod"
assert list(itertools.starmap(lambda a, b, c: a * b + c, [(1, 2, 3), (4, 5, 6)])) == [5, 26], "starmap 3-arg"

print("starmap_unpacks_tuples OK")
