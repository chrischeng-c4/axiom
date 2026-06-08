# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_parser_is_present"
# subject = "argparse.PARSER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.PARSER: api_parser_is_present (surface)."""
import argparse

assert hasattr(argparse, "PARSER")
print("api_parser_is_present OK")
