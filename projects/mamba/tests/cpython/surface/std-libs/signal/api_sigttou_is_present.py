# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigttou_is_present"
# subject = "signal.SIGTTOU"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGTTOU: api_sigttou_is_present (surface)."""
import signal

assert hasattr(signal, "SIGTTOU")
print("api_sigttou_is_present OK")
