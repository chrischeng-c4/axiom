# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_handler_is_present"
# subject = "xml.sax.handler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.handler: api_handler_is_present (surface)."""
import xml.sax.handler

assert hasattr(xml.sax, "handler")
print("api_handler_is_present OK")
