# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "errors"
# case = "rgb_to_yiq_too_few_args_raises"
# subject = "colorsys.rgb_to_yiq"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_yiq: rgb_to_yiq_too_few_args_raises (errors)."""
import colorsys

_raised = False
try:
    colorsys.rgb_to_yiq(0.5)
except TypeError:
    _raised = True
assert _raised, "rgb_to_yiq_too_few_args_raises: expected TypeError"
print("rgb_to_yiq_too_few_args_raises OK")
