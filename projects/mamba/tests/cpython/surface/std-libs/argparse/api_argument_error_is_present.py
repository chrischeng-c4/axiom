# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_argument_error_is_present"
# subject = "argparse.ArgumentError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.ArgumentError: api_argument_error_is_present (surface)."""
import argparse

assert hasattr(argparse, "ArgumentError")
print("api_argument_error_is_present OK")
