# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_remainder_is_present"
# subject = "argparse.REMAINDER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.REMAINDER: api_remainder_is_present (surface)."""
import argparse

assert hasattr(argparse, "REMAINDER")
print("api_remainder_is_present OK")
