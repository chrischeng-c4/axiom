# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "behavior"
# case = "empty_node_list_test_case__test_emptynodelist_item"
# subject = "cpython.test_xml_dom_minicompat.EmptyNodeListTestCase.test_emptynodelist_item"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_minicompat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_minicompat.py::EmptyNodeListTestCase::test_emptynodelist_item
"""Auto-ported test: EmptyNodeListTestCase::test_emptynodelist_item (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
import xml.dom
from xml.dom.minicompat import *


# --- test body ---
node_list = EmptyNodeList()

assert node_list.item(0) is None

assert node_list.item(-1) is None
try:
    node_list[0]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    node_list[-1]
    raise AssertionError('expected IndexError')
except IndexError:
    pass
print("EmptyNodeListTestCase::test_emptynodelist_item: ok")
