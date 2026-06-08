# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "manager_setloggerclass_non_logger_raises"
# subject = "logging.Manager"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Manager: manager_setloggerclass_non_logger_raises (errors)."""
import logging

_raised = False
try:
    logging.Manager(None).setLoggerClass(int)
except TypeError:
    _raised = True
assert _raised, "manager_setloggerclass_non_logger_raises: expected TypeError"
print("manager_setloggerclass_non_logger_raises OK")
