# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "surface"
# case = "final_special_form_attr"
# subject = "typing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: final_special_form_attr (surface)."""
import typing

assert hasattr(typing, "Final")
print("final_special_form_attr OK")
