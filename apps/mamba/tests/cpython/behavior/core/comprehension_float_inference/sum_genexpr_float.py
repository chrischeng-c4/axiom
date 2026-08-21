# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "comprehension_float_inference"
# dimension = "behavior"
# case = "sum_genexpr_float"
# subject = "sum of a generator expression of user-func floats"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() over a generator expression of user-func floats produces the correct float total."""


def ff(j):
    return j + 0.5


total = sum(ff(j) for j in range(4))
assert total == 0.5 + 1.5 + 2.5 + 3.5, total
assert total == 8.0, total
assert isinstance(total, float), (total, type(total))
print("sum_genexpr_float OK")
