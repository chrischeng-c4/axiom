# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "xml_is_callable"
# subject = "ET.XML"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.XML: xml_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.XML)
print("xml_is_callable OK")
