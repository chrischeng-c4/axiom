# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "parsing_error_without_source_raises"
# subject = "configparser.ParsingError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ParsingError: parsing_error_without_source_raises (errors)."""
import configparser

_raised = False
try:
    configparser.ParsingError()
except TypeError:
    _raised = True
assert _raised, "parsing_error_without_source_raises: expected TypeError"
print("parsing_error_without_source_raises OK")
