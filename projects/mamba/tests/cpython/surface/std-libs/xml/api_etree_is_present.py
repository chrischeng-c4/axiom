# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml"
# dimension = "surface"
# case = "api_etree_is_present"
# subject = "xml.etree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.etree: api_etree_is_present (surface)."""
import xml.etree

assert hasattr(xml, "etree")
print("api_etree_is_present OK")
