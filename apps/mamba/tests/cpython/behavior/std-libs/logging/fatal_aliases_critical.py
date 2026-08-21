# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "fatal_aliases_critical"
# subject = "logging.FATAL"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.FATAL: FATAL is an alias for CRITICAL: getLevelName('FATAL') == FATAL == CRITICAL == 50 (issue 27935)"""
import logging

assert logging.getLevelName("FATAL") == logging.FATAL, "FATAL resolves"
assert logging.FATAL == logging.CRITICAL == 50, "FATAL == CRITICAL == 50"
print("fatal_aliases_critical OK")
