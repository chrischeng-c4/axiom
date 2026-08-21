# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "surface"
# case = "dict_special_form_attr"
# subject = "typing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: dict_special_form_attr (surface)."""
import typing

assert hasattr(typing, "Dict")
print("dict_special_form_attr OK")
