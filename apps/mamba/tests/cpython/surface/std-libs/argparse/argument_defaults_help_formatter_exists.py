# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "argument_defaults_help_formatter_exists"
# subject = "argparse.ArgumentDefaultsHelpFormatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentDefaultsHelpFormatter: argument_defaults_help_formatter_exists (surface)."""
import argparse

assert callable(argparse.ArgumentDefaultsHelpFormatter)
print("argument_defaults_help_formatter_exists OK")
