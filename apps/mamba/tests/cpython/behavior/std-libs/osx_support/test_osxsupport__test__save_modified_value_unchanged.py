# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "osx_support"
# dimension = "behavior"
# case = "test_osxsupport__test__save_modified_value_unchanged"
# subject = "cpython.test__osx_support.Test_OSXSupport.test__save_modified_value_unchanged"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__osx_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Test_OSXSupport.test__save_modified_value_unchanged leaves dict unchanged."""
import _osx_support

config_vars = {"CC": "gcc-test -pthreads"}
expected_vars = dict(config_vars)

_osx_support._save_modified_value(config_vars, "CC", "gcc-test -pthreads")
assert config_vars == expected_vars, config_vars

print("Test_OSXSupport::test__save_modified_value_unchanged: ok")
