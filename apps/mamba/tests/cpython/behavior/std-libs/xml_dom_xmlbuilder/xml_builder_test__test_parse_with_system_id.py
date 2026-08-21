# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "behavior"
# case = "xml_builder_test__test_parse_with_system_id"
# subject = "cpython.test_xml_dom_xmlbuilder.XMLBuilderTest.test_parse_with_systemId"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_xmlbuilder.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_xmlbuilder.py::XMLBuilderTest::test_parse_with_systemId
"""Auto-ported test: XMLBuilderTest::test_parse_with_systemId (CPython 3.12 oracle)."""


import io
import unittest
from http import client
from test.test_httplib import FakeSocket
from unittest import mock
from xml.dom import getDOMImplementation, minidom, xmlbuilder


SMALL_SAMPLE = b'<?xml version="1.0"?>\n<html xmlns="http://www.w3.org/1999/xhtml" xmlns:xdc="http://www.xml.com/books">\n<!-- A comment -->\n<title>Introduction to XSL</title>\n<hr/>\n<p><xdc:author xdc:attrib="prefixed attribute" attrib="other attrib">A. Namespace</xdc:author></p>\n</html>'


# --- test body ---
response = io.BytesIO(SMALL_SAMPLE)
with mock.patch('urllib.request.urlopen') as mock_open:
    mock_open.return_value = response
    imp = getDOMImplementation()
    source = imp.createDOMInputSource()
    builder = imp.createDOMBuilder(imp.MODE_SYNCHRONOUS, None)
    source.systemId = 'http://example.com/2000/svg'
    document = builder.parse(source)

assert isinstance(document, minidom.Document)

assert len(document.childNodes) == 1
print("XMLBuilderTest::test_parse_with_systemId: ok")
