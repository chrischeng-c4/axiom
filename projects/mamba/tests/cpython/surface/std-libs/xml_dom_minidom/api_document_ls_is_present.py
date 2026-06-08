# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "surface"
# case = "api_document_ls_is_present"
# subject = "xml.dom.minidom.DocumentLS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.minidom.DocumentLS: api_document_ls_is_present (surface)."""
import xml.dom.minidom

assert hasattr(xml.dom.minidom, "DocumentLS")
print("api_document_ls_is_present OK")
