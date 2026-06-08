# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "api_rect_is_present"
# subject = "cmath.rect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cmath.rect: api_rect_is_present (surface)."""
import cmath

assert hasattr(cmath, "rect")
print("api_rect_is_present OK")
