# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_set_wakeup_fd_is_present"
# subject = "signal.set_wakeup_fd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.set_wakeup_fd: api_set_wakeup_fd_is_present (surface)."""
import signal

assert hasattr(signal, "set_wakeup_fd")
print("api_set_wakeup_fd_is_present OK")
