# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_no_modification_allowed_err_is_present_2"
# subject = "xml.dom.NoModificationAllowedErr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.NoModificationAllowedErr: api_no_modification_allowed_err_is_present_2 (surface)."""
import xml.dom

assert hasattr(xml.dom, "NoModificationAllowedErr")
print("api_no_modification_allowed_err_is_present_2 OK")
