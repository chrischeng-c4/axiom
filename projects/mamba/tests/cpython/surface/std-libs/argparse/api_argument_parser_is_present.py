# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_argument_parser_is_present"
# subject = "argparse.ArgumentParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.ArgumentParser: api_argument_parser_is_present (surface)."""
import argparse

assert hasattr(argparse, "ArgumentParser")
print("api_argument_parser_is_present OK")
