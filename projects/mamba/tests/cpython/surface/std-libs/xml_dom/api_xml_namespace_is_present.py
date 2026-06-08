# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_xml_namespace_is_present"
# subject = "xml.dom.XML_NAMESPACE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.XML_NAMESPACE: api_xml_namespace_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "XML_NAMESPACE")
print("api_xml_namespace_is_present OK")
