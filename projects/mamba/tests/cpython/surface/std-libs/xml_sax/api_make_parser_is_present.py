# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_make_parser_is_present"
# subject = "xml.sax.make_parser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.make_parser: api_make_parser_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "make_parser")
print("api_make_parser_is_present OK")
