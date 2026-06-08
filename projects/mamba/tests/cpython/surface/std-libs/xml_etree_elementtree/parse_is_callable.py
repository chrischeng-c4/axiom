# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "parse_is_callable"
# subject = "ET.parse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.parse: parse_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.parse)
print("parse_is_callable OK")
