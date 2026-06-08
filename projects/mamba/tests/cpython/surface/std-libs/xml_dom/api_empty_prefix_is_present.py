# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_empty_prefix_is_present"
# subject = "xml.dom.EMPTY_PREFIX"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.EMPTY_PREFIX: api_empty_prefix_is_present (surface)."""
import xml.dom

assert hasattr(xml.dom, "EMPTY_PREFIX")
print("api_empty_prefix_is_present OK")
