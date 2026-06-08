# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "errors"
# case = "interpolation_cycle_exceeds_depth_raises"
# subject = "configparser.ConfigParser.get"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser.get: interpolation_cycle_exceeds_depth_raises (errors)."""
import configparser
_cp_cyc = configparser.ConfigParser()
_cp_cyc.read_string('[s]\na = %(b)s\nb = %(a)s\n')

_raised = False
try:
    _cp_cyc.get('s', 'a')
except configparser.InterpolationDepthError:
    _raised = True
assert _raised, "interpolation_cycle_exceeds_depth_raises: expected configparser.InterpolationDepthError"
print("interpolation_cycle_exceeds_depth_raises OK")
