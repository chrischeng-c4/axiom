# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "level_constants_canonical_values"
# subject = "logging.DEBUG"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.DEBUG: the six built-in level constants have their canonical numeric values: DEBUG=10, INFO=20, WARNING=30, ERROR=40, CRITICAL=50, NOTSET=0"""
import logging

assert logging.DEBUG == 10, f"DEBUG = {logging.DEBUG!r}"
assert logging.INFO == 20, f"INFO = {logging.INFO!r}"
assert logging.WARNING == 30, f"WARNING = {logging.WARNING!r}"
assert logging.ERROR == 40, f"ERROR = {logging.ERROR!r}"
assert logging.CRITICAL == 50, f"CRITICAL = {logging.CRITICAL!r}"
assert logging.NOTSET == 0, f"NOTSET = {logging.NOTSET!r}"
print("level_constants_canonical_values OK")
