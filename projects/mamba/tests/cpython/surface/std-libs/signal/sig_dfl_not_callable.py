# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "sig_dfl_not_callable"
# subject = "signal.SIG_DFL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.SIG_DFL: sig_dfl_not_callable (surface)."""
import signal

assert not callable(signal.SIG_DFL)
print("sig_dfl_not_callable OK")
