# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_legacy_interpolation_is_present"
# subject = "configparser.LegacyInterpolation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.LegacyInterpolation: api_legacy_interpolation_is_present (surface)."""
import configparser

assert hasattr(configparser, "LegacyInterpolation")
print("api_legacy_interpolation_is_present OK")
