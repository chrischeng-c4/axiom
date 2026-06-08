# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "surface"
# case = "rgb_to_hsv_is_callable"
# subject = "colorsys.rgb_to_hsv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: rgb_to_hsv_is_callable (surface)."""
import colorsys

assert callable(colorsys.rgb_to_hsv)
print("rgb_to_hsv_is_callable OK")
