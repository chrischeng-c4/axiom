# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_validation_err_is_present"
# subject = "xml.dom.VALIDATION_ERR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.VALIDATION_ERR: api_validation_err_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "VALIDATION_ERR")
print("api_validation_err_is_present OK")
