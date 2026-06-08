# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_raw_description_help_formatter_is_present"
# subject = "argparse.RawDescriptionHelpFormatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.RawDescriptionHelpFormatter: api_raw_description_help_formatter_is_present (surface)."""
import argparse

assert hasattr(argparse, "RawDescriptionHelpFormatter")
print("api_raw_description_help_formatter_is_present OK")
