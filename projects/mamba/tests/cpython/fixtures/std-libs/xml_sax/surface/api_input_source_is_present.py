# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_sax"
# dimension = "surface"
# case = "api_input_source_is_present"
# subject = "xml.sax.InputSource"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""xml.sax.InputSource: api_input_source_is_present (surface)."""
import xml.sax

assert hasattr(xml.sax, "InputSource")
print("api_input_source_is_present OK")
