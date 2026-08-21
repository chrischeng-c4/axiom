# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "behavior"
# case = "minidom_test__test_minidom_attribute_order"
# subject = "cpython.test_minidom.MinidomTest.test_minidom_attribute_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_minidom.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_minidom.py::MinidomTest::test_minidom_attribute_order
"""Auto-ported test: MinidomTest::test_minidom_attribute_order (CPython 3.12 oracle)."""


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
xml_str = '<?xml version="1.0" ?><curriculum status="public" company="example"/>'
doc = parseString(xml_str)
output = io.StringIO()
doc.writexml(output)

assert output.getvalue() == xml_str
print("MinidomTest::test_minidom_attribute_order: ok")
