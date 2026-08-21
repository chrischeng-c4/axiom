# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "float_sum_comparison"
# subject = "IEEE-754 float addition then comparison (0.1 + 0.2 > 0.3)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""0.1 + 0.2 is 0.30000000000000004, so it compares strictly greater than 0.3."""
total = 0.1 + 0.2
assert total > 0.3, total
assert total != 0.3, total
assert isinstance(total, float), type(total)
print("float_sum_comparison OK")
