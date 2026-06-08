# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "product_repeat_multiplies_pools"
# subject = "itertools.product"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.product: product(*pools, repeat=k) equals repeating the pools k times, e.g. product('AB', repeat=2) == product('AB','AB')"""
import itertools

pp = list(itertools.product("AB", repeat=2))
assert pp == [("A", "A"), ("A", "B"), ("B", "A"), ("B", "B")], f"product repeat = {pp!r}"
assert list(itertools.product("AB", repeat=2)) == list(itertools.product("AB", "AB")), "repeat == args"

print("product_repeat_multiplies_pools OK")
