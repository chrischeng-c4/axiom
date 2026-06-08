# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "rootless_tree_write_raises"
# subject = "ET.ElementTree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.ElementTree: an ElementTree() built with no root has getroot() is None, so write() raises (AttributeError under CPython 3.12) rather than emitting a document"""
import xml.etree.ElementTree as ET

import io
t = ET.ElementTree()
assert t.getroot() is None, "a rootless ElementTree has getroot() is None"
_raised = False
try:
    t.write(io.BytesIO())
except (AttributeError, TypeError):
    _raised = True
assert _raised, "rootless ElementTree.write() must raise"

print("rootless_tree_write_raises OK")
