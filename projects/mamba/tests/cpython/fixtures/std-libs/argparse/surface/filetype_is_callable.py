# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "filetype_is_callable"
# subject = "argparse.FileType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.FileType: filetype_is_callable (surface)."""
import argparse

assert callable(argparse.FileType)
print("filetype_is_callable OK")
