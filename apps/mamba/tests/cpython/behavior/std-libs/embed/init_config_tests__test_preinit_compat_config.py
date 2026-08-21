# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "embed"
# dimension = "behavior"
# case = "init_config_tests__test_preinit_compat_config"
# subject = "cpython.test_embed.InitConfigTests.test_preinit_compat_config"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_embed.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_embed
_suite = unittest.defaultTestLoader.loadTestsFromName("InitConfigTests.test_preinit_compat_config", test_embed)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InitConfigTests.test_preinit_compat_config did not pass"
print("InitConfigTests::test_preinit_compat_config: ok")
