# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_pos_is_present"
# subject = "operator.pos"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.pos: api_pos_is_present (surface)."""
import operator

assert hasattr(operator, "pos")
print("api_pos_is_present OK")
