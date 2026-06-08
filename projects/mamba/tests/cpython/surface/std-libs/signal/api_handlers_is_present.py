# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_handlers_is_present"
# subject = "signal.Handlers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.Handlers: api_handlers_is_present (surface)."""
import signal

assert hasattr(signal, "Handlers")
print("api_handlers_is_present OK")
