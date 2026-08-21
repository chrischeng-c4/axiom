# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "getlevelname_name_to_number"
# subject = "logging.getLevelName"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.getLevelName: getLevelName resolves a level name to its numeric level: 'INFO' -> INFO, 'WARNING' -> 30"""
import logging

assert logging.getLevelName("INFO") == logging.INFO, "name -> number"
assert logging.getLevelName("WARNING") == logging.WARNING, "WARNING -> 30"
print("getlevelname_name_to_number OK")
