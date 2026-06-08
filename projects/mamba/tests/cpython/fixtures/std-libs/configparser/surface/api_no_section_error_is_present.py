# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_no_section_error_is_present"
# subject = "configparser.NoSectionError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.NoSectionError: api_no_section_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "NoSectionError")
print("api_no_section_error_is_present OK")
