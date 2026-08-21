# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "argument_error_is_class"
# subject = "argparse.ArgumentError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentError: argument_error_is_class (surface)."""
import argparse

assert type(argparse.ArgumentError).__name__ == "type"
print("argument_error_is_class OK")
