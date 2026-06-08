# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml"
# dimension = "surface"
# case = "api_dom_is_present"
# subject = "xml.dom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom: api_dom_is_present (surface)."""
import xml.dom

assert hasattr(xml, "dom")
print("api_dom_is_present OK")
