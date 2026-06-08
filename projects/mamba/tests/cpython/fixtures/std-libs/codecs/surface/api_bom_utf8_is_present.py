# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_bom_utf8_is_present"
# subject = "codecs.BOM_UTF8"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.BOM_UTF8: api_bom_utf8_is_present (surface)."""
import codecs

assert hasattr(codecs, "BOM_UTF8")
print("api_bom_utf8_is_present OK")
