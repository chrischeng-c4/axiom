# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_xmlreader_is_present"
# subject = "xml.sax.xmlreader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.xmlreader: api_xmlreader_is_present (surface)."""
import xml.sax.xmlreader

assert hasattr(xml.sax, "xmlreader")
print("api_xmlreader_is_present OK")
