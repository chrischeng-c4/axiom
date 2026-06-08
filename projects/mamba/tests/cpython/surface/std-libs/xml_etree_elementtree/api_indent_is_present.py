# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "api_indent_is_present"
# subject = "xml.etree.ElementTree.indent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.etree.ElementTree.indent: api_indent_is_present (surface)."""
import xml.etree.ElementTree

assert hasattr(xml.etree.ElementTree, "indent")
print("api_indent_is_present OK")
