# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_sax_not_supported_exception_is_present"
# subject = "xml.sax.SAXNotSupportedException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.SAXNotSupportedException: api_sax_not_supported_exception_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "SAXNotSupportedException")
print("api_sax_not_supported_exception_is_present OK")
