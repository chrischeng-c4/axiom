# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_extended_interpolation_is_present"
# subject = "configparser.ExtendedInterpolation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.ExtendedInterpolation: api_extended_interpolation_is_present (surface)."""
import configparser

assert hasattr(configparser, "ExtendedInterpolation")
print("api_extended_interpolation_is_present OK")
