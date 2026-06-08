# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "behavior"
# case = "node_list_test_case__test_nodelist_deepcopy"
# subject = "cpython.test_xml_dom_minicompat.NodeListTestCase.test_nodelist_deepcopy"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_minicompat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_minicompat.py::NodeListTestCase::test_nodelist_deepcopy
"""Auto-ported test: NodeListTestCase::test_nodelist_deepcopy (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
import xml.dom
from xml.dom.minicompat import *


# --- test body ---
node_list = NodeList()
copied = copy.deepcopy(node_list)

assert copied is not node_list

assert copied == node_list
node_list.append([1])
node_list.append([2])
copied = copy.deepcopy(node_list)

assert copied is not node_list

assert copied == node_list
for x, y in zip(copied, node_list):

    assert x is not y

    assert x == y
print("NodeListTestCase::test_nodelist_deepcopy: ok")
