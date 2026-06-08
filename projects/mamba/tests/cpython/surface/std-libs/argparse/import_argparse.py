# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "import_argparse"
# subject = "argparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse: import_argparse (surface)."""
import argparse

assert hasattr(argparse, "ArgumentParser")
print("import_argparse OK")
