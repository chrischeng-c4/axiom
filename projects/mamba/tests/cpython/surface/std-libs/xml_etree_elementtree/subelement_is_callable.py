# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "subelement_is_callable"
# subject = "ET.SubElement"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.SubElement: subelement_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.SubElement)
print("subelement_is_callable OK")
