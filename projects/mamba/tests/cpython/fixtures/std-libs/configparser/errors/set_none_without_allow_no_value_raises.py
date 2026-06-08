# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "set_none_without_allow_no_value_raises"
# subject = "configparser.ConfigParser.set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.set: set_none_without_allow_no_value_raises (errors)."""
import configparser
_cp_set_none = configparser.ConfigParser(allow_no_value=False)
_cp_set_none.add_section('s')

_raised = False
try:
    _cp_set_none.set('s', 'opt', None)
except TypeError:
    _raised = True
assert _raised, "set_none_without_allow_no_value_raises: expected TypeError"
print("set_none_without_allow_no_value_raises OK")
