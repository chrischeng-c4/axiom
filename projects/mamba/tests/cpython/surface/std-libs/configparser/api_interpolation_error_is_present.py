# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_interpolation_error_is_present"
# subject = "configparser.InterpolationError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.InterpolationError: api_interpolation_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "InterpolationError")
print("api_interpolation_error_is_present OK")
