# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_parse_is_present"
# subject = "xml.sax.parse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.parse: api_parse_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "parse")
print("api_parse_is_present OK")
