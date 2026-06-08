# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_frozen_instance_error_is_present"
# subject = "dataclasses.FrozenInstanceError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.FrozenInstanceError: api_frozen_instance_error_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "FrozenInstanceError")
print("api_frozen_instance_error_is_present OK")
