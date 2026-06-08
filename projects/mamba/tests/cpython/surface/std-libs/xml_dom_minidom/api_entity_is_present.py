# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "surface"
# case = "api_entity_is_present"
# subject = "xml.dom.minidom.Entity"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.minidom.Entity: api_entity_is_present (surface)."""
import xml.dom.minidom

assert hasattr(xml.dom.minidom, "Entity")
print("api_entity_is_present OK")
