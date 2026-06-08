# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_index_size_err_is_present"
# subject = "xml.dom.INDEX_SIZE_ERR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.INDEX_SIZE_ERR: api_index_size_err_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "INDEX_SIZE_ERR")
print("api_index_size_err_is_present OK")
