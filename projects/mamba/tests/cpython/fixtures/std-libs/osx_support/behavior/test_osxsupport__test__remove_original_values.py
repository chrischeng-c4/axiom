# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "osx_support"
# dimension = "behavior"
# case = "test_osxsupport__test__remove_original_values"
# subject = "cpython.test__osx_support.Test_OSXSupport.test__remove_original_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__osx_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Test_OSXSupport.test__remove_original_values removes saved initial keys."""
import _osx_support

config_vars = {"CC": "gcc-test -pthreads"}
expected_vars = {"CC": "clang -pthreads"}

_osx_support._save_modified_value(config_vars, "CC", "clang -pthreads")
assert config_vars != expected_vars, config_vars

_osx_support._remove_original_values(config_vars)
assert config_vars == expected_vars, config_vars

print("Test_OSXSupport::test__remove_original_values: ok")
