# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "elementtree_is_callable"
# subject = "ET.ElementTree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.ElementTree: elementtree_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.ElementTree)
print("elementtree_is_callable OK")
