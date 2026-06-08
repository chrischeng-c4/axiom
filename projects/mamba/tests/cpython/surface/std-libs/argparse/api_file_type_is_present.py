# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "api_file_type_is_present"
# subject = "argparse.FileType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""argparse.FileType: api_file_type_is_present (surface)."""
import argparse

assert hasattr(argparse, "FileType")
print("api_file_type_is_present OK")
