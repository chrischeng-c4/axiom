# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "surface"
# case = "frozen_instance_error_is_attr"
# subject = "dataclasses"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses: frozen_instance_error_is_attr (surface)."""
import dataclasses

assert hasattr(dataclasses, "FrozenInstanceError")
print("frozen_instance_error_is_attr OK")
