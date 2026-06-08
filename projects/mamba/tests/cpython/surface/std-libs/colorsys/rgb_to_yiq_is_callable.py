# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "surface"
# case = "rgb_to_yiq_is_callable"
# subject = "colorsys.rgb_to_yiq"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_yiq: rgb_to_yiq_is_callable (surface)."""
import colorsys

assert callable(colorsys.rgb_to_yiq)
print("rgb_to_yiq_is_callable OK")
