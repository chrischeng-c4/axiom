# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_not_found_err_is_present_2"
# subject = "xml.dom.NotFoundErr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.NotFoundErr: api_not_found_err_is_present_2 (surface)."""
import xml.dom

assert hasattr(xml.dom, "NotFoundErr")
print("api_not_found_err_is_present_2 OK")
