# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_zip_floats"
# subject = "generator expression of floats feeding zip()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float generator expression feeding zip() must pair the correct float values."""

floats = (i + 0.5 for i in range(3))
labels = ["a", "b", "c"]
paired = list(zip(labels, floats))
assert paired == [("a", 0.5), ("b", 1.5), ("c", 2.5)], paired
assert all(isinstance(value, float) for _, value in paired), paired
print("genexpr_zip_floats OK")
