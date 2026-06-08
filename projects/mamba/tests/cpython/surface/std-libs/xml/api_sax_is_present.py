# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml"
# dimension = "surface"
# case = "api_sax_is_present"
# subject = "xml.sax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax: api_sax_is_present (surface)."""
import xml.sax

assert hasattr(xml, "sax")
print("api_sax_is_present OK")
