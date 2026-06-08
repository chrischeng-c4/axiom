# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom"
# dimension = "surface"
# case = "api_domreg_is_present"
# subject = "xml.dom.domreg"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.domreg: api_domreg_is_present (surface)."""
import xml.dom.domreg

assert hasattr(xml.dom, "domreg")
print("api_domreg_is_present OK")
