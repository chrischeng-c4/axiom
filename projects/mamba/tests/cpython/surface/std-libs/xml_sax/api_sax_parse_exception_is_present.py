# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_sax_parse_exception_is_present"
# subject = "xml.sax.SAXParseException"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.SAXParseException: api_sax_parse_exception_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "SAXParseException")
print("api_sax_parse_exception_is_present OK")
