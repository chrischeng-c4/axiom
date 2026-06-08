# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_interpolation_missing_option_error_is_present"
# subject = "configparser.InterpolationMissingOptionError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.InterpolationMissingOptionError: api_interpolation_missing_option_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "InterpolationMissingOptionError")
print("api_interpolation_missing_option_error_is_present OK")
