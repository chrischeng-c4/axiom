# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_codec_is_present"
# subject = "codecs.Codec"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.Codec: api_codec_is_present (surface)."""
import codecs

assert hasattr(codecs, "Codec")
print("api_codec_is_present OK")
