# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "argument_parser_is_callable"
# subject = "argparse.ArgumentParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser: argument_parser_is_callable (surface)."""
import argparse

assert callable(argparse.ArgumentParser)
print("argument_parser_is_callable OK")
