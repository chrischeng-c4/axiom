# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "unhashable_annotated_metadata_raises"
# subject = "typing.Annotated"
# kind = "mechanical"
# xfail = "mamba does not raise hashing an Annotated form with unhashable metadata (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Annotated: unhashable_annotated_metadata_raises (errors)."""
import typing

_raised = False
try:
    hash(typing.Annotated[int, []])
except TypeError:
    _raised = True
assert _raised, "unhashable_annotated_metadata_raises: expected TypeError"
print("unhashable_annotated_metadata_raises OK")
