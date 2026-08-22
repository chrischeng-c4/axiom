use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_getitem.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_getitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_getitem"
# subject = "cpython.test_minidom.MinidomTest.testAttrList__getitem__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrList__getitem__
"""Auto-ported test: MinidomTest::testAttrList__getitem__ (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrList__getitem__: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrList__getitem__: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_item.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_item() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_item"
# subject = "cpython.test_minidom.MinidomTest.testAttrListItem"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrListItem
"""Auto-ported test: MinidomTest::testAttrListItem (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrListItem: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrListItem: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_item_ns.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_item_ns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_item_ns"
# subject = "cpython.test_minidom.MinidomTest.testAttrListItemNS"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrListItemNS
"""Auto-ported test: MinidomTest::testAttrListItemNS (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrListItemNS: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrListItemNS: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_items.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_items() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_items"
# subject = "cpython.test_minidom.MinidomTest.testAttrListItems"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrListItems
"""Auto-ported test: MinidomTest::testAttrListItems (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrListItems: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrListItems: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_keys.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_keys() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_keys"
# subject = "cpython.test_minidom.MinidomTest.testAttrListKeys"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrListKeys
"""Auto-ported test: MinidomTest::testAttrListKeys (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrListKeys: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrListKeys: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_keys_ns.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_keys_ns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_keys_ns"
# subject = "cpython.test_minidom.MinidomTest.testAttrListKeysNS"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrListKeysNS
"""Auto-ported test: MinidomTest::testAttrListKeysNS (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrListKeysNS: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrListKeysNS: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_length.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_length"
# subject = "cpython.test_minidom.MinidomTest.testAttrListLength"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrListLength
"""Auto-ported test: MinidomTest::testAttrListLength (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrListLength: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrListLength: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_setitem.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_setitem() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_setitem"
# subject = "cpython.test_minidom.MinidomTest.testAttrList__setitem__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrList__setitem__
"""Auto-ported test: MinidomTest::testAttrList__setitem__ (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrList__setitem__: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrList__setitem__: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_attr_list_values.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_attr_list_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_attr_list_values"
# subject = "cpython.test_minidom.MinidomTest.testAttrListValues"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testAttrListValues
"""Auto-ported test: MinidomTest::testAttrListValues (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testAttrListValues: ok")
"###);
    assert_output(&out, r###"MinidomTest::testAttrListValues: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_child_nodes.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_child_nodes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_child_nodes"
# subject = "cpython.test_minidom.MinidomTest.testChildNodes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testChildNodes
"""Auto-ported test: MinidomTest::testChildNodes (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testChildNodes: ok")
"###);
    assert_output(&out, r###"MinidomTest::testChildNodes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_comment.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_comment() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_comment"
# subject = "cpython.test_minidom.MinidomTest.testComment"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testComment
"""Auto-ported test: MinidomTest::testComment (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testComment: ok")
"###);
    assert_output(&out, r###"MinidomTest::testComment: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_create_attribute_ns.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_create_attribute_ns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_create_attribute_ns"
# subject = "cpython.test_minidom.MinidomTest.testCreateAttributeNS"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testCreateAttributeNS
"""Auto-ported test: MinidomTest::testCreateAttributeNS (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testCreateAttributeNS: ok")
"###);
    assert_output(&out, r###"MinidomTest::testCreateAttributeNS: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_create_element_ns.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_create_element_ns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_create_element_ns"
# subject = "cpython.test_minidom.MinidomTest.testCreateElementNS"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testCreateElementNS
"""Auto-ported test: MinidomTest::testCreateElementNS (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testCreateElementNS: ok")
"###);
    assert_output(&out, r###"MinidomTest::testCreateElementNS: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_document_element.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_document_element() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_document_element"
# subject = "cpython.test_minidom.MinidomTest.testDocumentElement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testDocumentElement
"""Auto-ported test: MinidomTest::testDocumentElement (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testDocumentElement: ok")
"###);
    assert_output(&out, r###"MinidomTest::testDocumentElement: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_first_child.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_first_child() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_first_child"
# subject = "cpython.test_minidom.MinidomTest.testFirstChild"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testFirstChild
"""Auto-ported test: MinidomTest::testFirstChild (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testFirstChild: ok")
"###);
    assert_output(&out, r###"MinidomTest::testFirstChild: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_get_attr_length.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_get_attr_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_get_attr_length"
# subject = "cpython.test_minidom.MinidomTest.testGetAttrLength"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testGetAttrLength
"""Auto-ported test: MinidomTest::testGetAttrLength (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testGetAttrLength: ok")
"###);
    assert_output(&out, r###"MinidomTest::testGetAttrLength: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_get_attr_list.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_get_attr_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_get_attr_list"
# subject = "cpython.test_minidom.MinidomTest.testGetAttrList"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testGetAttrList
"""Auto-ported test: MinidomTest::testGetAttrList (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testGetAttrList: ok")
"###);
    assert_output(&out, r###"MinidomTest::testGetAttrList: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_get_attr_values.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_get_attr_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_get_attr_values"
# subject = "cpython.test_minidom.MinidomTest.testGetAttrValues"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testGetAttrValues
"""Auto-ported test: MinidomTest::testGetAttrValues (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testGetAttrValues: ok")
"###);
    assert_output(&out, r###"MinidomTest::testGetAttrValues: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_get_attribute_node.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_get_attribute_node() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_get_attribute_node"
# subject = "cpython.test_minidom.MinidomTest.testGetAttributeNode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testGetAttributeNode
"""Auto-ported test: MinidomTest::testGetAttributeNode (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testGetAttributeNode: ok")
"###);
    assert_output(&out, r###"MinidomTest::testGetAttributeNode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_parse.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_parse() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_parse"
# subject = "cpython.test_minidom.MinidomTest.testParse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testParse
"""Auto-ported test: MinidomTest::testParse (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testParse: ok")
"###);
    assert_output(&out, r###"MinidomTest::testParse: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_parse_attribute_namespaces.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_parse_attribute_namespaces() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_parse_attribute_namespaces"
# subject = "cpython.test_minidom.MinidomTest.testParseAttributeNamespaces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testParseAttributeNamespaces
"""Auto-ported test: MinidomTest::testParseAttributeNamespaces (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testParseAttributeNamespaces: ok")
"###);
    assert_output(&out, r###"MinidomTest::testParseAttributeNamespaces: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_parse_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_parse_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_parse_attributes"
# subject = "cpython.test_minidom.MinidomTest.testParseAttributes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testParseAttributes
"""Auto-ported test: MinidomTest::testParseAttributes (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testParseAttributes: ok")
"###);
    assert_output(&out, r###"MinidomTest::testParseAttributes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_parse_element.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_parse_element() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_parse_element"
# subject = "cpython.test_minidom.MinidomTest.testParseElement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testParseElement
"""Auto-ported test: MinidomTest::testParseElement (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testParseElement: ok")
"###);
    assert_output(&out, r###"MinidomTest::testParseElement: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_parse_element_namespaces.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_parse_element_namespaces() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_parse_element_namespaces"
# subject = "cpython.test_minidom.MinidomTest.testParseElementNamespaces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testParseElementNamespaces
"""Auto-ported test: MinidomTest::testParseElementNamespaces (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testParseElementNamespaces: ok")
"###);
    assert_output(&out, r###"MinidomTest::testParseElementNamespaces: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_parse_processing_instructions.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_parse_processing_instructions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_parse_processing_instructions"
# subject = "cpython.test_minidom.MinidomTest.testParseProcessingInstructions"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testParseProcessingInstructions
"""Auto-ported test: MinidomTest::testParseProcessingInstructions (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testParseProcessingInstructions: ok")
"###);
    assert_output(&out, r###"MinidomTest::testParseProcessingInstructions: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_parse_string.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_parse_string() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_parse_string"
# subject = "cpython.test_minidom.MinidomTest.testParseString"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testParseString
"""Auto-ported test: MinidomTest::testParseString (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testParseString: ok")
"###);
    assert_output(&out, r###"MinidomTest::testParseString: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_processing_instruction_repr.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_processing_instruction_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_processing_instruction_repr"
# subject = "cpython.test_minidom.MinidomTest.testProcessingInstructionRepr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testProcessingInstructionRepr
"""Auto-ported test: MinidomTest::testProcessingInstructionRepr (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testProcessingInstructionRepr: ok")
"###);
    assert_output(&out, r###"MinidomTest::testProcessingInstructionRepr: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_set_attr_valueand_node_value.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_set_attr_valueand_node_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_set_attr_valueand_node_value"
# subject = "cpython.test_minidom.MinidomTest.testSetAttrValueandNodeValue"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testSetAttrValueandNodeValue
"""Auto-ported test: MinidomTest::testSetAttrValueandNodeValue (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testSetAttrValueandNodeValue: ok")
"###);
    assert_output(&out, r###"MinidomTest::testSetAttrValueandNodeValue: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_text_node_repr.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_text_node_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_text_node_repr"
# subject = "cpython.test_minidom.MinidomTest.testTextNodeRepr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testTextNodeRepr
"""Auto-ported test: MinidomTest::testTextNodeRepr (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testTextNodeRepr: ok")
"###);
    assert_output(&out, r###"MinidomTest::testTextNodeRepr: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_text_repr.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_text_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_text_repr"
# subject = "cpython.test_minidom.MinidomTest.testTextRepr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testTextRepr
"""Auto-ported test: MinidomTest::testTextRepr (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testTextRepr: ok")
"###);
    assert_output(&out, r###"MinidomTest::testTextRepr: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/xml_dom_minidom/minidom_test__test_write_text.py`.
#[test]
fn test_gen_behavior_std_libs_xml_dom_minidom_minidom_test__test_write_text() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_write_text"
# subject = "cpython.test_minidom.MinidomTest.testWriteText"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testWriteText
"""Auto-ported test: MinidomTest::testWriteText (CPython 3.12 oracle)."""


import copy
import pickle
import io
from test import support
import unittest
import xml.dom.minidom
from xml.dom.minidom import parse, Attr, Node, Document, parseString
from xml.dom.minidom import getDOMImplementation
from xml.parsers.expat import ExpatError


tstfile = support.findfile('test.xml', subdir='xmltestdata')

sample = "<?xml version='1.0' encoding='us-ascii'?>\n<!DOCTYPE doc PUBLIC 'http://xml.python.org/public' 'http://xml.python.org/system' [\n  <!ELEMENT e EMPTY>\n  <!ENTITY ent SYSTEM 'http://xml.python.org/entity'>\n]><doc attr='value'> text\n<?pi sample?> <!-- comment --> <e/> </doc>"

def create_doc_without_doctype(doctype=None):
    return getDOMImplementation().createDocument(None, 'doc', doctype)

def create_nonempty_doctype():
    doctype = getDOMImplementation().createDocumentType('doc', None, None)
    doctype.entities._seq = []
    doctype.notations._seq = []
    notation = xml.dom.minidom.Notation('my-notation', None, 'http://xml.python.org/notations/my')
    doctype.notations._seq.append(notation)
    entity = xml.dom.minidom.Entity('my-entity', None, 'http://xml.python.org/entities/my', 'my-notation')
    entity.version = '1.0'
    entity.encoding = 'utf-8'
    entity.actualEncoding = 'us-ascii'
    doctype.entities._seq.append(entity)
    return doctype

def create_doc_with_doctype():
    doctype = create_nonempty_doctype()
    doc = create_doc_without_doctype(doctype)
    doctype.entities.item(0).ownerDocument = doc
    doctype.notations.item(0).ownerDocument = doc
    return doc


# --- test body ---
def _create_fragment_test_nodes():
    dom = parseString('<doc/>')
    orig = dom.createTextNode('original')
    c1 = dom.createTextNode('foo')
    c2 = dom.createTextNode('bar')
    c3 = dom.createTextNode('bat')
    dom.documentElement.appendChild(orig)
    frag = dom.createDocumentFragment()
    frag.appendChild(c1)
    frag.appendChild(c2)
    frag.appendChild(c3)
    return (dom, orig, c1, c2, c3, frag)

def _setupCloneElement(deep):
    dom = parseString("<doc attr='value'><foo/></doc>")
    root = dom.documentElement
    clone = root.cloneNode(deep)
    _testCloneElementCopiesAttributes(root, clone, 'testCloneElement' + (deep and 'Deep' or 'Shallow'))
    root.tagName = root.nodeName = 'MODIFIED'
    root.setAttribute('attr', 'NEW VALUE')
    root.setAttribute('added', 'VALUE')
    return (dom, clone)

def _testCloneElementCopiesAttributes(e1, e2, test):
    attrs1 = e1.attributes
    attrs2 = e2.attributes
    keys1 = list(attrs1.keys())
    keys2 = list(attrs2.keys())
    keys1.sort()
    keys2.sort()
    confirm(keys1 == keys2, 'clone of element has same attribute keys')
    for i in range(len(keys1)):
        a1 = attrs1.item(i)
        a2 = attrs2.item(i)
        confirm(a1 is not a2 and a1.value == a2.value and (a1.nodeValue == a2.nodeValue) and (a1.namespaceURI == a2.namespaceURI) and (a1.localName == a2.localName), 'clone of attribute node has proper attribute values')
        confirm(a2.ownerElement is e2, 'clone of attribute node correctly owned')

def assert_recursive_equal(doc, doc2):
    stack = [(doc, doc2)]
    while stack:
        n1, n2 = stack.pop()

        assert n1.nodeType == n2.nodeType

        assert len(n1.childNodes) == len(n2.childNodes)

        assert n1.nodeName == n2.nodeName

        assert not n1.isSameNode(n2)

        assert not n2.isSameNode(n1)
        if n1.nodeType == Node.DOCUMENT_TYPE_NODE:
            len(n1.entities)
            len(n2.entities)
            len(n1.notations)
            len(n2.notations)

            assert len(n1.entities) == len(n2.entities)

            assert len(n1.notations) == len(n2.notations)
            for i in range(len(n1.notations)):
                no1 = n1.notations.item(i)
                no2 = n1.notations.item(i)

                assert no1.name == no2.name

                assert no1.publicId == no2.publicId

                assert no1.systemId == no2.systemId
                stack.append((no1, no2))
            for i in range(len(n1.entities)):
                e1 = n1.entities.item(i)
                e2 = n2.entities.item(i)

                assert e1.notationName == e2.notationName

                assert e1.publicId == e2.publicId

                assert e1.systemId == e2.systemId
                stack.append((e1, e2))
        if n1.nodeType != Node.DOCUMENT_NODE:

            assert n1.ownerDocument.isSameNode(doc)

            assert n2.ownerDocument.isSameNode(doc2)
        for i in range(len(n1.childNodes)):
            stack.append((n1.childNodes[i], n2.childNodes[i]))

def checkRenameNodeSharedConstraints(doc, node):

    try:
        doc.renameNode(node, 'http://xml.python.org/ns', 'xmlns:foo')
        raise AssertionError('expected xml.dom.NamespaceErr')
    except xml.dom.NamespaceErr:
        pass
    doc2 = parseString('<doc/>')

    try:
        doc2.renameNode(node, xml.dom.EMPTY_NAMESPACE, 'foo')
        raise AssertionError('expected xml.dom.WrongDocumentErr')
    except xml.dom.WrongDocumentErr:
        pass

def checkWholeText(node, s):
    t = node.wholeText
    confirm(t == s, 'looking for %r, found %r' % (s, t))

def check_clone_attribute(deep, testName):
    doc = parseString("<doc attr='value'/>")
    attr = doc.documentElement.getAttributeNode('attr')

    assert attr != None
    clone = attr.cloneNode(deep)
    confirm(not clone.isSameNode(attr))
    confirm(not attr.isSameNode(clone))
    confirm(clone.ownerElement is None, testName + ': ownerElement should be None')
    confirm(clone.ownerDocument.isSameNode(attr.ownerDocument), testName + ': ownerDocument does not match')
    confirm(clone.specified, testName + ': cloned attribute must have specified == True')

def check_clone_node_entity(clone_document):
    document = xml.dom.minidom.parseString('\n            <?xml version="1.0" ?>\n            <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN"\n                "http://www.w3.org/TR/html4/strict.dtd"\n                [ <!ENTITY smile "☺"> ]\n            >\n            <doc>Don\'t let entities make you frown &smile;</doc>\n        '.strip())

    class Handler:

        def handle(self, operation, key, data, src, dst):
            self.operation = operation
            self.key = key
            self.data = data
            self.src = src
            self.dst = dst
    handler = Handler()
    doctype = document.doctype
    entity = doctype.entities['smile']
    entity.setUserData('key', 'data', handler)
    if clone_document:
        clone = document.cloneNode(deep=True)

        assert clone.documentElement.firstChild.wholeText == "Don't let entities make you frown ☺"
        operation = xml.dom.UserDataHandler.NODE_IMPORTED
        dst = clone.doctype.entities['smile']
    else:
        with support.swap_attr(doctype, 'ownerDocument', None):
            clone = doctype.cloneNode(deep=True)
        operation = xml.dom.UserDataHandler.NODE_CLONED
        dst = clone.entities['smile']

    assert handler.operation == operation

    assert handler.key == 'key'

    assert handler.data == 'data'

    assert handler.src is entity

    assert handler.dst is dst

def check_clone_pi(deep, testName):
    doc = parseString('<?target data?><doc/>')
    pi = doc.firstChild

    assert pi.nodeType == Node.PROCESSING_INSTRUCTION_NODE
    clone = pi.cloneNode(deep)
    confirm(clone.target == pi.target and clone.data == pi.data)

def check_import_document(deep, testName):
    doc1 = parseString('<doc/>')
    doc2 = parseString('<doc/>')

    try:
        doc1.importNode(doc2, deep)
        raise AssertionError('expected xml.dom.NotSupportedErr')
    except xml.dom.NotSupportedErr:
        pass

def confirm(test, testname='Test'):

    assert test

def get_empty_nodelist_from_elements_by_tagName_ns_helper(doc, nsuri, lname):
    nodelist = doc.getElementsByTagNameNS(nsuri, lname)
    confirm(len(nodelist) == 0)
pass
print("MinidomTest::testWriteText: ok")
"###);
    assert_output(&out, r###"MinidomTest::testWriteText: ok
"###);
}
