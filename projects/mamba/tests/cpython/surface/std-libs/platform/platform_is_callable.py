# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "platform_is_callable"
# subject = "platform.platform"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.platform: platform_is_callable (surface)."""
import platform

assert callable(platform.platform)
print("platform_is_callable OK")
