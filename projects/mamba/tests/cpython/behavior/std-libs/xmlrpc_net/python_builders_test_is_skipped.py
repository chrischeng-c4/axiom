# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_net"
# dimension = "behavior"
# case = "python_builders_test_is_skipped"
# subject = "cpython.test_xmlrpc_net.PythonBuildersTest.test_python_builders"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc_net.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""test_xmlrpc_net: PythonBuildersTest is skipped without touching network."""
import io
import unittest

import test.test_xmlrpc_net as test_xmlrpc_net

suite = unittest.defaultTestLoader.loadTestsFromTestCase(
    test_xmlrpc_net.PythonBuildersTest
)
result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert len(result.skipped) == 1, result.skipped
assert result.skipped[0][1] == "XXX: buildbot.python.org/all/xmlrpc/ is gone"
assert not result.failures, result.failures
assert not result.errors, result.errors

print("python_builders_test_is_skipped OK")
