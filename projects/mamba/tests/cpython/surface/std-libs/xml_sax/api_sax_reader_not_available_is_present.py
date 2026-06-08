# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_sax_reader_not_available_is_present"
# subject = "xml.sax.SAXReaderNotAvailable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.SAXReaderNotAvailable: api_sax_reader_not_available_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "SAXReaderNotAvailable")
print("api_sax_reader_not_available_is_present OK")
