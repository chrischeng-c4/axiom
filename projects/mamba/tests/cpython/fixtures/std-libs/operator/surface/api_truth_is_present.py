# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_truth_is_present"
# subject = "operator.truth"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.truth: api_truth_is_present (surface)."""
import operator

assert hasattr(operator, "truth")
print("api_truth_is_present OK")
