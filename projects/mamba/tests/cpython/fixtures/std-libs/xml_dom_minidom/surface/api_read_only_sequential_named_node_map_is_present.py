# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "surface"
# case = "api_read_only_sequential_named_node_map_is_present"
# subject = "xml.dom.minidom.ReadOnlySequentialNamedNodeMap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.minidom.ReadOnlySequentialNamedNodeMap: api_read_only_sequential_named_node_map_is_present (surface)."""
import xml.dom.minidom

assert hasattr(xml.dom.minidom, "ReadOnlySequentialNamedNodeMap")
print("api_read_only_sequential_named_node_map_is_present OK")
