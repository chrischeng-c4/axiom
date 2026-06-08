# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_incremental_encoder_is_present"
# subject = "codecs.IncrementalEncoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.IncrementalEncoder: api_incremental_encoder_is_present (surface)."""
import codecs

assert hasattr(codecs, "IncrementalEncoder")
print("api_incremental_encoder_is_present OK")
