# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "errors"
# case = "element_index_out_of_range_raises"
# subject = "ET.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: element_index_out_of_range_raises (errors)."""
import xml.etree.ElementTree as ET

_raised = False
try:
    ET.Element("parent")[5]
except IndexError:
    _raised = True
assert _raised, "element_index_out_of_range_raises: expected IndexError"
print("element_index_out_of_range_raises OK")
