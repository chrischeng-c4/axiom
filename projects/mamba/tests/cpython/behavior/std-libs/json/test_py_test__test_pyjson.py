# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "test_py_test__test_pyjson"
# subject = "cpython.test_json.TestPyTest.test_pyjson"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::TestPyTest::test_pyjson
"""Auto-ported test: TestPyTest::test_pyjson (CPython 3.12 oracle)."""


import os
import json
import doctest
import unittest
from test import support
from test.support import import_helper


cjson = import_helper.import_fresh_module('json', fresh=['_json'])

pyjson = import_helper.import_fresh_module('json', blocked=['_json'])

cjson.JSONDecodeError = cjson.decoder.JSONDecodeError = json.JSONDecodeError

class PyTest(unittest.TestCase):
    json = pyjson
    loads = staticmethod(pyjson.loads)
    dumps = staticmethod(pyjson.dumps)
    JSONDecodeError = staticmethod(pyjson.JSONDecodeError)

@unittest.skipUnless(cjson, 'requires _json')
class CTest(unittest.TestCase):
    if cjson is not None:
        json = cjson
        loads = staticmethod(cjson.loads)
        dumps = staticmethod(cjson.dumps)
        JSONDecodeError = staticmethod(cjson.JSONDecodeError)

def load_tests(loader, _, pattern):
    suite = unittest.TestSuite()
    for mod in (json, json.encoder, json.decoder):
        suite.addTest(doctest.DocTestSuite(mod))
    suite.addTest(TestPyTest('test_pyjson'))
    suite.addTest(TestCTest('test_cjson'))
    pkg_dir = os.path.dirname(__file__)
    return support.load_package_tests(pkg_dir, loader, suite, pattern)


# --- test body ---
json = pyjson
loads = staticmethod(pyjson.loads)
dumps = staticmethod(pyjson.dumps)
JSONDecodeError = staticmethod(pyjson.JSONDecodeError)

assert json.scanner.make_scanner.__module__ == 'json.scanner'

assert json.decoder.scanstring.__module__ == 'json.decoder'

assert json.encoder.encode_basestring_ascii.__module__ == 'json.encoder'
print("TestPyTest::test_pyjson: ok")
