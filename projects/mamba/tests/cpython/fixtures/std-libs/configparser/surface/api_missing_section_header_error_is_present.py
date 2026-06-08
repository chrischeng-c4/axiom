# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_missing_section_header_error_is_present"
# subject = "configparser.MissingSectionHeaderError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.MissingSectionHeaderError: api_missing_section_header_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "MissingSectionHeaderError")
print("api_missing_section_header_error_is_present OK")
