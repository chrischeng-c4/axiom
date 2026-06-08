# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "interning_test__test_issue9402"
# subject = "cpython.test_pyexpat.InterningTest.test_issue9402"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pyexpat.py::InterningTest::test_issue9402
"""Auto-ported test: InterningTest::test_issue9402 (CPython 3.12 oracle)."""


import os
import platform
import sys
import sysconfig
import unittest
import traceback
from io import BytesIO
from test import support
from test.support import os_helper
from xml.parsers import expat
from xml.parsers.expat import errors
from test.support import sortdict, is_emscripten, is_wasi


data = b'<?xml version="1.0" encoding="iso-8859-1" standalone="no"?>\n<?xml-stylesheet href="stylesheet.css"?>\n<!-- comment data -->\n<!DOCTYPE quotations SYSTEM "quotations.dtd" [\n<!ELEMENT root ANY>\n<!ATTLIST root attr1 CDATA #REQUIRED attr2 CDATA #IMPLIED>\n<!NOTATION notation SYSTEM "notation.jpeg">\n<!ENTITY acirc "&#226;">\n<!ENTITY external_entity SYSTEM "entity.file">\n<!ENTITY unparsed_entity SYSTEM "entity.file" NDATA notation>\n%unparsed_entity;\n]>\n\n<root attr1="value1" attr2="value2&#8000;">\n<myns:subelement xmlns:myns="http://www.python.org/namespace">\n     Contents of subelements\n</myns:subelement>\n<sub2><![CDATA[contents of CDATA section]]></sub2>\n&external_entity;\n&skipped_entity;\n\xb5\n</root>\n'

class PositionTest(unittest.TestCase):

    def StartElementHandler(self, name, attrs):
        self.check_pos('s')

    def EndElementHandler(self, name):
        self.check_pos('e')

    def check_pos(self, event):
        pos = (event, self.parser.CurrentByteIndex, self.parser.CurrentLineNumber, self.parser.CurrentColumnNumber)
        self.assertTrue(self.upto < len(self.expected_list), 'too many parser events')
        expected = self.expected_list[self.upto]
        self.assertEqual(pos, expected, 'Expected position %s, got position %s' % (pos, expected))
        self.upto += 1

    def test(self):
        self.parser = expat.ParserCreate()
        self.parser.StartElementHandler = self.StartElementHandler
        self.parser.EndElementHandler = self.EndElementHandler
        self.upto = 0
        self.expected_list = [('s', 0, 1, 0), ('s', 5, 2, 1), ('s', 11, 3, 2), ('e', 15, 3, 6), ('e', 17, 4, 1), ('e', 22, 5, 0)]
        xml = b'<a>\n <b>\n  <c/>\n </b>\n</a>'
        self.parser.Parse(xml, True)

class MalformedInputTest(unittest.TestCase):

    def test1(self):
        xml = b'\x00\r\n'
        parser = expat.ParserCreate()
        try:
            parser.Parse(xml, True)
            self.fail()
        except expat.ExpatError as e:
            self.assertEqual(str(e), 'unclosed token: line 2, column 0')

    def test2(self):
        xml = b"<?xml version\xc2\x85='1.0'?>\r\n"
        parser = expat.ParserCreate()
        err_pattern = 'XML declaration not well-formed: line 1, column \\d+'
        with self.assertRaisesRegex(expat.ExpatError, err_pattern):
            parser.Parse(xml, True)


# --- test body ---
class ExternalOutputter:

    def __init__(self, parser):
        self.parser = parser
        self.parser_result = None

    def ExternalEntityRefHandler(self, context, base, sysId, pubId):
        external_parser = self.parser.ExternalEntityParserCreate('')
        self.parser_result = external_parser.Parse(b'', True)
        return 1
parser = expat.ParserCreate(namespace_separator='!')
parser.buffer_text = 1
out = ExternalOutputter(parser)
parser.ExternalEntityRefHandler = out.ExternalEntityRefHandler
parser.Parse(data, True)

assert out.parser_result == 1
print("InterningTest::test_issue9402: ok")
