# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "behavior"
# case = "empty_node_list_test_case__test_emptynodelist_length"
# subject = "cpython.test_xml_dom_minicompat.EmptyNodeListTestCase.test_emptynodelist_length"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_minicompat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_minicompat.py::EmptyNodeListTestCase::test_emptynodelist_length
"""Auto-ported test: EmptyNodeListTestCase::test_emptynodelist_length (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
import xml.dom
from xml.dom.minicompat import *


# --- test body ---
node_list = EmptyNodeList()

assert node_list.length == 0
try:
    node_list.length = 111
    raise AssertionError('expected xml.dom.NoModificationAllowedErr')
except xml.dom.NoModificationAllowedErr:
    pass
print("EmptyNodeListTestCase::test_emptynodelist_length: ok")
