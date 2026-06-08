# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "api_tree_builder_is_present"
# subject = "xml.etree.ElementTree.TreeBuilder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.etree.ElementTree.TreeBuilder: api_tree_builder_is_present (surface)."""
import xml.etree.ElementTree

assert hasattr(xml.etree.ElementTree, "TreeBuilder")
print("api_tree_builder_is_present OK")
