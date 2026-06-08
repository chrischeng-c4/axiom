# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "variable_annotations"
# dimension = "surface"
# case = "annotations_mapping_exists"
# subject = "__annotations__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: annotations_mapping_exists (surface)."""
import typing

assert hasattr(__annotations__, "keys")
print("annotations_mapping_exists OK")
