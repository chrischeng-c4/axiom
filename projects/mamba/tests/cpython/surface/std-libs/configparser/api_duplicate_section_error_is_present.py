# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_duplicate_section_error_is_present"
# subject = "configparser.DuplicateSectionError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.DuplicateSectionError: api_duplicate_section_error_is_present (surface)."""
import configparser

assert hasattr(configparser, "DuplicateSectionError")
print("api_duplicate_section_error_is_present OK")
