# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_help_formatter_is_present"
# subject = "argparse.HelpFormatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.HelpFormatter: api_help_formatter_is_present (surface)."""
import argparse

assert hasattr(argparse, "HelpFormatter")
print("api_help_formatter_is_present OK")
