# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_default_parser_list_is_present"
# subject = "xml.sax.default_parser_list"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.default_parser_list: api_default_parser_list_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "default_parser_list")
print("api_default_parser_list_is_present OK")
