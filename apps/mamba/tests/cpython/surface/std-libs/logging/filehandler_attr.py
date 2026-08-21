# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "filehandler_attr"
# subject = "logging"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging: filehandler_attr (surface)."""
import logging

assert hasattr(logging, "FileHandler")
print("filehandler_attr OK")
