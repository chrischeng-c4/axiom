# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_raw_text_help_formatter_is_present"
# subject = "argparse.RawTextHelpFormatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.RawTextHelpFormatter: api_raw_text_help_formatter_is_present (surface)."""
import argparse

assert hasattr(argparse, "RawTextHelpFormatter")
print("api_raw_text_help_formatter_is_present OK")
