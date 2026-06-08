# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "release_is_callable"
# subject = "platform.release"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.release: release_is_callable (surface)."""
import platform

assert callable(platform.release)
print("release_is_callable OK")
