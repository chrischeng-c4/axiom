# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "top_level_generator_expression_contract"
# subject = "generator expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""generator expression: module-level expressions have generator object semantics."""

import copy


g = (i for i in [1])
assert type(g).__name__ == "generator"
assert iter(g) is g
first_value = next(g)
assert first_value == 1

g = (i for i in [1])
try:
    copy.copy(g)
    raise AssertionError("copy.copy must reject generator expressions")
except TypeError:
    pass

g = (i for i in [1])
try:
    copy.deepcopy(g)
    raise AssertionError("copy.deepcopy must reject generator expressions")
except TypeError:
    pass

seen = 0
first = next((seen := i) for i in [7])
assert first == 7
assert seen == 7

print("top_level_generator_expression_contract OK")
