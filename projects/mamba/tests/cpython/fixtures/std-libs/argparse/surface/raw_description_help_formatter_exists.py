# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "raw_description_help_formatter_exists"
# subject = "argparse.RawDescriptionHelpFormatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.RawDescriptionHelpFormatter: raw_description_help_formatter_exists (surface)."""
import argparse

assert callable(argparse.RawDescriptionHelpFormatter)
print("raw_description_help_formatter_exists OK")
