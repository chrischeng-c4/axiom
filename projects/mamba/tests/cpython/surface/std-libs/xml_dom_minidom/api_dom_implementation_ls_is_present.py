# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "surface"
# case = "api_dom_implementation_ls_is_present"
# subject = "xml.dom.minidom.DOMImplementationLS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.minidom.DOMImplementationLS: api_dom_implementation_ls_is_present (surface)."""
import xml.dom.minidom

assert hasattr(xml.dom.minidom, "DOMImplementationLS")
print("api_dom_implementation_ls_is_present OK")
