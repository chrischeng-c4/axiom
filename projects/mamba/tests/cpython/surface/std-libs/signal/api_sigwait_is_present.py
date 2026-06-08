# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sigwait_is_present"
# subject = "signal.sigwait"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.sigwait: api_sigwait_is_present (surface)."""
import signal

assert hasattr(signal, "sigwait")
print("api_sigwait_is_present OK")
