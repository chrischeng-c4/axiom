# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "fromstring_is_callable"
# subject = "ET.fromstring"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.fromstring: fromstring_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.fromstring)
print("fromstring_is_callable OK")
