# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "makelogrecord_is_callable"
# subject = "logging.makeLogRecord"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.makeLogRecord: makelogrecord_is_callable (surface)."""
import logging

assert callable(logging.makeLogRecord)
print("makelogrecord_is_callable OK")
