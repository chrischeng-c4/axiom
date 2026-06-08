# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_xhtml_namespace_is_present"
# subject = "xml.dom.XHTML_NAMESPACE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.XHTML_NAMESPACE: api_xhtml_namespace_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "XHTML_NAMESPACE")
print("api_xhtml_namespace_is_present OK")
