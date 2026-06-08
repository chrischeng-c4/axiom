# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_default_int_handler_is_present"
# subject = "signal.default_int_handler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.default_int_handler: api_default_int_handler_is_present (surface)."""
import signal

assert hasattr(signal, "default_int_handler")
print("api_default_int_handler_is_present OK")
