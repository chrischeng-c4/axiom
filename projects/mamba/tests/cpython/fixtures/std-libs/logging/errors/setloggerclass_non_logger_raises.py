# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "setloggerclass_non_logger_raises"
# subject = "logging.setLoggerClass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.setLoggerClass: setloggerclass_non_logger_raises (errors)."""
import logging

_raised = False
try:
    logging.setLoggerClass(object)
except TypeError:
    _raised = True
assert _raised, "setloggerclass_non_logger_raises: expected TypeError"
print("setloggerclass_non_logger_raises OK")
