# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_optional_is_present"
# subject = "argparse.OPTIONAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.OPTIONAL: api_optional_is_present (surface)."""
import argparse

assert hasattr(argparse, "OPTIONAL")
print("api_optional_is_present OK")
