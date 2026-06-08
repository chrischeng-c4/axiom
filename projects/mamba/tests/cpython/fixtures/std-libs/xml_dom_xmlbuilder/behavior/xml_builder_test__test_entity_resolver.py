# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "behavior"
# case = "xml_builder_test__test_entity_resolver"
# subject = "cpython.test_xml_dom_xmlbuilder.XMLBuilderTest.test_entity_resolver"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_xmlbuilder.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_xmlbuilder.py::XMLBuilderTest::test_entity_resolver
"""Auto-ported test: XMLBuilderTest::test_entity_resolver (CPython 3.12 oracle)."""


import io
import unittest
from http import client
from test.test_httplib import FakeSocket
from unittest import mock
from xml.dom import getDOMImplementation, minidom, xmlbuilder


SMALL_SAMPLE = b'<?xml version="1.0"?>\n<html xmlns="http://www.w3.org/1999/xhtml" xmlns:xdc="http://www.xml.com/books">\n<!-- A comment -->\n<title>Introduction to XSL</title>\n<hr/>\n<p><xdc:author xdc:attrib="prefixed attribute" attrib="other attrib">A. Namespace</xdc:author></p>\n</html>'


# --- test body ---
body = b'HTTP/1.1 200 OK\r\nContent-Type: text/xml; charset=utf-8\r\n\r\n' + SMALL_SAMPLE
sock = FakeSocket(body)
response = client.HTTPResponse(sock)
response.begin()
attrs = {'open.return_value': response}
opener = mock.Mock(**attrs)
resolver = xmlbuilder.DOMEntityResolver()
with mock.patch('urllib.request.build_opener') as mock_build:
    mock_build.return_value = opener
    source = resolver.resolveEntity(None, 'http://example.com/2000/svg')

assert isinstance(source, xmlbuilder.DOMInputSource)

assert source.publicId is None

assert source.systemId == 'http://example.com/2000/svg'

assert source.baseURI == 'http://example.com/2000/'

assert source.encoding == 'utf-8'

assert source.byteStream is response

assert source.characterStream is None

assert source.stringData is None
print("XMLBuilderTest::test_entity_resolver: ok")
