# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "behavior"
# case = "expat_reader_test__test_expat_binary_file"
# subject = "cpython.test_sax.ExpatReaderTest.test_expat_binary_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sax.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sax.py::ExpatReaderTest::test_expat_binary_file
"""Auto-ported test: ExpatReaderTest::test_expat_binary_file (CPython 3.12 oracle)."""


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
def verify_attrs_wattr(attrs):

    assert attrs.getLength() == 1

    assert attrs.getNames() == ['attr']

    assert attrs.getQNames() == ['attr']

    assert len(attrs) == 1

    assert 'attr' in attrs

    assert list(attrs.keys()) == ['attr']

    assert attrs.get('attr') == 'val'

    assert attrs.get('attr', 25) == 'val'

    assert list(attrs.items()) == [('attr', 'val')]

    assert list(attrs.values()) == ['val']

    assert attrs.getValue('attr') == 'val'

    assert attrs.getValueByQName('attr') == 'val'

    assert attrs.getNameByQName('attr') == 'attr'

    assert attrs['attr'] == 'val'

    assert attrs.getQNameByName('attr') == 'attr'

def verify_empty_attrs(attrs):

    try:
        attrs.getValue('attr')
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    try:
        attrs.getValueByQName('attr')
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    try:
        attrs.getNameByQName('attr')
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    try:
        attrs.getQNameByName('attr')
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    try:
        attrs.__getitem__('attr')
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    assert attrs.getLength() == 0

    assert attrs.getNames() == []

    assert attrs.getQNames() == []

    assert len(attrs) == 0

    assert 'attr' not in attrs

    assert list(attrs.keys()) == []

    assert attrs.get('attrs') == None

    assert attrs.get('attrs', 25) == 25

    assert list(attrs.items()) == []

    assert list(attrs.values()) == []

def verify_empty_nsattrs(attrs):

    try:
        attrs.getValue((ns_uri, 'attr'))
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    try:
        attrs.getValueByQName('ns:attr')
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    try:
        attrs.getNameByQName('ns:attr')
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    try:
        attrs.getQNameByName((ns_uri, 'attr'))
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    try:
        attrs.__getitem__((ns_uri, 'attr'))
        raise AssertionError('expected KeyError')
    except KeyError:
        pass

    assert attrs.getLength() == 0

    assert attrs.getNames() == []

    assert attrs.getQNames() == []

    assert len(attrs) == 0

    assert (ns_uri, 'attr') not in attrs

    assert list(attrs.keys()) == []

    assert attrs.get((ns_uri, 'attr')) == None

    assert attrs.get((ns_uri, 'attr'), 25) == 25

    assert list(attrs.items()) == []

    assert list(attrs.values()) == []
parser = create_parser()
result = BytesIO()
xmlgen = XMLGenerator(result)
parser.setContentHandler(xmlgen)
with open(TEST_XMLFILE, 'rb') as f:
    parser.parse(f)

assert result.getvalue() == xml_test_out
print("ExpatReaderTest::test_expat_binary_file: ok")
