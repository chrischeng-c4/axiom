# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "product_cartesian_and_empty_rules"
# subject = "itertools.product"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.product: product is the cartesian product; product() is [()], any empty pool collapses to [], size is the product of lengths"""
import itertools

prod = list(itertools.product([1, 2], [3, 4]))
assert prod == [(1, 3), (1, 4), (2, 3), (2, 4)], f"product = {prod!r}"
assert list(itertools.product()) == [()], "product() = [()]"
assert list(itertools.product([])) == [], "product([]) empty"
assert list(itertools.product(range(2), range(0), range(3))) == [], "empty pool collapses"
assert len(list(itertools.product(*[range(7)] * 2))) == 49, "product size"

print("product_cartesian_and_empty_rules OK")
