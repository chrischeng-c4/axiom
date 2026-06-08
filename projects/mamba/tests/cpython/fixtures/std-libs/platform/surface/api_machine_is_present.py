# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_machine_is_present"
# subject = "platform.machine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.machine: api_machine_is_present (surface)."""
import platform

assert hasattr(platform, "machine")
print("api_machine_is_present OK")
