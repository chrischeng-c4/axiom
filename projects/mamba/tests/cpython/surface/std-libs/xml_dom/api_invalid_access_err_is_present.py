# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_invalid_access_err_is_present"
# subject = "xml.dom.INVALID_ACCESS_ERR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.INVALID_ACCESS_ERR: api_invalid_access_err_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "INVALID_ACCESS_ERR")
print("api_invalid_access_err_is_present OK")
