# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_interpolation_depth_error_is_present"
# subject = "configparser.InterpolationDepthError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.InterpolationDepthError: api_interpolation_depth_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "InterpolationDepthError")
print("api_interpolation_depth_error_is_present OK")
