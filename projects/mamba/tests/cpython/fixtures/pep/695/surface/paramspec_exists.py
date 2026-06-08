# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "surface"
# case = "paramspec_exists"
# subject = "typing.ParamSpec"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.ParamSpec: paramspec_exists (surface)."""
import typing

assert callable(typing.ParamSpec)
print("paramspec_exists OK")
