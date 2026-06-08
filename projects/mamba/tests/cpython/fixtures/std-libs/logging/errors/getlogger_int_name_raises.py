# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "getlogger_int_name_raises"
# subject = "logging.getLogger"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLogger: getlogger_int_name_raises (errors)."""
import logging

_raised = False
try:
    logging.getLogger(123)
except TypeError:
    _raised = True
assert _raised, "getlogger_int_name_raises: expected TypeError"
print("getlogger_int_name_raises OK")
