# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_wrong_document_err_is_present_2"
# subject = "xml.dom.WrongDocumentErr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.WrongDocumentErr: api_wrong_document_err_is_present_2 (surface)."""
import xml.dom

assert hasattr(xml.dom, "WrongDocumentErr")
print("api_wrong_document_err_is_present_2 OK")
