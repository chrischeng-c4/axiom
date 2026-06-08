# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "interpolation_missing_referent_raises"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: interpolation_missing_referent_raises (errors)."""
import configparser
_cp_miss = configparser.ConfigParser()
_cp_miss.read_string('[s]\nv = %(absent)s\n')

_raised = False
try:
    _cp_miss.get('s', 'v')
except configparser.InterpolationMissingOptionError:
    _raised = True
assert _raised, "interpolation_missing_referent_raises: expected configparser.InterpolationMissingOptionError"
print("interpolation_missing_referent_raises OK")
