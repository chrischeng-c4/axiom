# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "surface"
# case = "final_importable"
# subject = "typing.Final"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Final: final_importable (surface)."""
import typing

assert hasattr(typing.Final, "__getitem__")
print("final_importable OK")
