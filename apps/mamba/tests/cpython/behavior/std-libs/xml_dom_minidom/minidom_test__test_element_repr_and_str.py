# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_element_repr_and_str"
# subject = "cpython.test_minidom.MinidomTest.testElementReprAndStr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::testElementReprAndStr
"""Auto-ported test: MinidomTest::testElementReprAndStr (CPython 3.12 oracle)."""


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
dom = Document()
el = dom.appendChild(dom.createElement('abc'))
string1 = repr(el)
string2 = str(el)
confirm(string1 == string2)
dom.unlink()
print("MinidomTest::testElementReprAndStr: ok")
