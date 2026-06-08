# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "surface"
# case = "hls_to_rgb_is_callable"
# subject = "colorsys.hls_to_rgb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.hls_to_rgb: hls_to_rgb_is_callable (surface)."""
import colorsys

assert callable(colorsys.hls_to_rgb)
print("hls_to_rgb_is_callable OK")
