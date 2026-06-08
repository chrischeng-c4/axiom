# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_basic_interpolation_is_present"
# subject = "configparser.BasicInterpolation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.BasicInterpolation: api_basic_interpolation_is_present (surface)."""
import configparser

assert hasattr(configparser, "BasicInterpolation")
print("api_basic_interpolation_is_present OK")
