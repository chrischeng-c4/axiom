# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "streamhandler_attr"
# subject = "logging"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging: streamhandler_attr (surface)."""
import logging

assert hasattr(logging, "StreamHandler")
print("streamhandler_attr OK")
