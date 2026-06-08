# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigxfsz_is_present"
# subject = "signal.SIGXFSZ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIGXFSZ: api_sigxfsz_is_present (surface)."""
import signal

assert hasattr(signal, "SIGXFSZ")
print("api_sigxfsz_is_present OK")
