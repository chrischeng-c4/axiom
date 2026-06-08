# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_dom_exception_is_present"
# subject = "xml.dom.DOMException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.DOMException: api_dom_exception_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "DOMException")
print("api_dom_exception_is_present OK")
