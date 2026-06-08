# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "element_is_callable"
# subject = "ET.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.Element: element_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.Element)
print("element_is_callable OK")
