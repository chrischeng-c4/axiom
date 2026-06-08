# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_gt_is_present"
# subject = "operator.gt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.gt: api_gt_is_present (surface)."""
import operator

assert hasattr(operator, "gt")
print("api_gt_is_present OK")
