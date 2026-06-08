# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "tostring_is_callable"
# subject = "ET.tostring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.tostring: tostring_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.tostring)
print("tostring_is_callable OK")
