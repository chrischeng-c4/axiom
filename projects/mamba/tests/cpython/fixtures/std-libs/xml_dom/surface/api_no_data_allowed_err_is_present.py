# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_no_data_allowed_err_is_present"
# subject = "xml.dom.NO_DATA_ALLOWED_ERR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.NO_DATA_ALLOWED_ERR: api_no_data_allowed_err_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "NO_DATA_ALLOWED_ERR")
print("api_no_data_allowed_err_is_present OK")
