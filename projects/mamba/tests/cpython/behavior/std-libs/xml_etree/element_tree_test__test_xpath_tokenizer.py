# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "element_tree_test__test_xpath_tokenizer"
# subject = "cpython.test_xml_etree.ElementTreeTest.test_xpath_tokenizer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
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
import warnings
import weakref
from contextlib import nullcontext
from functools import partial
from itertools import product, islice

def serialize_check(elem, expected):
    assert serialize(elem) == expected
from xml.etree import ElementPath

def check(p, expected, namespaces=None):
    assert [op or tag for op, tag in ElementPath.xpath_tokenizer(p, namespaces)] == expected
check('*', ['*'])
check('text()', ['text', '()'])
check('@name', ['@', 'name'])
check('@*', ['@', '*'])
check('para[1]', ['para', '[', '1', ']'])
check('para[last()]', ['para', '[', 'last', '()', ']'])
check('*/para', ['*', '/', 'para'])
check('/doc/chapter[5]/section[2]', ['/', 'doc', '/', 'chapter', '[', '5', ']', '/', 'section', '[', '2', ']'])
check('chapter//para', ['chapter', '//', 'para'])
check('//para', ['//', 'para'])
check('//olist/item', ['//', 'olist', '/', 'item'])
check('.', ['.'])
check('.//para', ['.', '//', 'para'])
check('..', ['..'])
check('../@lang', ['..', '/', '@', 'lang'])
check('chapter[title]', ['chapter', '[', 'title', ']'])
check('employee[@secretary and @assistant]', ['employee', '[', '@', 'secretary', '', 'and', '', '@', 'assistant', ']'])
check('@{ns}attr', ['@', '{ns}attr'])
check('{http://spam}egg', ['{http://spam}egg'])
check('./spam.egg', ['.', '/', 'spam.egg'])
check('.//{http://spam}egg', ['.', '//', '{http://spam}egg'])
check('{ns}*', ['{ns}*'])
check('{}*', ['{}*'])
check('{*}tag', ['{*}tag'])
check('{*}*', ['{*}*'])
check('.//{*}tag', ['.', '//', '{*}tag'])
check('./xsd:type', ['.', '/', '{http://www.w3.org/2001/XMLSchema}type'], {'xsd': 'http://www.w3.org/2001/XMLSchema'})
check('type', ['{http://www.w3.org/2001/XMLSchema}type'], {'': 'http://www.w3.org/2001/XMLSchema'})
check('@xsd:type', ['@', '{http://www.w3.org/2001/XMLSchema}type'], {'xsd': 'http://www.w3.org/2001/XMLSchema'})
check('@type', ['@', 'type'], {'': 'http://www.w3.org/2001/XMLSchema'})
check('@{*}type', ['@', '{*}type'], {'': 'http://www.w3.org/2001/XMLSchema'})
check('@{ns}attr', ['@', '{ns}attr'], {'': 'http://www.w3.org/2001/XMLSchema', 'ns': 'http://www.w3.org/2001/XMLSchema'})

print("ElementTreeTest::test_xpath_tokenizer: ok")
