# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "getlevelname_is_callable"
# subject = "logging.getLevelName"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLevelName: getlevelname_is_callable (surface)."""
import logging

assert callable(logging.getLevelName)
print("getlevelname_is_callable OK")
