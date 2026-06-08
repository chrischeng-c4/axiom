# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "behavior"
# case = "test_c_test__test_cjson"
# subject = "cpython.test_json.TestCTest.test_cjson"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_json/__init__.py::TestCTest::test_cjson
"""Auto-ported test: TestCTest::test_cjson (CPython 3.12 oracle)."""


import json
from test.support import import_helper


cjson = import_helper.import_fresh_module("json", fresh=["_json"])
if not cjson:
    print("TestCTest::test_cjson: skipped; requires _json")
    raise SystemExit(0)

cjson.JSONDecodeError = cjson.decoder.JSONDecodeError = json.JSONDecodeError

assert cjson.scanner.make_scanner.__module__ == "_json"
assert cjson.decoder.scanstring.__module__ == "_json"
assert cjson.encoder.c_make_encoder.__module__ == "_json"
assert cjson.encoder.encode_basestring_ascii.__module__ == "_json"

print("TestCTest::test_cjson: ok")
