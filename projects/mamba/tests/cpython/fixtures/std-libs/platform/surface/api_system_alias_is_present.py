# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_system_alias_is_present"
# subject = "platform.system_alias"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.system_alias: api_system_alias_is_present (surface)."""
import platform

assert hasattr(platform, "system_alias")
print("api_system_alias_is_present OK")
