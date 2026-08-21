# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minicompat"
# dimension = "behavior"
# case = "node_list_test_case__test_nodelist_pickle_roundtrip"
# subject = "cpython.test_xml_dom_minicompat.NodeListTestCase.test_nodelist_pickle_roundtrip"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_dom_minicompat.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xml_dom_minicompat.py::NodeListTestCase::test_nodelist_pickle_roundtrip
"""Auto-ported test: NodeListTestCase::test_nodelist_pickle_roundtrip (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
import xml.dom
from xml.dom.minicompat import *


# --- test body ---
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    node_list = NodeList()
    pickled = pickle.dumps(node_list, proto)
    unpickled = pickle.loads(pickled)

    assert unpickled is not node_list

    assert unpickled == node_list
    node_list.append(1)
    node_list.append(2)
    pickled = pickle.dumps(node_list, proto)
    unpickled = pickle.loads(pickled)

    assert unpickled is not node_list

    assert unpickled == node_list
print("NodeListTestCase::test_nodelist_pickle_roundtrip: ok")
