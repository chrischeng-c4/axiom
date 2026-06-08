# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_float_repr_style_is_present"
# subject = "sys.float_repr_style"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.float_repr_style: api_float_repr_style_is_present (surface)."""
import sys

assert hasattr(sys, "float_repr_style")
print("api_float_repr_style_is_present OK")
