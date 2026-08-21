# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "behavior"
# case = "import_side_effect_tests__test_license_exists_at_url_uc739450"
# subject = "cpython.test_site.ImportSideEffectTests.test_license_exists_at_url"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_site.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_site
_suite = unittest.defaultTestLoader.loadTestsFromName("ImportSideEffectTests.test_license_exists_at_url", test_site)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImportSideEffectTests.test_license_exists_at_url did not pass"
print("ImportSideEffectTests::test_license_exists_at_url: ok")
