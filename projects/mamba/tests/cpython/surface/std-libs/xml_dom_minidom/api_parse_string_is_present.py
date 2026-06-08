# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_dom_minidom"
# dimension = "surface"
# case = "api_parse_string_is_present"
# subject = "xml.dom.minidom.parseString"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.dom.minidom.parseString: api_parse_string_is_present (surface)."""
import xml.dom.minidom

assert hasattr(xml.dom.minidom, "parseString")
print("api_parse_string_is_present OK")
