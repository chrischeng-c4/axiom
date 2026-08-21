# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_c"
# dimension = "behavior"
# case = "misc_tests__test_uninitialized_parser"
# subject = "cpython.test_xml_etree_c.MiscTests.test_uninitialized_parser"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree_c.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_etree_c.py::MiscTests::test_uninitialized_parser
"""Auto-ported test: MiscTests::test_uninitialized_parser (CPython 3.12 oracle)."""


import io
import struct
from test import support
from test.support.import_helper import import_fresh_module
import types
import unittest


cET = import_fresh_module('xml.etree.ElementTree', fresh=['_elementtree'])

cET_alias = import_fresh_module('xml.etree.cElementTree', fresh=['_elementtree', 'xml.etree'], deprecated=True)

def install_tests():
    from test import test_xml_etree
    for name, base in vars(test_xml_etree).items():
        if isinstance(base, type) and issubclass(base, unittest.TestCase):

            class Temp(base):
                pass
            Temp.__name__ = Temp.__qualname__ = name
            Temp.__module__ = __name__
            assert name not in globals()
            globals()[name] = Temp

install_tests()

def setUpModule():
    from test import test_xml_etree
    test_xml_etree.setUpModule(module=cET)


# --- test body ---
parser = cET.XMLParser.__new__(cET.XMLParser)

try:
    parser.close()
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    parser.feed('foo')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

class MockFile:

    def read(*args):
        return ''

try:
    parser._parse_whole(MockFile())
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    parser._setevents(None)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert parser.entity is None

assert parser.target is None
print("MiscTests::test_uninitialized_parser: ok")
