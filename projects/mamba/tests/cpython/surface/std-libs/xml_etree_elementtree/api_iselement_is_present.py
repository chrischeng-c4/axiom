# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_elementtree"
# dimension = "surface"
# case = "api_iselement_is_present"
# subject = "xml.etree.ElementTree.iselement"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.etree.ElementTree.iselement: api_iselement_is_present (surface)."""
import xml.etree.ElementTree

assert hasattr(xml.etree.ElementTree, "iselement")
print("api_iselement_is_present OK")
