# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "surface"
# case = "classvar_importable"
# subject = "typing.ClassVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.ClassVar: classvar_importable (surface)."""
import typing

assert hasattr(typing.ClassVar, "__getitem__")
print("classvar_importable OK")
