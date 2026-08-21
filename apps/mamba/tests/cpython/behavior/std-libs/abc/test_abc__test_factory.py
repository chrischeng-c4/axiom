# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "test_abc__test_factory"
# subject = "cpython.test_abc.test_factory"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_abc.py::test_factory
"""Auto-ported test: test_abc::test_factory (CPython 3.12 oracle)."""

import _py_abc
import abc
import unittest
from test import test_abc


def assert_factory_suite(label, meta, get_cache_token):
    classes = test_abc.test_factory(meta, get_cache_token)
    assert [cls.__name__ for cls in classes] == [
        "TestLegacyAPI",
        "TestABC",
        "TestABCWithInitSubclass",
    ], (label, classes)
    for cls in classes:
        assert issubclass(cls, unittest.TestCase), (label, cls)
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(cls)
        result = unittest.TestResult()
        suite.run(result)
        assert result.wasSuccessful(), (label, cls, result)
        assert not result.failures, (label, cls, result.failures)
        assert not result.errors, (label, cls, result.errors)


assert_factory_suite("abc", abc.ABCMeta, abc.get_cache_token)
assert_factory_suite("_py_abc", _py_abc.ABCMeta, _py_abc.get_cache_token)

print("test_abc::test_factory generated ABC suites: ok")
