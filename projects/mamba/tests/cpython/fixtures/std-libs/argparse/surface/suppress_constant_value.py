# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "suppress_constant_value"
# subject = "argparse.SUPPRESS"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.SUPPRESS: the SUPPRESS sentinel equals the documented '==SUPPRESS==' string"""
import argparse

assert argparse.SUPPRESS == "==SUPPRESS==", f"SUPPRESS = {argparse.SUPPRESS!r}"
print("suppress_constant_value OK")
