# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "override_is_callable"
# subject = "typing.override"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.override: override_is_callable (surface)."""
import typing

assert callable(typing.override)
print("override_is_callable OK")
