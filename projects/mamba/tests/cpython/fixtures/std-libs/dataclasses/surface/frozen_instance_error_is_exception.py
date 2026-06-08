# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "frozen_instance_error_is_exception"
# subject = "dataclasses.FrozenInstanceError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""dataclasses.FrozenInstanceError: frozen_instance_error_is_exception (surface)."""
import dataclasses

assert hasattr(dataclasses.FrozenInstanceError, "__cause__")
print("frozen_instance_error_is_exception OK")
