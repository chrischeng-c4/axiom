# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_inuse_attribute_err_is_present_2"
# subject = "xml.dom.InuseAttributeErr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.InuseAttributeErr: api_inuse_attribute_err_is_present_2 (surface)."""
import xml.dom

assert hasattr(xml.dom, "InuseAttributeErr")
print("api_inuse_attribute_err_is_present_2 OK")
