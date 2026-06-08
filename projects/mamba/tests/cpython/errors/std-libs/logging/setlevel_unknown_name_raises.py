# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "setlevel_unknown_name_raises"
# subject = "logging.Logger"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Logger: setlevel_unknown_name_raises (errors)."""
import logging

_raised = False
try:
    logging.getLogger('err.setlevel').setLevel('NO_SUCH_LEVEL')
except ValueError:
    _raised = True
assert _raised, "setlevel_unknown_name_raises: expected ValueError"
print("setlevel_unknown_name_raises OK")
