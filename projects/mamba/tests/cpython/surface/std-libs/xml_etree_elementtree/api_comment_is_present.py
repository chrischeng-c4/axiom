# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "api_comment_is_present"
# subject = "xml.etree.ElementTree.Comment"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.etree.ElementTree.Comment: api_comment_is_present (surface)."""
import xml.etree.ElementTree

assert hasattr(xml.etree.ElementTree, "Comment")
print("api_comment_is_present OK")
