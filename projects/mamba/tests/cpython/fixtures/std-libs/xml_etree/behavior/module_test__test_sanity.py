# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "module_test__test_sanity"
# subject = "cpython.test_xml_etree.ModuleTest.test_sanity"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_etree.py::ModuleTest::test_sanity
"""Auto-ported test: ModuleTest::test_sanity (CPython 3.12 oracle)."""


import copy
import functools
import html
import io
import itertools
import operator
import os
import pickle
import pyexpat
import sys
import textwrap
import types
import unittest
import unittest.mock as mock
import warnings
import weakref
from contextlib import nullcontext
from functools import partial
from itertools import product, islice
from test import support
from test.support import os_helper
from test.support import warnings_helper
from test.support import findfile, gc_collect, swap_attr, swap_item
from test.support.import_helper import import_fresh_module
from test.support.os_helper import TESTFN


pyET = None

ET = None

SIMPLE_XMLFILE = findfile('simple.xml', subdir='xmltestdata')

try:
    SIMPLE_XMLFILE.encode('utf-8')
except UnicodeEncodeError:
    raise unittest.SkipTest('filename is not encodable to utf8')

SIMPLE_NS_XMLFILE = findfile('simple-ns.xml', subdir='xmltestdata')

UTF8_BUG_XMLFILE = findfile('expat224_utf8_bug.xml', subdir='xmltestdata')

SAMPLE_XML = "<body>\n  <tag class='a'>text</tag>\n  <tag class='b' />\n  <section>\n    <tag class='b' id='inner'>subtext</tag>\n  </section>\n</body>\n"

SAMPLE_SECTION = "<section>\n  <tag class='b' id='inner'>subtext</tag>\n  <nexttag />\n  <nextsection>\n    <tag />\n  </nextsection>\n</section>\n"

SAMPLE_XML_NS = '\n<body xmlns="http://effbot.org/ns">\n  <tag>text</tag>\n  <tag />\n  <section>\n    <tag>subtext</tag>\n  </section>\n</body>\n'

SAMPLE_XML_NS_ELEMS = '\n<root>\n<h:table xmlns:h="hello">\n  <h:tr>\n    <h:td>Apples</h:td>\n    <h:td>Bananas</h:td>\n  </h:tr>\n</h:table>\n\n<f:table xmlns:f="foo">\n  <f:name>African Coffee Table</f:name>\n  <f:width>80</f:width>\n  <f:length>120</f:length>\n</f:table>\n</root>\n'

ENTITY_XML = "<!DOCTYPE points [\n<!ENTITY % user-entities SYSTEM 'user-entities.xml'>\n%user-entities;\n]>\n<document>&entity;</document>\n"

EXTERNAL_ENTITY_XML = '<!DOCTYPE points [\n<!ENTITY entity SYSTEM "file:///non-existing-file.xml">\n]>\n<document>&entity;</document>\n'

ATTLIST_XML = '<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE Foo [\n<!ELEMENT foo (bar*)>\n<!ELEMENT bar (#PCDATA)*>\n<!ATTLIST bar xml:lang CDATA "eng">\n<!ENTITY qux "quux">\n]>\n<foo>\n<bar>&qux;</bar>\n</foo>\n'

def is_python_implementation():
    assert ET is not None, 'ET must be initialized'
    assert pyET is not None, 'pyET must be initialized'
    return ET is pyET

def equal_wrapper(cls):
    """Mock cls.__eq__ to check whether it has been called or not.

    The behaviour of cls.__eq__ (side-effects included) is left as is.
    """
    eq = cls.__eq__
    return mock.patch.object(cls, '__eq__', autospec=True, wraps=eq)

def checkwarnings(*filters, quiet=False):

    def decorator(test):

        def newtest(*args, **kwargs):
            with warnings_helper.check_warnings(*filters, quiet=quiet):
                test(*args, **kwargs)
        functools.update_wrapper(newtest, test)
        return newtest
    return decorator

def convlinesep(data):
    return data.replace(b'\n', os.linesep.encode())

def serialize(elem, to_string=True, encoding='unicode', **options):
    if encoding != 'unicode':
        file = io.BytesIO()
    else:
        file = io.StringIO()
    tree = ET.ElementTree(elem)
    tree.write(file, encoding=encoding, **options)
    if to_string:
        return file.getvalue()
    else:
        file.seek(0)
        return file

def summarize_list(seq):
    return [elem.tag for elem in seq]

class ElementTestCase:

    @classmethod
    def setUpClass(cls):
        cls.modules = {pyET, ET}

    def pickleRoundTrip(self, obj, name, dumper, loader, proto):
        try:
            with swap_item(sys.modules, name, dumper):
                temp = pickle.dumps(obj, proto)
            with swap_item(sys.modules, name, loader):
                result = pickle.loads(temp)
        except pickle.PicklingError as pe:
            human = dict([(ET, 'cET'), (pyET, 'pyET')])
            raise support.TestFailed('Failed to round-trip %r from %r to %r' % (obj, human.get(dumper, dumper), human.get(loader, loader))) from pe
        return result

    def assertEqualElements(self, alice, bob):
        self.assertIsInstance(alice, (ET.Element, pyET.Element))
        self.assertIsInstance(bob, (ET.Element, pyET.Element))
        self.assertEqual(len(list(alice)), len(list(bob)))
        for x, y in zip(alice, bob):
            self.assertEqualElements(x, y)
        properties = operator.attrgetter('tag', 'tail', 'text', 'attrib')
        self.assertEqual(properties(alice), properties(bob))

