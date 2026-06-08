# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "getlevelnamesmapping_is_callable"
# subject = "logging.getLevelNamesMapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLevelNamesMapping: getlevelnamesmapping_is_callable (surface)."""
import logging

assert callable(logging.getLevelNamesMapping)
print("getlevelnamesmapping_is_callable OK")
