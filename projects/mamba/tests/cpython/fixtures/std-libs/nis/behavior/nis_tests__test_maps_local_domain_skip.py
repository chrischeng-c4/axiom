# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nis"
# dimension = "behavior"
# case = "nis_tests__test_maps_local_domain_skip"
# subject = "cpython.test_nis.NisTests.test_maps"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_nis.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""NisTests.test_maps: local NIS misconfiguration is reported as skipped."""
import io
import unittest

import test.test_nis as test_nis

suite = unittest.defaultTestLoader.loadTestsFromTestCase(test_nis.NisTests)
result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert len(result.skipped) == 1, result.skipped
skip_reason = result.skipped[0][1]
assert skip_reason, result.skipped
assert not result.failures, result.failures
assert not result.errors, result.errors

print("NisTests::test_maps local-domain skip: ok")
