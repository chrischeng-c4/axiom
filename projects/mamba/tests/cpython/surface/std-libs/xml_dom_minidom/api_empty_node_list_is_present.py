# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "surface"
# case = "api_empty_node_list_is_present"
# subject = "xml.dom.minidom.EmptyNodeList"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.minidom.EmptyNodeList: api_empty_node_list_is_present (surface)."""
import xml.dom.minidom

assert hasattr(xml.dom.minidom, "EmptyNodeList")
print("api_empty_node_list_is_present OK")
