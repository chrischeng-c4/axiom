# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "duplicate_option_in_section_raises"
# subject = "configparser.ConfigParser.read_string"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_string: duplicate_option_in_section_raises (errors)."""
import configparser

_raised = False
try:
    configparser.ConfigParser().read_string('[s]\na = 1\na = 2\n')
except configparser.DuplicateOptionError:
    _raised = True
assert _raised, "duplicate_option_in_section_raises: expected configparser.DuplicateOptionError"
print("duplicate_option_in_section_raises OK")
