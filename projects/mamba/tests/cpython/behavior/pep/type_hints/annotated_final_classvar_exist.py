# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "annotated_final_classvar_exist"
# subject = "typing"
# kind = "semantic"
# xfail = "mamba diverges on the typing special-form surface (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: the documented PEP 484/586/591 special forms are present on the typing module: hasattr(typing,'Annotated'), hasattr(typing,'Final') and hasattr(typing,'ClassVar') are all True"""
import typing

assert hasattr(typing, "Annotated"), "typing.Annotated exists"
assert hasattr(typing, "Final"), "typing.Final exists"
assert hasattr(typing, "ClassVar"), "typing.ClassVar exists"

print("annotated_final_classvar_exist OK")
