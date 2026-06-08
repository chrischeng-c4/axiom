# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "api_fromstringlist_is_present"
# subject = "xml.etree.ElementTree.fromstringlist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstringlist: api_fromstringlist_is_present (surface)."""
import xml.etree.ElementTree

assert hasattr(xml.etree.ElementTree, "fromstringlist")
print("api_fromstringlist_is_present OK")
