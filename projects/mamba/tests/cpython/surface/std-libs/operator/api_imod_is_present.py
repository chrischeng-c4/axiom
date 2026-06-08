# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_imod_is_present"
# subject = "operator.imod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.imod: api_imod_is_present (surface)."""
import operator

assert hasattr(operator, "imod")
print("api_imod_is_present OK")
