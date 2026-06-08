# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_empty_namespace_is_present"
# subject = "xml.dom.EMPTY_NAMESPACE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.EMPTY_NAMESPACE: api_empty_namespace_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "EMPTY_NAMESPACE")
print("api_empty_namespace_is_present OK")
