# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "surface"
# case = "deepcopy_is_callable"
# subject = "copy.deepcopy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""copy.deepcopy: deepcopy_is_callable (surface)."""
import copy

assert callable(copy.deepcopy)
print("deepcopy_is_callable OK")
