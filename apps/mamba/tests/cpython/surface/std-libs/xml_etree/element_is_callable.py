# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "surface"
# case = "element_is_callable"
# subject = "xml.etree.ElementTree.Element"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""xml.etree.ElementTree.Element: element_is_callable (surface)."""
import xml.etree.ElementTree

assert callable(xml.etree.ElementTree.Element)
print("element_is_callable OK")
