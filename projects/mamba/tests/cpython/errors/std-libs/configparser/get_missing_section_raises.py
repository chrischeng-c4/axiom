# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "get_missing_section_raises"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: get_missing_section_raises (errors)."""
import configparser

_raised = False
try:
    configparser.ConfigParser().get('nosection', 'key')
except configparser.NoSectionError:
    _raised = True
assert _raised, "get_missing_section_raises: expected configparser.NoSectionError"
print("get_missing_section_raises OK")
