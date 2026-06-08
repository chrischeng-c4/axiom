# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "xmlparser_is_callable"
# subject = "ET.XMLParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.XMLParser: xmlparser_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.XMLParser)
print("xmlparser_is_callable OK")
