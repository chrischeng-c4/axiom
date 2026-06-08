# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "behavior"
# case = "make_parser_test__test_make_parser5"
# subject = "cpython.test_sax.MakeParserTest.test_make_parser5"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sax.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sax.py::MakeParserTest::test_make_parser5
"""Auto-ported test: MakeParserTest::test_make_parser5 (CPython 3.12 oracle)."""


from xml.sax import make_parser, ContentHandler, SAXException, SAXReaderNotAvailable, SAXParseException
import unittest
from unittest import mock
from xml.sax.saxutils import XMLGenerator, escape, unescape, quoteattr, XMLFilterBase, prepare_input_source
from xml.sax.expatreader import create_parser
from xml.sax.handler import feature_namespaces, feature_external_ges, LexicalHandler
from xml.sax.xmlreader import InputSource, AttributesImpl, AttributesNSImpl
from io import BytesIO, StringIO
import codecs
import os.path
import pyexpat
import shutil
import sys
from urllib.error import URLError
import urllib.request
from test.support import os_helper
from test.support import findfile
from test.support.os_helper import FakePath, TESTFN


try:
    make_parser()
except SAXReaderNotAvailable:
    raise unittest.SkipTest('no XML parsers available')

TEST_XMLFILE = findfile('test.xml', subdir='xmltestdata')

TEST_XMLFILE_OUT = findfile('test.xml.out', subdir='xmltestdata')

try:
    TEST_XMLFILE.encode('utf-8')
    TEST_XMLFILE_OUT.encode('utf-8')
except UnicodeEncodeError:
    raise unittest.SkipTest('filename is not encodable to utf8')

supports_nonascii_filenames = True

if not os.path.supports_unicode_filenames:
    try:
        os_helper.TESTFN_UNICODE.encode(sys.getfilesystemencoding())
    except (UnicodeError, TypeError):
        supports_nonascii_filenames = False

requires_nonascii_filenames = unittest.skipUnless(supports_nonascii_filenames, 'Requires non-ascii filenames support')

ns_uri = 'http://www.python.org/xml-ns/saxtest/'

def xml_str(doc, encoding=None):
    if encoding is None:
        return doc
    return '<?xml version="1.0" encoding="%s"?>\n%s' % (encoding, doc)

def xml_bytes(doc, encoding, decl_encoding=...):
    if decl_encoding is ...:
        decl_encoding = encoding
    return xml_str(doc, decl_encoding).encode(encoding, 'xmlcharrefreplace')

def make_xml_file(doc, encoding, decl_encoding=...):
    if decl_encoding is ...:
        decl_encoding = encoding
    with open(TESTFN, 'w', encoding=encoding, errors='xmlcharrefreplace') as f:
        f.write(xml_str(doc, decl_encoding))

start = b'<?xml version="1.0" encoding="iso-8859-1"?>\n'

with open(TEST_XMLFILE_OUT, 'rb') as f:
    xml_test_out = f.read()


# --- test body ---
make_parser(['module1', 'module2'])
make_parser(('module1', 'module2'))
make_parser({'module1', 'module2'})
make_parser(frozenset({'module1', 'module2'}))
make_parser({'module1': None, 'module2': None})
make_parser(iter(['module1', 'module2']))
print("MakeParserTest::test_make_parser5: ok")
