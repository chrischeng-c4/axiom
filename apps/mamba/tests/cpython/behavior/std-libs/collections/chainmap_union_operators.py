# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "chainmap_union_operators"
# subject = "collections.ChainMap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: PEP 584 | merges the other mapping into a copy of the first map preserving trailing maps; |= mutates in place; ChainMap | dict and dict | ChainMap fold correctly; result type follows the left operand"""
from collections import ChainMap

cm1 = ChainMap(dict(a=1, b=2), dict(c=3, d=4))
cm2 = ChainMap(dict(a=10, e=5), dict(b=20, d=4))

merged = cm1 | cm2
assert merged.maps == [cm1.maps[0] | dict(cm2), *cm1.maps[1:]], "| merges into a copy of the front map"

cm1_copy = ChainMap(dict(a=1, b=2), dict(c=3, d=4))
cm1_copy |= cm2
assert cm1_copy == merged, "|= matches |"

d = dict(a=10, c=30)
assert (cm2 | d).maps == [cm2.maps[0] | d, *cm2.maps[1:]], "ChainMap | dict folds into the front map"
assert (d | cm2).maps == [d | dict(cm2)], "dict | ChainMap yields a single-map ChainMap"

try:
    ChainMap() | [("c", 3)]
    raise AssertionError("expected TypeError for a non-mapping operand")
except TypeError:
    pass

class Sub(ChainMap):
    pass

assert type(ChainMap() | ChainMap()) is ChainMap, "base | base -> base"
assert type(ChainMap() | Sub()) is ChainMap, "base | sub -> base"
assert type(Sub() | ChainMap()) is Sub, "sub | base -> sub (result type follows the left)"

print("chainmap_union_operators OK")
