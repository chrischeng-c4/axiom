# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pulldom"
# dimension = "behavior"
# case = "pull_dom_test_case__test_parse_semantics"
# subject = "cpython.test_pulldom.PullDOMTestCase.test_parse_semantics"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pulldom.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pulldom.py::PullDOMTestCase::test_parse_semantics
"""Auto-ported test: PullDOMTestCase::test_parse_semantics (CPython 3.12 oracle)."""


import io
import unittest
import xml.sax
from xml.sax.xmlreader import AttributesImpl
from xml.sax.handler import feature_external_ges
from xml.dom import pulldom
from test.support import findfile


tstfile = findfile('test.xml', subdir='xmltestdata')

SMALL_SAMPLE = '<?xml version="1.0"?>\n<html xmlns="http://www.w3.org/1999/xhtml" xmlns:xdc="http://www.xml.com/books">\n<!-- A comment -->\n<title>Introduction to XSL</title>\n<hr/>\n<p><xdc:author xdc:attrib="prefixed attribute" attrib="other attrib">A. Namespace</xdc:author></p>\n</html>'

class SAXExerciser(object):
    """A fake sax parser that calls some of the harder-to-reach sax methods to
    ensure it emits the correct events"""

    def setContentHandler(self, handler):
        self._handler = handler

    def parse(self, _):
        h = self._handler
        h.startDocument()
        h.comment('a comment')
        h.processingInstruction('target', 'data')
        h.startElement('html', AttributesImpl({}))
        h.comment('a comment')
        h.processingInstruction('target', 'data')
        h.startElement('p', AttributesImpl({'class': 'paraclass'}))
        h.characters('text')
        h.endElement('p')
        h.endElement('html')
        h.endDocument()

    def stub(self, *args, **kwargs):
        """Stub method. Does nothing."""
        pass
    setProperty = stub
    setFeature = stub

class SAX2DOMExerciser(SAXExerciser):
    """The same as SAXExerciser, but without the processing instruction and
    comment before the root element, because S2D can"t handle it"""

    def parse(self, _):
        h = self._handler
        h.startDocument()
        h.startElement('html', AttributesImpl({}))
        h.comment('a comment')
        h.processingInstruction('target', 'data')
        h.startElement('p', AttributesImpl({'class': 'paraclass'}))
        h.characters('text')
        h.endElement('p')
        h.endElement('html')
        h.endDocument()

class SAX2DOMTestHelper(pulldom.DOMEventStream):
    """Allows us to drive SAX2DOM from a DOMEventStream."""

    def reset(self):
        self.pulldom = pulldom.SAX2DOM()
        self.parser.setFeature(xml.sax.handler.feature_namespaces, 1)
        self.parser.setContentHandler(self.pulldom)


# --- test body ---
"""Test DOMEventStream parsing semantics."""
items = pulldom.parseString(SMALL_SAMPLE)
evt, node = next(items)

assert hasattr(node, 'createElement')

assert pulldom.START_DOCUMENT == evt
evt, node = next(items)

assert pulldom.START_ELEMENT == evt

assert 'html' == node.tagName

assert 2 == len(node.attributes)

assert node.attributes.getNamedItem('xmlns:xdc').value == 'http://www.xml.com/books'
evt, node = next(items)

assert pulldom.CHARACTERS == evt
evt, node = next(items)

assert pulldom.CHARACTERS == evt
evt, node = next(items)

assert 'title' == node.tagName
title_node = node
evt, node = next(items)

assert pulldom.CHARACTERS == evt

assert 'Introduction to XSL' == node.data
evt, node = next(items)

assert pulldom.END_ELEMENT == evt

assert 'title' == node.tagName

assert title_node is node
evt, node = next(items)

assert pulldom.CHARACTERS == evt
evt, node = next(items)

assert pulldom.START_ELEMENT == evt

assert 'hr' == node.tagName
evt, node = next(items)

assert pulldom.END_ELEMENT == evt

assert 'hr' == node.tagName
evt, node = next(items)

assert pulldom.CHARACTERS == evt
evt, node = next(items)

assert pulldom.START_ELEMENT == evt

assert 'p' == node.tagName
evt, node = next(items)

assert pulldom.START_ELEMENT == evt

assert 'xdc:author' == node.tagName
evt, node = next(items)

assert pulldom.CHARACTERS == evt
evt, node = next(items)

assert pulldom.END_ELEMENT == evt

assert 'xdc:author' == node.tagName
evt, node = next(items)

assert pulldom.END_ELEMENT == evt
evt, node = next(items)

assert pulldom.CHARACTERS == evt
evt, node = next(items)

assert pulldom.END_ELEMENT == evt
print("PullDOMTestCase::test_parse_semantics: ok")
