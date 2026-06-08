# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "formatter_unknown_style_raises"
# subject = "logging.Formatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Formatter: formatter_unknown_style_raises (errors)."""
import logging

_raised = False
try:
    logging.Formatter(None, None, 'x')
except ValueError:
    _raised = True
assert _raised, "formatter_unknown_style_raises: expected ValueError"
print("formatter_unknown_style_raises OK")
