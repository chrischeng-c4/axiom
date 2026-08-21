# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_float_accumulator_mean"
# subject = "mean computed from sum() over a float generator expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A mean computed by summing a float generator expression then dividing must give the correct float."""

samples = [1.0, 2.0, 3.0, 4.0]
total = sum(x for x in samples)
mean = total / len(samples)
assert isinstance(total, float), type(total)
assert isinstance(mean, float), type(mean)
assert total == 10.0, total
assert mean == 2.5, mean
print("genexpr_float_accumulator_mean OK")
