# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_interpolation_is_present"
# subject = "configparser.Interpolation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.Interpolation: api_interpolation_is_present (surface)."""
import configparser

assert hasattr(configparser, "Interpolation")
print("api_interpolation_is_present OK")
