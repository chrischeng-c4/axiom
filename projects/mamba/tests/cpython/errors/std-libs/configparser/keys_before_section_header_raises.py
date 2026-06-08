# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "keys_before_section_header_raises"
# subject = "configparser.ConfigParser.read_string"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.read_string: keys_before_section_header_raises (errors)."""
import configparser

_raised = False
try:
    configparser.ConfigParser().read_string('key = value\n')
except configparser.MissingSectionHeaderError:
    _raised = True
assert _raised, "keys_before_section_header_raises: expected configparser.MissingSectionHeaderError"
print("keys_before_section_header_raises OK")
