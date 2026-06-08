# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "errors"
# case = "rgb_to_hsv_str_input_raises"
# subject = "colorsys.rgb_to_hsv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: rgb_to_hsv_str_input_raises (errors)."""
import colorsys

_raised = False
try:
    colorsys.rgb_to_hsv("a", "b", "c")
except TypeError:
    _raised = True
assert _raised, "rgb_to_hsv_str_input_raises: expected TypeError"
print("rgb_to_hsv_str_input_raises OK")
