# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigmasks_is_present"
# subject = "signal.Sigmasks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.Sigmasks: api_sigmasks_is_present (surface)."""
import signal

assert hasattr(signal, "Sigmasks")
print("api_sigmasks_is_present OK")
