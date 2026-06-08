# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "surface"
# case = "rgb_to_hls_is_callable"
# subject = "colorsys.rgb_to_hls"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hls: rgb_to_hls_is_callable (surface)."""
import colorsys

assert callable(colorsys.rgb_to_hls)
print("rgb_to_hls_is_callable OK")
