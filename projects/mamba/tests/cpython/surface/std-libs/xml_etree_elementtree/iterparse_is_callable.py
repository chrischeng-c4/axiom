# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "iterparse_is_callable"
# subject = "ET.iterparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.iterparse: iterparse_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.iterparse)
print("iterparse_is_callable OK")
