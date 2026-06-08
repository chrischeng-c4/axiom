# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "get_missing_option_raises"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: get_missing_option_raises (errors)."""
import configparser
_cp_no_opt = configparser.ConfigParser()
_cp_no_opt.read_string('[s1]\nkey=val\n')

_raised = False
try:
    _cp_no_opt.get('s1', 'nokey')
except configparser.NoOptionError:
    _raised = True
assert _raised, "get_missing_option_raises: expected configparser.NoOptionError"
print("get_missing_option_raises OK")
