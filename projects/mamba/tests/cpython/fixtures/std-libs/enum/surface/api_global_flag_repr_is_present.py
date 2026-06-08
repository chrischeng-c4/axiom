# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "surface"
# case = "api_global_flag_repr_is_present"
# subject = "enum.global_flag_repr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""enum.global_flag_repr: api_global_flag_repr_is_present (surface)."""
import enum

assert hasattr(enum, "global_flag_repr")
print("api_global_flag_repr_is_present OK")
