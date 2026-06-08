# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_interpolation_syntax_error_is_present"
# subject = "configparser.InterpolationSyntaxError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.InterpolationSyntaxError: api_interpolation_syntax_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "InterpolationSyntaxError")
print("api_interpolation_syntax_error_is_present OK")
