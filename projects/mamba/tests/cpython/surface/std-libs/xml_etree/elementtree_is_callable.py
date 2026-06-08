# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "surface"
# case = "elementtree_is_callable"
# subject = "xml.etree.ElementTree.ElementTree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""xml.etree.ElementTree.ElementTree: elementtree_is_callable (surface)."""
import xml.etree.ElementTree

assert callable(xml.etree.ElementTree.ElementTree)
print("elementtree_is_callable OK")
