# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_suppress_is_present"
# subject = "argparse.SUPPRESS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.SUPPRESS: api_suppress_is_present (surface)."""
import argparse

assert hasattr(argparse, "SUPPRESS")
print("api_suppress_is_present OK")
