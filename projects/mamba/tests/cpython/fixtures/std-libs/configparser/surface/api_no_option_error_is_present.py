# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_no_option_error_is_present"
# subject = "configparser.NoOptionError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.NoOptionError: api_no_option_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "NoOptionError")
print("api_no_option_error_is_present OK")
