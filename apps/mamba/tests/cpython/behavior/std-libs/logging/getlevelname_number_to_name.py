# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getlevelname_number_to_name"
# subject = "logging.getLevelName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLevelName: getLevelName resolves a numeric level back to its name: INFO -> 'INFO', ERROR (40) -> 'ERROR'"""
import logging

assert logging.getLevelName(logging.INFO) == "INFO", "number -> name"
assert logging.getLevelName(logging.ERROR) == "ERROR", "40 -> ERROR"
print("getlevelname_number_to_name OK")
