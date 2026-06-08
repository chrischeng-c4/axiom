# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_max_interpolation_depth_is_present"
# subject = "configparser.MAX_INTERPOLATION_DEPTH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.MAX_INTERPOLATION_DEPTH: api_max_interpolation_depth_is_present (surface)."""
import configparser

assert hasattr(configparser, "MAX_INTERPOLATION_DEPTH")
print("api_max_interpolation_depth_is_present OK")
