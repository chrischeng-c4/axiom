# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "behavior"
# case = "parse_test__test_parse_string_bytes"
# subject = "cpython.test_sax.ParseTest.test_parseString_bytes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sax.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sax.py::ParseTest::test_parseString_bytes
"""Auto-ported test: ParseTest::test_parseString_bytes (CPython 3.12 oracle)."""


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
data = '<money value="$£€𐅻">$£€𐅻</money>'

def check_parse(f):
    from xml.sax import parse
    result = StringIO()
    parse(f, XMLGenerator(result, 'utf-8'))

    assert result.getvalue() == xml_str(data, 'utf-8')

def check_parseString(s):
    from xml.sax import parseString
    result = StringIO()
    parseString(s, XMLGenerator(result, 'utf-8'))

    assert result.getvalue() == xml_str(data, 'utf-8')
encodings = ('us-ascii', 'utf-8', 'utf-16', 'utf-16le', 'utf-16be')
for encoding in encodings:
    check_parseString(xml_bytes(data, encoding))
    check_parseString(xml_bytes(data, encoding, None))
check_parseString(xml_bytes(data, 'utf-8-sig', 'utf-8'))
check_parseString(xml_bytes(data, 'utf-8-sig', None))
check_parseString(xml_bytes(data, 'iso-8859-1'))
try:
    check_parseString(xml_bytes(data, 'iso-8859-1', None))
    raise AssertionError('expected SAXException')
except SAXException:
    pass
print("ParseTest::test_parseString_bytes: ok")
