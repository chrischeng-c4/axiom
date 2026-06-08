# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "argument_type_error_is_class"
# subject = "argparse.ArgumentTypeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentTypeError: argument_type_error_is_class (surface)."""
import argparse

assert type(argparse.ArgumentTypeError).__name__ == "type"
print("argument_type_error_is_class OK")
