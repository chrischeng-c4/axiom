# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "getlogger_is_callable"
# subject = "logging.getLogger"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLogger: getlogger_is_callable (surface)."""
import logging

assert callable(logging.getLogger)
print("getlogger_is_callable OK")
