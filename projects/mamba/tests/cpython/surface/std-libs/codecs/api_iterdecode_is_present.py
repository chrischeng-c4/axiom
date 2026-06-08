# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_iterdecode_is_present"
# subject = "codecs.iterdecode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.iterdecode: api_iterdecode_is_present (surface)."""
import codecs

assert hasattr(codecs, "iterdecode")
print("api_iterdecode_is_present OK")
