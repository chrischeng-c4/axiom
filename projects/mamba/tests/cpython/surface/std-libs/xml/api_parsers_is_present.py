# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml"
# dimension = "surface"
# case = "api_parsers_is_present"
# subject = "xml.parsers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.parsers: api_parsers_is_present (surface)."""
import xml.parsers

assert hasattr(xml, "parsers")
print("api_parsers_is_present OK")
