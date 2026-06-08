# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_inuse_attribute_err_is_present"
# subject = "xml.dom.INUSE_ATTRIBUTE_ERR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.INUSE_ATTRIBUTE_ERR: api_inuse_attribute_err_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "INUSE_ATTRIBUTE_ERR")
print("api_inuse_attribute_err_is_present OK")
