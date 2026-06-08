# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "cycle_repeats_sequence"
# subject = "itertools.cycle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.cycle: cycle(seq) repeats the source endlessly; the first 2*len pulls reproduce the sequence twice"""
import itertools

cy = itertools.cycle([1, 2, 3])
got = [next(cy) for _ in range(6)]
assert got == [1, 2, 3, 1, 2, 3], f"cycle = {got!r}"

print("cycle_repeats_sequence OK")
