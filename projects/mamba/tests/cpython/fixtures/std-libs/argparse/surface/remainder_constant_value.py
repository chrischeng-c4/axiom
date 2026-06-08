# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "remainder_constant_value"
# subject = "argparse.REMAINDER"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.REMAINDER: the REMAINDER nargs sentinel equals the documented '...' string"""
import argparse

assert argparse.REMAINDER == "...", f"REMAINDER = {argparse.REMAINDER!r}"
print("remainder_constant_value OK")
