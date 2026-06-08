# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "chain_from_iterable_flattens"
# subject = "itertools.chain.from_iterable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain.from_iterable: chain.from_iterable interleaves nested iterables (incl. per-char strings) and ends cleanly"""
import itertools

assert list(itertools.chain.from_iterable([[1, 2], [3, 4]])) == [1, 2, 3, 4], "from_iterable lists"
assert "".join(itertools.chain.from_iterable(["ABC", "DEF"])) == "ABCDEF", "from_iterable join"

print("chain_from_iterable_flattens OK")
