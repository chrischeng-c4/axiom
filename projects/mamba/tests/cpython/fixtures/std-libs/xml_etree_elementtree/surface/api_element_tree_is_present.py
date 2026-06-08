# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "api_element_tree_is_present"
# subject = "xml.etree.ElementTree.ElementTree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.etree.ElementTree.ElementTree: api_element_tree_is_present (surface)."""
import xml.etree.ElementTree

assert hasattr(xml.etree.ElementTree, "ElementTree")
print("api_element_tree_is_present OK")
