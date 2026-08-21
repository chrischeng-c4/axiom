# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "surface"
# case = "hsv_to_rgb_is_callable"
# subject = "colorsys.hsv_to_rgb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.hsv_to_rgb: hsv_to_rgb_is_callable (surface)."""
import colorsys

assert callable(colorsys.hsv_to_rgb)
print("hsv_to_rgb_is_callable OK")
