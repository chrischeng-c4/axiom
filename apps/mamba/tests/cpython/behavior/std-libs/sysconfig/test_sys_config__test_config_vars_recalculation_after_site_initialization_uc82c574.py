# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "behavior"
# case = "test_sys_config__test_config_vars_recalculation_after_site_initialization_uc82c574"
# subject = "cpython.test_sysconfig.TestSysConfig.test_config_vars_recalculation_after_site_initialization"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sysconfig.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sysconfig
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSysConfig.test_config_vars_recalculation_after_site_initialization", test_sysconfig)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSysConfig.test_config_vars_recalculation_after_site_initialization did not pass"
print("TestSysConfig::test_config_vars_recalculation_after_site_initialization: ok")
