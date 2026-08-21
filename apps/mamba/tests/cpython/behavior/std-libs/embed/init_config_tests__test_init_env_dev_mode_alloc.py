# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "embed"
# dimension = "behavior"
# case = "init_config_tests__test_init_env_dev_mode_alloc"
# subject = "cpython.test_embed.InitConfigTests.test_init_env_dev_mode_alloc"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_embed.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_embed
_suite = unittest.defaultTestLoader.loadTestsFromName("InitConfigTests.test_init_env_dev_mode_alloc", test_embed)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InitConfigTests.test_init_env_dev_mode_alloc did not pass"
print("InitConfigTests::test_init_env_dev_mode_alloc: ok")
