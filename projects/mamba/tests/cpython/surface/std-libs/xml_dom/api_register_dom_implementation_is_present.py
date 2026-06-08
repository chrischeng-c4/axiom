# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_register_dom_implementation_is_present"
# subject = "xml.dom.registerDOMImplementation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.registerDOMImplementation: api_register_dom_implementation_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "registerDOMImplementation")
print("api_register_dom_implementation_is_present OK")
