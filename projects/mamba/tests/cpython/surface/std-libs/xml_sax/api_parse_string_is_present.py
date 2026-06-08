# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_parse_string_is_present"
# subject = "xml.sax.parseString"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.parseString: api_parse_string_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "parseString")
print("api_parse_string_is_present OK")
