# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "surface"
# case = "subelement_is_callable"
# subject = "xml.etree.ElementTree.SubElement"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""xml.etree.ElementTree.SubElement: subelement_is_callable (surface)."""
import xml.etree.ElementTree

assert callable(xml.etree.ElementTree.SubElement)
print("subelement_is_callable OK")
