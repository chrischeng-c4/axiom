# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_domstring_size_err_is_present_2"
# subject = "xml.dom.DomstringSizeErr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.DomstringSizeErr: api_domstring_size_err_is_present_2 (surface)."""
import xml.dom

assert hasattr(xml.dom, "DomstringSizeErr")
print("api_domstring_size_err_is_present_2 OK")
