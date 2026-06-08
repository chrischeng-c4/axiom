# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pulldom"
# dimension = "behavior"
# case = "sax2_dom_test_case__test_sax2_dom"
# subject = "cpython.test_pulldom.SAX2DOMTestCase.testSAX2DOM"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pulldom.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pulldom.py::SAX2DOMTestCase::testSAX2DOM
"""Auto-ported test: SAX2DOMTestCase::testSAX2DOM (CPython 3.12 oracle)."""


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
def confirm(test, testname='Test'):

    assert test
'Ensure SAX2DOM expands nodes as expected.'
sax2dom = pulldom.SAX2DOM()
sax2dom.startDocument()
sax2dom.startElement('doc', {})
sax2dom.characters('text')
sax2dom.startElement('subelm', {})
sax2dom.characters('text')
sax2dom.endElement('subelm')
sax2dom.characters('text')
sax2dom.endElement('doc')
sax2dom.endDocument()
doc = sax2dom.document
root = doc.documentElement
text1, elm1, text2 = root.childNodes
text3 = elm1.childNodes[0]

assert text1.previousSibling is None

assert text1.nextSibling is elm1

assert elm1.previousSibling is text1

assert elm1.nextSibling is text2

assert text2.previousSibling is elm1

assert text2.nextSibling is None

assert text3.previousSibling is None

assert text3.nextSibling is None

assert root.parentNode is doc

assert text1.parentNode is root

assert elm1.parentNode is root

assert text2.parentNode is root

assert text3.parentNode is elm1
doc.unlink()
print("SAX2DOMTestCase::testSAX2DOM: ok")
