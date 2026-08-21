# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "surface"
# case = "namedtuple_special_form_attr"
# subject = "typing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: namedtuple_special_form_attr (surface)."""
import typing

assert hasattr(typing, "NamedTuple")
print("namedtuple_special_form_attr OK")
