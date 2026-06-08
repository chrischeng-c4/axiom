# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_duplicate_option_error_is_present"
# subject = "configparser.DuplicateOptionError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.DuplicateOptionError: api_duplicate_option_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "DuplicateOptionError")
print("api_duplicate_option_error_is_present OK")
