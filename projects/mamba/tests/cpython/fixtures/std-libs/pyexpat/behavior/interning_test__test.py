# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "interning_test__test"
# subject = "cpython.test_pyexpat.InterningTest.test"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pyexpat.py::InterningTest::test
"""Auto-ported test: InterningTest::test (CPython 3.12 oracle)."""


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
p = expat.ParserCreate()
L = []

def collector(name, *args):
    L.append(name)
p.StartElementHandler = collector
p.EndElementHandler = collector
p.Parse(b'<e> <e/> <e></e> </e>', True)
tag = L[0]

assert len(L) == 6
for entry in L:

    assert tag is entry
print("InterningTest::test: ok")
