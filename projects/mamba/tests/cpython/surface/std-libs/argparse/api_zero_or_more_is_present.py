# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_zero_or_more_is_present"
# subject = "argparse.ZERO_OR_MORE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.ZERO_OR_MORE: api_zero_or_more_is_present (surface)."""
import argparse

assert hasattr(argparse, "ZERO_OR_MORE")
print("api_zero_or_more_is_present OK")
