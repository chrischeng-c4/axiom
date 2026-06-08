# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_parsing_error_is_present"
# subject = "configparser.ParsingError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.ParsingError: api_parsing_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "ParsingError")
print("api_parsing_error_is_present OK")
