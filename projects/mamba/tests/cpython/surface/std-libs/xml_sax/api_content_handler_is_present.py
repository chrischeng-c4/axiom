# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_content_handler_is_present"
# subject = "xml.sax.ContentHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.ContentHandler: api_content_handler_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "ContentHandler")
print("api_content_handler_is_present OK")
