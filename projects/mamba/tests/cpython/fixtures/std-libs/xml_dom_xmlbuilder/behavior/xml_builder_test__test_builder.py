# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_xmlbuilder"
# dimension = "behavior"
# case = "xml_builder_test__test_builder"
# subject = "cpython.test_xml_dom_xmlbuilder.XMLBuilderTest.test_builder"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_xmlbuilder.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_xmlbuilder.py::XMLBuilderTest::test_builder
"""Auto-ported test: XMLBuilderTest::test_builder (CPython 3.12 oracle)."""


import io
import unittest
from http import client
from test.test_httplib import FakeSocket
from unittest import mock
from xml.dom import getDOMImplementation, minidom, xmlbuilder


SMALL_SAMPLE = b'<?xml version="1.0"?>\n<html xmlns="http://www.w3.org/1999/xhtml" xmlns:xdc="http://www.xml.com/books">\n<!-- A comment -->\n<title>Introduction to XSL</title>\n<hr/>\n<p><xdc:author xdc:attrib="prefixed attribute" attrib="other attrib">A. Namespace</xdc:author></p>\n</html>'


# --- test body ---
imp = getDOMImplementation()

assert isinstance(imp, xmlbuilder.DOMImplementationLS)
builder = imp.createDOMBuilder(imp.MODE_SYNCHRONOUS, None)

assert isinstance(builder, xmlbuilder.DOMBuilder)
print("XMLBuilderTest::test_builder: ok")
