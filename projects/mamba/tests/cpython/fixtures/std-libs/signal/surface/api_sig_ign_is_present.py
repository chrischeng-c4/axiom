# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "api_sig_ign_is_present"
# subject = "signal.SIG_IGN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""signal.SIG_IGN: api_sig_ign_is_present (surface)."""
import signal

assert hasattr(signal, "SIG_IGN")
print("api_sig_ign_is_present OK")
