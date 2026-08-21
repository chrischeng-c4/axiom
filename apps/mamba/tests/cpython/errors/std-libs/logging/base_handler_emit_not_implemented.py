# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "errors"
# case = "base_handler_emit_not_implemented"
# subject = "logging.Handler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.Handler: base_handler_emit_not_implemented (errors)."""
import logging

_raised = False
try:
    logging.Handler().emit(None)
except NotImplementedError:
    _raised = True
assert _raised, "base_handler_emit_not_implemented: expected NotImplementedError"
print("base_handler_emit_not_implemented OK")
