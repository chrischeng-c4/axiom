# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_wrong_document_err_is_present"
# subject = "xml.dom.WRONG_DOCUMENT_ERR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.WRONG_DOCUMENT_ERR: api_wrong_document_err_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "WRONG_DOCUMENT_ERR")
print("api_wrong_document_err_is_present OK")
