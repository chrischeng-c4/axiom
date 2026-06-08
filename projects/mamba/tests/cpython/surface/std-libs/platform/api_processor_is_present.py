# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_processor_is_present"
# subject = "platform.processor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.processor: api_processor_is_present (surface)."""
import platform

assert hasattr(platform, "processor")
print("api_processor_is_present OK")
