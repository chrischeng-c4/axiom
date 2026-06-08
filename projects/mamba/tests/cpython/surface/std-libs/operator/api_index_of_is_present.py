# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_index_of_is_present"
# subject = "operator.indexOf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.indexOf: api_index_of_is_present (surface)."""
import operator

assert hasattr(operator, "indexOf")
print("api_index_of_is_present OK")
