# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "raw_text_help_formatter_exists"
# subject = "argparse.RawTextHelpFormatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.RawTextHelpFormatter: raw_text_help_formatter_exists (surface)."""
import argparse

assert callable(argparse.RawTextHelpFormatter)
print("raw_text_help_formatter_exists OK")
