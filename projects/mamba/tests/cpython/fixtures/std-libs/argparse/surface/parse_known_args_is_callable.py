# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "parse_known_args_is_callable"
# subject = "argparse.ArgumentParser.parse_known_args"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.parse_known_args: parse_known_args_is_callable (surface)."""
import argparse

assert callable(argparse.ArgumentParser.parse_known_args)
print("parse_known_args_is_callable OK")
