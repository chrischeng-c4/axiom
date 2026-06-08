# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "buffer_text_test__test_default_to_disabled"
# subject = "cpython.test_pyexpat.BufferTextTest.test_default_to_disabled"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pyexpat.py::BufferTextTest::test_default_to_disabled
"""Auto-ported test: BufferTextTest::test_default_to_disabled (CPython 3.12 oracle)."""


import os
import platform
import sys
import sysconfig
import unittest
import traceback
from io import BytesIO
from test import support
from test.support import os_helper
from xml.parsers import expat
from xml.parsers.expat import errors
from test.support import sortdict, is_emscripten, is_wasi


data = b'<?xml version="1.0" encoding="iso-8859-1" standalone="no"?>\n<?xml-stylesheet href="stylesheet.css"?>\n<!-- comment data -->\n<!DOCTYPE quotations SYSTEM "quotations.dtd" [\n<!ELEMENT root ANY>\n<!ATTLIST root attr1 CDATA #REQUIRED attr2 CDATA #IMPLIED>\n<!NOTATION notation SYSTEM "notation.jpeg">\n<!ENTITY acirc "&#226;">\n<!ENTITY external_entity SYSTEM "entity.file">\n<!ENTITY unparsed_entity SYSTEM "entity.file" NDATA notation>\n%unparsed_entity;\n]>\n\n<root attr1="value1" attr2="value2&#8000;">\n<myns:subelement xmlns:myns="http://www.python.org/namespace">\n     Contents of subelements\n</myns:subelement>\n<sub2><![CDATA[contents of CDATA section]]></sub2>\n&external_entity;\n&skipped_entity;\n\xb5\n</root>\n'


# --- test body ---
def CharacterDataHandler(text):
    self_stuff.append(text)

def CommentHandler(data):
    self_stuff.append('<!--%s-->' % data)

def EndElementHandler(name):
    self_stuff.append('</%s>' % name)

def StartElementHandler(name, attrs):
    self_stuff.append('<%s>' % name)
    bt = attrs.get('buffer-text')
    if bt == 'yes':
        self_parser.buffer_text = 1
    elif bt == 'no':
        self_parser.buffer_text = 0

def check(expected, label):

    assert self_stuff == expected

def setHandlers(handlers=[]):
    for name in handlers:
        setattr(self_parser, name, getattr(self, name))

def test1():
    setHandlers(['StartElementHandler'])
    self_parser.Parse(b"<a>1<b buffer-text='no'/>2\n3<c buffer-text='yes'/>4\n5</a>", True)

    assert self_stuff == ['<a>', '1', '<b>', '2', '\n', '3', '<c>', '4\n5']

def test2():
    self_parser.Parse(b'<a>1<b/>&lt;2&gt;<c/>&#32;\n&#x20;3</a>', True)

    assert self_stuff == ['1<2> \n 3']

def test3():
    setHandlers(['StartElementHandler'])
    self_parser.Parse(b'<a>1<b/>2<c/>3</a>', True)

    assert self_stuff == ['<a>', '1', '<b>', '2', '<c>', '3']

def test4():
    setHandlers(['StartElementHandler', 'EndElementHandler'])
    self_parser.CharacterDataHandler = None
    self_parser.Parse(b'<a>1<b/>2<c/>3</a>', True)

    assert self_stuff == ['<a>', '<b>', '</b>', '<c>', '</c>', '</a>']

def test5():
    setHandlers(['StartElementHandler', 'EndElementHandler'])
    self_parser.Parse(b'<a>1<b></b>2<c/>3</a>', True)

    assert self_stuff == ['<a>', '1', '<b>', '</b>', '2', '<c>', '</c>', '3', '</a>']

def test6():
    setHandlers(['CommentHandler', 'EndElementHandler', 'StartElementHandler'])
    self_parser.Parse(b'<a>1<b/>2<c></c>345</a> ', True)

    assert self_stuff == ['<a>', '1', '<b>', '</b>', '2', '<c>', '</c>', '345', '</a>']

def test7():
    setHandlers(['CommentHandler', 'EndElementHandler', 'StartElementHandler'])
    self_parser.Parse(b'<a>1<b/>2<c></c>3<!--abc-->4<!--def-->5</a> ', True)

    assert self_stuff == ['<a>', '1', '<b>', '</b>', '2', '<c>', '</c>', '3', '<!--abc-->', '4', '<!--def-->', '5', '</a>']
self_stuff = []
self_parser = expat.ParserCreate()
self_parser.buffer_text = 1
self_parser.CharacterDataHandler = CharacterDataHandler
parser = expat.ParserCreate()

assert not parser.buffer_text
print("BufferTextTest::test_default_to_disabled: ok")
