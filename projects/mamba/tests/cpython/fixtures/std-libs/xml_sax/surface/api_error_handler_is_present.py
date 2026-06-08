# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_error_handler_is_present"
# subject = "xml.sax.ErrorHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.ErrorHandler: api_error_handler_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "ErrorHandler")
print("api_error_handler_is_present OK")
