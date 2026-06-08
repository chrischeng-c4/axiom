# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dict_version"
# dimension = "behavior"
# case = "dict_version_tests__test_setdefault"
# subject = "cpython.test_dict_version.DictVersionTests.test_setdefault"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict_version.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict_version.py::DictVersionTests::test_setdefault
"""Auto-ported test: DictVersionTests::test_setdefault (CPython 3.12 oracle)."""


import unittest
from test.support import import_helper


'\nTest implementation of the PEP 509: dictionary versioning.\n'

_testcapi = import_helper.import_module('_testcapi')

class Dict(dict):
    pass


# --- test body ---
type2test = dict

def check_version_changed(mydict, method, *args, **kw):
    result = method(*args, **kw)
    check_version_unique(mydict)
    return result

def check_version_dont_change(mydict, method, *args, **kw):
    version1 = _testcapi.dict_get_version(mydict)
    self_seen_versions.add(version1)
    result = method(*args, **kw)
    version2 = _testcapi.dict_get_version(mydict)

    assert version2 == version1
    return result

def check_version_unique(mydict):
    version = _testcapi.dict_get_version(mydict)

    assert version not in self_seen_versions
    self_seen_versions.add(version)

def new_dict(*args, **kw):
    d = type2test(*args, **kw)
    check_version_unique(d)
    return d
self_seen_versions = set()
self_dict = None
d = new_dict()
check_version_changed(d, d.setdefault, 'key', 'value1')
check_version_dont_change(d, d.setdefault, 'key', 'value2')
print("DictVersionTests::test_setdefault: ok")
