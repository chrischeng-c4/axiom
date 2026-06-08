# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "qname_is_callable"
# subject = "ET.QName"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.QName: qname_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.QName)
print("qname_is_callable OK")
