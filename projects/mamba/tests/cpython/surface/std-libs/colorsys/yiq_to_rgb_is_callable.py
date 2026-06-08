# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "surface"
# case = "yiq_to_rgb_is_callable"
# subject = "colorsys.yiq_to_rgb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.yiq_to_rgb: yiq_to_rgb_is_callable (surface)."""
import colorsys

assert callable(colorsys.yiq_to_rgb)
print("yiq_to_rgb_is_callable OK")
