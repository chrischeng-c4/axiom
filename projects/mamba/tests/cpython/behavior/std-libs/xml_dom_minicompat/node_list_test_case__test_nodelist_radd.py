# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "behavior"
# case = "node_list_test_case__test_nodelist_radd"
# subject = "cpython.test_xml_dom_minicompat.NodeListTestCase.test_nodelist___radd__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_minicompat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_minicompat.py::NodeListTestCase::test_nodelist___radd__
"""Auto-ported test: NodeListTestCase::test_nodelist___radd__ (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
import xml.dom
from xml.dom.minicompat import *


# --- test body ---
node_list = [1, 2] + NodeList([3, 4])

assert node_list == NodeList([1, 2, 3, 4])
print("NodeListTestCase::test_nodelist___radd__: ok")
