# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "indent_is_callable"
# subject = "ET.indent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.indent: indent_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.indent)
print("indent_is_callable OK")
