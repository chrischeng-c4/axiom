# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "add_argument_is_callable"
# subject = "argparse.ArgumentParser.add_argument"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentParser.add_argument: add_argument_is_callable (surface)."""
import argparse

assert callable(argparse.ArgumentParser.add_argument)
print("add_argument_is_callable OK")