XINCLUDE = {}

XINCLUDE['C1.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>120 Mz is adequate for an average home user.</p>\n  <xi:include href="disclaimer.xml"/>\n</document>\n'

XINCLUDE['disclaimer.xml'] = "<?xml version='1.0'?>\n<disclaimer>\n  <p>The opinions represented herein represent those of the individual\n  and should not be interpreted as official policy endorsed by this\n  organization.</p>\n</disclaimer>\n"

XINCLUDE['C2.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>This document has been accessed\n  <xi:include href="count.txt" parse="text"/> times.</p>\n</document>\n'

XINCLUDE['count.txt'] = '324387'

XINCLUDE['C2b.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>This document has been <em>accessed</em>\n  <xi:include href="count.txt" parse="text"/> times.</p>\n</document>\n'

XINCLUDE['C3.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>The following is the source of the "data.xml" resource:</p>\n  <example><xi:include href="data.xml" parse="text"/></example>\n</document>\n'

XINCLUDE['data.xml'] = "<?xml version='1.0'?>\n<data>\n  <item><![CDATA[Brooks & Shields]]></item>\n</data>\n"

XINCLUDE['C5.xml'] = '<?xml version=\'1.0\'?>\n<div xmlns:xi="http://www.w3.org/2001/XInclude">\n  <xi:include href="example.txt" parse="text">\n    <xi:fallback>\n      <xi:include href="fallback-example.txt" parse="text">\n        <xi:fallback><a href="mailto:bob@example.org">Report error</a></xi:fallback>\n      </xi:include>\n    </xi:fallback>\n  </xi:include>\n</div>\n'

XINCLUDE['default.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>Example.</p>\n  <xi:include href="{}"/>\n</document>\n'.format(html.escape(SIMPLE_XMLFILE, True))

XINCLUDE['include_c1_repeated.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>The following is the source code of Recursive1.xml:</p>\n  <xi:include href="C1.xml"/>\n  <xi:include href="C1.xml"/>\n  <xi:include href="C1.xml"/>\n  <xi:include href="C1.xml"/>\n</document>\n'

XINCLUDE_BAD = {}

XINCLUDE_BAD['B1.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>120 Mz is adequate for an average home user.</p>\n  <xi:include href="disclaimer.xml" parse="BAD_TYPE"/>\n</document>\n'

XINCLUDE_BAD['B2.xml'] = '<?xml version=\'1.0\'?>\n<div xmlns:xi="http://www.w3.org/2001/XInclude">\n    <xi:fallback></xi:fallback>\n</div>\n'

XINCLUDE['Recursive1.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>The following is the source code of Recursive2.xml:</p>\n  <xi:include href="Recursive2.xml"/>\n</document>\n'

XINCLUDE['Recursive2.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>The following is the source code of Recursive3.xml:</p>\n  <xi:include href="Recursive3.xml"/>\n</document>\n'

XINCLUDE['Recursive3.xml'] = '<?xml version=\'1.0\'?>\n<document xmlns:xi="http://www.w3.org/2001/XInclude">\n  <p>The following is the source code of Recursive1.xml:</p>\n  <xi:include href="Recursive1.xml"/>\n</document>\n'

class MutationDeleteElementPath(str):

    def __new__(cls, elem, *args):
        self = str.__new__(cls, *args)
        self.elem = elem
        return self

    def __eq__(self, o):
        del self.elem[:]
        return True
    __hash__ = str.__hash__

class MutationClearElementPath(str):

    def __new__(cls, elem, *args):
        self = str.__new__(cls, *args)
        self.elem = elem
        return self

    def __eq__(self, o):
        self.elem.clear()
        return True
    __hash__ = str.__hash__

class BadElementPath(str):

    def __eq__(self, o):
        raise 1 / 0
    __hash__ = str.__hash__

def c14n_roundtrip(xml, **options):
    return pyET.canonicalize(xml, **options)

def setUpModule(module=None):
    global pyET
    pyET = import_fresh_module('xml.etree.ElementTree', blocked=['_elementtree'])
    if module is None:
        module = pyET
    global ET
    ET = module

    def cleanup():
        global ET, pyET
        ET = pyET = None
    unittest.addModuleCleanup(cleanup)
    from xml.etree import ElementPath
    nsmap = ET.register_namespace._namespace_map
    nsmap_copy = nsmap.copy()
    unittest.addModuleCleanup(nsmap.update, nsmap_copy)
    unittest.addModuleCleanup(nsmap.clear)
    path_cache = ElementPath._cache
    unittest.addModuleCleanup(setattr, ElementPath, '_cache', path_cache)
    ElementPath._cache = path_cache.copy()
    if hasattr(ET, '_set_factories'):
        old_factories = ET._set_factories(ET.Comment, ET.PI)
        unittest.addModuleCleanup(ET._set_factories, *old_factories)


# --- test body ---
from xml.etree import ElementTree
from xml.etree import ElementInclude
from xml.etree import ElementPath
print("ModuleTest::test_sanity: ok")
