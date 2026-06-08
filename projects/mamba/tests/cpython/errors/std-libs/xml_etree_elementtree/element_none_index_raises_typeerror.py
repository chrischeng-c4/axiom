# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "element_none_index_raises_typeerror"
# subject = "ET.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: element_none_index_raises_typeerror (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.Element("parent")[None]
except TypeError:
    _raised = True
assert _raised, "element_none_index_raises_typeerror: expected TypeError"
print("element_none_index_raises_typeerror OK")
