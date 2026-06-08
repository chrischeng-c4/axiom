# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_node_is_present"
# subject = "xml.dom.Node"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.Node: api_node_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "Node")
print("api_node_is_present OK")
