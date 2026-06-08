# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_bom64_le_is_present"
# subject = "codecs.BOM64_LE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.BOM64_LE: api_bom64_le_is_present (surface)."""
import codecs

assert hasattr(codecs, "BOM64_LE")
print("api_bom64_le_is_present OK")
