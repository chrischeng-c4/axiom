# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_p_e_p302_tests__test_alreadyloaded_uc9fda6c"
# subject = "cpython.test_pkgutil.PkgutilPEP302Tests.test_alreadyloaded"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pkgutil
_suite = unittest.defaultTestLoader.loadTestsFromName("PkgutilPEP302Tests.test_alreadyloaded", test_pkgutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PkgutilPEP302Tests.test_alreadyloaded did not pass"
print("PkgutilPEP302Tests::test_alreadyloaded: ok")
