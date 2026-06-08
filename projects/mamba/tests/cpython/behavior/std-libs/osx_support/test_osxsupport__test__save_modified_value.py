# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "osx_support"
# dimension = "behavior"
# case = "test_osxsupport__test__save_modified_value"
# subject = "cpython.test__osx_support.Test_OSXSupport.test__save_modified_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__osx_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Test_OSXSupport.test__save_modified_value: changed values preserve original."""
import _osx_support

config_vars = {"CC": "gcc-test -pthreads"}
expected_vars = {
    "CC": "clang -pthreads",
    "_OSX_SUPPORT_INITIAL_CC": "gcc-test -pthreads",
}

_osx_support._save_modified_value(config_vars, "CC", "clang -pthreads")
assert config_vars == expected_vars, config_vars

print("Test_OSXSupport::test__save_modified_value: ok")
