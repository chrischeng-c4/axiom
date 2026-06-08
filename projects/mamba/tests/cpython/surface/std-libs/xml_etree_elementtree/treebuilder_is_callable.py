# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "treebuilder_is_callable"
# subject = "ET.TreeBuilder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ET.TreeBuilder: treebuilder_is_callable (surface)."""
import xml.etree.ElementTree as ET

assert callable(ET.TreeBuilder)
print("treebuilder_is_callable OK")
