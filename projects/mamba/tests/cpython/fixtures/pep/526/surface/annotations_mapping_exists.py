# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "526"
# dimension = "surface"
# case = "annotations_mapping_exists"
# subject = "__annotations__"
# kind = "mechanical"
# xfail = "module __annotations__ is an undefined name on mamba (force-typed; the mapping is never materialized). See project_mamba_pep_silent_divergences_2026_05_27."
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""__annotations__: annotations_mapping_exists (surface)."""
import sys

assert hasattr(__annotations__, "keys")
print("annotations_mapping_exists OK")
