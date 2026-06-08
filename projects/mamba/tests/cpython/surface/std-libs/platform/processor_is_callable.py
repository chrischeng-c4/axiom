# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "processor_is_callable"
# subject = "platform.processor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""platform.processor: processor_is_callable (surface)."""
import platform

assert callable(platform.processor)
print("processor_is_callable OK")
