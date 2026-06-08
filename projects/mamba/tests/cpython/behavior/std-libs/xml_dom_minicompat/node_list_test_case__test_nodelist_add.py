# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "behavior"
# case = "node_list_test_case__test_nodelist_add"
# subject = "cpython.test_xml_dom_minicompat.NodeListTestCase.test_nodelist___add__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_minicompat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_minicompat.py::NodeListTestCase::test_nodelist___add__
"""Auto-ported test: NodeListTestCase::test_nodelist___add__ (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
import xml.dom
from xml.dom.minicompat import *


# --- test body ---
node_list = NodeList([3, 4]) + [1, 2]

assert node_list == NodeList([3, 4, 1, 2])
print("NodeListTestCase::test_nodelist___add__: ok")
